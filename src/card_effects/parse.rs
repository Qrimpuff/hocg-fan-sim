use iter_tools::Itertools;

use super::error::*;
use std::collections::{HashMap, VecDeque};
use std::fmt::Display;
use std::str::FromStr;
use std::sync::OnceLock;

pub trait SerializeEffect {
    fn serialize_effect(self) -> String;
}

impl<T: Into<Tokens>> SerializeEffect for T {
    fn serialize_effect(self) -> String {
        let tokens: Tokens = self.into();
        tokens.to_string()
    }
}

pub trait ParseEffect {
    fn parse_effect<F: ParseTokens>(&self) -> Result<F>;
}

impl ParseEffect for str {
    fn parse_effect<F: ParseTokens>(&self) -> Result<F> {
        ParseTokens::from_str(self)
    }
}

pub trait ParseTokens: Sized {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self>;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_tokens(s.parse()?)
    }

    fn from_tokens(tokens: Tokens) -> Result<Self> {
        let mut tokens = match tokens {
            t @ Tokens::Token(_) => VecDeque::from([t]),
            Tokens::List(v) => v,
        };

        Self::parse_tokens(&mut tokens).and_then(|ok| {
            // check for remaining Tokens
            if tokens.is_empty() {
                Ok(ok)
            } else {
                Err(Error::RemainingTokens)
            }
        })
    }

    fn take_param<T: ParseTokens>(tokens: &mut VecDeque<Tokens>) -> Result<T> {
        let (ctx, clean) = Self::get_tokens_context(tokens)?;
        let param = T::parse_tokens(ctx);
        if param.is_ok() && clean {
            Self::clean_list(tokens)?;
        }
        param
    }

    fn take_string(tokens: &mut VecDeque<Tokens>) -> Result<String> {
        let t = tokens.pop_front().ok_or(Error::ExpectedToken)?;
        if let Tokens::Token(s) = t {
            return Ok(s);
        }
        Err(Error::ExpectedString)
    }

    fn return_string(tokens: &mut VecDeque<Tokens>, s: String) {
        tokens.push_front(Tokens::Token(s));
    }

    fn get_tokens_context(tokens: &mut VecDeque<Tokens>) -> Result<(&mut VecDeque<Tokens>, bool)> {
        let is_list = {
            let t = tokens.get_mut(0).ok_or(Error::ExpectedToken)?;

            matches!(t, Tokens::List(_))
        };

        if is_list {
            let Tokens::List(v) = tokens.get_mut(0).ok_or(Error::ExpectedToken)? else {
                unreachable!()
            };
            Ok((v, true))
        } else {
            Ok((tokens, false))
        }
    }

    fn clean_list(tokens: &mut VecDeque<Tokens>) -> Result<()> {
        let t = tokens.pop_front().ok_or(Error::ExpectedToken)?;

        if let Tokens::List(v) = t {
            assert_eq!(v.len(), 0, "list not empty, shouldn't clean");
        } else {
            panic!("removing something we shouldn't")
        }
        Ok(())
    }

    fn infix_token_map() -> &'static HashMap<&'static str, &'static str> {
        static INFIX_TOKEN_MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
        INFIX_TOKEN_MAP.get_or_init(HashMap::new)
    }

    fn process_infix_tokens(tokens: &mut VecDeque<Tokens>) {
        let map = Self::infix_token_map();
        if map.is_empty() {
            return;
        }
        if let Some(Tokens::Token(second)) = tokens.get(1) {
            if let Some(token) = map.get(second.as_str()) {
                tokens.remove(1);
                tokens.push_front(Tokens::Token(token.to_string()));
            }
        }
    }
}

impl<T> ParseTokens for Vec<T>
where
    T: ParseTokens,
{
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        let mut v = Vec::new();
        while !tokens.is_empty() {
            v.push(T::take_param(tokens)?);
        }
        Ok(v)
    }
}

impl<T> ParseTokens for Box<T>
where
    T: ParseTokens,
{
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        Ok(Box::new(T::take_param(tokens)?))
    }
}

pub trait TakeParam<T> {
    fn take_param(&mut self) -> Result<T>;
}
impl<T> TakeParam<T> for VecDeque<Tokens>
where
    T: ParseTokens,
{
    fn take_param(&mut self) -> Result<T> {
        T::take_param(self)
    }
}

#[derive(Debug)]
pub enum Tokens {
    Token(String),
    List(VecDeque<Tokens>),
}

impl From<&str> for Tokens {
    fn from(value: &str) -> Self {
        Self::Token(value.into())
    }
}

impl<const N: usize> From<[Tokens; N]> for Tokens {
    fn from(value: [Tokens; N]) -> Self {
        Self::List(value.into())
    }
}

impl<T: Into<Tokens>> From<Vec<T>> for Tokens {
    fn from(value: Vec<T>) -> Self {
        Self::List(value.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<Tokens>> From<Box<T>> for Tokens {
    fn from(value: Box<T>) -> Self {
        (*value).into()
    }
}

impl Display for Tokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tokens::Token(t) => write!(f, "{t}"),
            Tokens::List(v) => {
                write!(
                    f,
                    "({})",
                    v.iter().map(ToString::to_string).collect_vec().join(" ")
                )
            }
        }
    }
}

impl FromStr for Tokens {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        fn add_token(list: &mut VecDeque<Tokens>, token: String) -> Result<()> {
            if !token.is_empty() {
                list.push_back(Tokens::Token(token));
            }
            Ok(())
        }

        let mut stack = VecDeque::new();
        let mut token = String::new();
        let mut list = VecDeque::new();
        let mut bracket_level = 0;

        for c in s.chars() {
            match c {
                '(' => {
                    bracket_level += 1;
                    add_token(&mut list, token)?;
                    token = String::new();
                    stack.push_back(list);
                    list = VecDeque::new();
                }
                ')' => {
                    bracket_level -= 1;
                    add_token(&mut list, token)?;
                    token = String::new();
                    let mut _list = stack.pop_back().ok_or(Error::MissingBracket)?;
                    if list.len() > 1 {
                        _list.push_back(Tokens::List(list));
                    } else {
                        _list.push_back(list.pop_back().ok_or(Error::NoTokens)?);
                    }
                    list = _list;
                }
                c if c.is_whitespace() => {
                    add_token(&mut list, token)?;
                    token = String::new();
                }
                c => token.push(c),
            }
        }
        add_token(&mut list, token)?;

        // check balanced bracket
        if bracket_level != 0 {
            return Err(Error::UnbalancedBrackets);
        }

        if list.len() > 1 {
            Ok(Tokens::List(list))
        } else {
            Ok(list.pop_back().ok_or(Error::NoTokens)?)
        }
    }
}
