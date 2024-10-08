use iter_tools::Itertools;
use tracing::error;

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
            // println!("take_string - {:?}", (s, &self[1..]));
            return Ok((s, &self[1..]));
        }
        Err(Error::ExpectedString)
    }
}

#[allow(unused)]
pub trait ParseTokens: Debug + Sized {
    fn parse_tokens(tokens: &[Tokens]) -> Result<(Self, &[Tokens])>;

    fn take_param<T: ParseTokens + Debug>(tokens: &[Tokens]) -> Result<(T, &[Tokens])> {
        let (ctx, is_sub_ctx) = Self::get_tokens_context(tokens)?;

        // println!("take_param - before - {:?}", &ctx);
        let t = T::parse_tokens(ctx)?;
        // println!("take_param - after - {:?}", &t);

        if is_sub_ctx {
            // check for remaining Tokens
            if t.1.is_empty() {
                Ok((t.0, &tokens[1..]))
            } else {
                Err(Error::RemainingTokens)
            }
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

    fn default_effect() -> Option<Self>;

    fn from_str(s: &str) -> Result<Self> {
        let tokens = s.parse();
        let default = Self::default_effect();
        match (tokens, default) {
            (Ok(tokens), _) => Self::from_tokens(tokens),
            (Err(Error::NoTokens), Some(default)) => Ok(default),
            (Err(err), _) => Err(err),
        }
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
                error!("RemainingTokens: {:?}", &ok.1);
                Err(Error::RemainingTokens)
            }
        })
    }
}

impl<T> ParseTokens for Vec<T>
where
    T: ParseTokens + Debug,
{
    fn default_effect() -> Option<Self> {
        T::default_effect().map(|t| vec![t])
    }

    fn parse_tokens(mut tokens: &[Tokens]) -> Result<(Self, &[Tokens])> {
        let mut v = Vec::new();
        while !tokens.is_empty() {
            let (param, t) = if tokens.iter().all(|t| matches!(t, Tokens::List(_))) {
                T::take_param(tokens)?
            } else {
                T::parse_tokens(tokens)?
            };
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
    fn default_effect() -> Option<Self> {
        T::default_effect().map(|t| Box::new(t))
    }

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
    fn from(mut value: [Tokens; N]) -> Self {
        if value.len() != 1 {
            Self::List(value.into())
        } else {
            // this is a mess
            let mut s = [Tokens::Token("".into())];
            value[..1].swap_with_slice(&mut s);
            let [s, ..] = s;
            s
        }
    }
}

impl<T: Into<Tokens>> From<Vec<T>> for Tokens {
    fn from(mut value: Vec<T>) -> Self {
        if value.len() != 1 {
            Self::List(value.into_iter().map(Into::into).collect())
        } else {
            value.swap_remove(0).into()
        }
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

        // group tokens by line
        let by_line = s.replace('(', "((").replace(')', "))").replace('\n', ")(");
        let by_line = format!("({by_line})");

        let mut stack = Vec::new();
        let mut token = String::new();
        let mut list = Vec::new();
        let mut bracket_level = 0;

        for c in by_line.chars() {
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
                    match list.len() {
                        2.. => _list.push(Tokens::List(list)),
                        1 => _list.push(list.pop().ok_or(Error::NoTokens)?),
                        _ => {}
                    }
                    list = _list;
                }
                '\n' => {
                    add_token(&mut list, token)?;
                    token = String::new();
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
