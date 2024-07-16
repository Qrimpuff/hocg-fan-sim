use super::error::Error;
use std::collections::VecDeque;
use std::fmt::Display;
use std::str::FromStr;

pub trait ParseEffect {
    fn parse_effect<F: ParseTokens>(&self) -> Result<F, Error>;
}

impl ParseEffect for str {
    fn parse_effect<F: ParseTokens>(&self) -> Result<F, Error> {
        ParseTokens::from_str(self)
    }
}

pub trait ParseTokens: Sized {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self, Error>;

    fn from_str(s: &str) -> Result<Self, Error> {
        Self::from_tokens(s.parse()?)
    }

    fn from_tokens(tokens: Tokens) -> Result<Self, Error> {
        let mut tokens = match tokens {
            t @ Tokens::Token(_) => VecDeque::from([t]),
            Tokens::List(v) => v,
        };

        Self::parse_tokens(&mut tokens)

        // TODO check for remaining Tokens
    }

    fn take_param<T: ParseTokens>(tokens: &mut VecDeque<Tokens>) -> Result<T, Error> {
        let (ctx, clean) = Self::get_tokens_context(tokens)?;
        let param = T::parse_tokens(ctx);
        if clean {
            println!("clean param {:#?}", &tokens);
            Self::clean_list(tokens)?;
        }
        param
    }

    fn take_string(tokens: &mut VecDeque<Tokens>) -> Result<String, Error> {
        let t = tokens.pop_front().ok_or(Error::ExpectedToken)?;
        if let Tokens::Token(s) = t {
            return Ok(s);
        }
        Err(Error::ExpectedString)
    }

    fn return_string(tokens: &mut VecDeque<Tokens>, s: String) {
        tokens.push_front(Tokens::Token(s));
    }

    fn get_tokens_context(
        tokens: &mut VecDeque<Tokens>,
    ) -> Result<(&mut VecDeque<Tokens>, bool), Error> {
        println!("{:#?}", &tokens);
        let is_list = {
            let t = tokens.get_mut(0).ok_or(Error::ExpectedToken)?;
            println!("{:#?}", &t);
            matches!(t, Tokens::List(_))
        };
        println!("{:#?}", is_list);
        if is_list {
            let Tokens::List(v) = tokens.get_mut(0).ok_or(Error::ExpectedToken)? else {
                unreachable!()
            };
            Ok((v, true))
        } else {
            Ok((tokens, false))
        }
    }

    fn clean_list(tokens: &mut VecDeque<Tokens>) -> Result<(), Error> {
        let t = tokens.pop_front().ok_or(Error::ExpectedToken)?;
        println!("{:#?}", &t);
        if let Tokens::List(v) = t {
            assert_eq!(v.len(), 0, "list not empty, shouldn't clean");
        } else {
            panic!("removing something we shouldn't")
        }
        Ok(())
    }
}

impl<T> ParseTokens for Vec<T>
where
    T: ParseTokens,
{
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self, Error> {
        let mut v = Vec::new();
        while !tokens.is_empty() {
            v.push(T::take_param(tokens)?);
        }
        Ok(v)
    }
}

pub trait TakeParam<T> {
    fn take_param(&mut self) -> Result<T, Error>;
}
impl<T> TakeParam<T> for VecDeque<Tokens>
where
    T: ParseTokens,
{
    fn take_param(&mut self) -> Result<T, Error> {
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

impl Display for Tokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tokens::Token(t) => write!(f, "{t}"),
            Tokens::List(v) => {
                write!(
                    f,
                    "({})",
                    v.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
        }
    }
}

impl FromStr for Tokens {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        // FIXME check balanced bracket

        let mut stack = VecDeque::new();
        let mut token = String::new();
        let mut list = VecDeque::new();
        for c in s.chars() {
            match c {
                '(' => {
                    if !token.is_empty() {
                        list.push_back(Tokens::Token(token));
                    }
                    token = String::new();
                    stack.push_back(list);
                    list = VecDeque::new();
                }
                ')' => {
                    if !token.is_empty() {
                        list.push_back(Tokens::Token(token));
                    }
                    token = String::new();
                    let mut _list = stack
                        .pop_back()
                        .ok_or(Error::Message("missing bracket".into()))?;
                    if list.len() > 1 {
                        _list.push_back(Tokens::List(list));
                    } else {
                        _list.push_back(
                            list.pop_back()
                                .ok_or(Error::Message("error to be defined 2".into()))?,
                        );
                    }
                    list = _list;
                }
                c if c.is_whitespace() => {
                    if !token.is_empty() {
                        list.push_back(Tokens::Token(token));
                    }
                    token = String::new();
                }
                c => token.push(c),
            }
        }
        if !token.is_empty() {
            list.push_back(Tokens::Token(token));
        }

        println!("{:#?}", stack);

        if list.len() > 1 {
            Ok(Tokens::List(list))
        } else {
            Ok(list
                .pop_back()
                .ok_or(Error::Message("error to be defined 3".into()))?)
        }
    }
}
