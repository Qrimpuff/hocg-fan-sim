use iter_tools::Itertools;

use super::error::*;
use std::fmt::{Debug, Display};
use std::str::FromStr;

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

pub trait TakeParam<T> {
    fn take_param(&self) -> Result<(T, &[Tokens])>;
}
impl<T> TakeParam<T> for [Tokens]
where
    T: ParseTokens + Debug,
{
    fn take_param(&self) -> Result<(T, &[Tokens])> {
        T::take_param(self)
    }
}
pub trait TakeString {
    fn take_string(&self) -> Result<(&String, &[Tokens])>;
}
impl TakeString for [Tokens] {
    fn take_string(&self) -> Result<(&String, &[Tokens])> {
        let t = self.first().ok_or(Error::ExpectedToken)?;
        if let Tokens::Token(s) = t {
            println!("take_string - {:?}", (s, &self[1..]));
            return Ok((s, &self[1..]));
        }
        Err(Error::ExpectedString)
    }
}

#[allow(unused)]
pub trait ParseTokens: Sized {
    fn parse_tokens(tokens: &[Tokens]) -> Result<(Self, &[Tokens])>;

    fn take_param<T: ParseTokens + Debug>(tokens: &[Tokens]) -> Result<(T, &[Tokens])> {
        let (ctx, is_sub_ctx) = Self::get_tokens_context(tokens)?;

        println!("take_param - before - {:?}", &ctx);
        let t = T::parse_tokens(ctx)?;
        println!("take_param - after - {:?}", &t);

        if is_sub_ctx {
            Ok((t.0, &tokens[1..]))
        } else {
            Ok(t)
        }
    }

    fn take_string<T: ParseTokens>(tokens: &[Tokens]) -> Result<(&String, &[Tokens])> {
        tokens.take_string()
    }

    fn get_tokens_context(tokens: &[Tokens]) -> Result<(&[Tokens], bool)> {
        let is_list = {
            let t = tokens.first().ok_or(Error::ExpectedToken)?;

            matches!(t, Tokens::List(_))
        };

        if is_list {
            let Tokens::List(v) = tokens.first().ok_or(Error::ExpectedToken)? else {
                unreachable!()
            };
            Ok((v, true))
        } else {
            Ok((tokens, false))
        }
    }

    fn from_str(s: &str) -> Result<Self> {
        Self::from_tokens(s.parse()?)
    }

    fn from_tokens(tokens: Tokens) -> Result<Self> {
        let mut tokens = match tokens {
            t @ Tokens::Token(_) => Vec::from([t]),
            Tokens::List(v) => v,
        };

        Self::parse_tokens(&tokens).and_then(|ok| {
            // check for remaining Tokens
            if ok.1.is_empty() {
                Ok(ok.0)
            } else {
                Err(Error::RemainingTokens)
            }
        })
    }
}

impl<T> ParseTokens for Vec<T>
where
    T: ParseTokens + Debug,
{
    fn parse_tokens(mut tokens: &[Tokens]) -> Result<(Self, &[Tokens])> {
        let mut v = Vec::new();
        while !tokens.is_empty() {
            let (param, t) = T::parse_tokens(tokens)?;
            tokens = t;
            v.push(param);
        }
        Ok((v, tokens))
    }
}

impl<T> ParseTokens for Box<T>
where
    T: ParseTokens + Debug,
{
    fn parse_tokens(tokens: &[Tokens]) -> Result<(Self, &[Tokens])> {
        let (param, t) = T::parse_tokens(tokens)?;
        Ok((Box::new(param), t))
    }
}

#[derive(Debug)]
pub enum Tokens {
    Token(String),
    List(Vec<Tokens>),
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
        fn add_token(list: &mut Vec<Tokens>, token: String) -> Result<()> {
            if !token.is_empty() {
                list.push(Tokens::Token(token));
            }
            Ok(())
        }

        let mut stack = Vec::new();
        let mut token = String::new();
        let mut list = Vec::new();
        let mut bracket_level = 0;

        for c in s.chars() {
            match c {
                '(' => {
                    bracket_level += 1;
                    add_token(&mut list, token)?;
                    token = String::new();
                    stack.push(list);
                    list = Vec::new();
                }
                ')' => {
                    bracket_level -= 1;
                    add_token(&mut list, token)?;
                    token = String::new();
                    let mut _list = stack.pop().ok_or(Error::MissingBracket)?;
                    if list.len() > 1 {
                        _list.push(Tokens::List(list));
                    } else {
                        _list.push(list.pop().ok_or(Error::NoTokens)?);
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
            Ok(list.pop().ok_or(Error::NoTokens)?)
        }
    }
}
