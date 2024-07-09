#![allow(dead_code)]
#![allow(unused_imports)]

use std::{fmt::Display, iter::Peekable, str::FromStr};

use logos::{Lexer, Logos, Span};

// type Error = (String, Span);
type Error = ();

type Result<T> = std::result::Result<T, Error>;

const TEST_TEXT: &str = "buff active_holo more_def 134 next_turn";

fn main() {
    let buff = Action::Buff(
        Target::BuiltIn(BuiltIn::ActiveHoloMember),
        Buff::MoreDefence(Value::Number(20)),
        LifeTime::NextTurn,
    );
    dbg!(&buff);
    println!("{buff}");

    println!("start lexing...");
    let mut lexer = Token::lexer(TEST_TEXT).peekable();
    let act = Action::deserialize(&mut lexer);
    dbg!(act);
}

trait Deserialize {
    fn deserialize(lexer: &mut Peekable<Lexer<Token>>) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug)]
enum BuiltIn {
    ActiveHoloMember,
}
impl Deserialize for BuiltIn {
    fn deserialize(lexer: &mut Peekable<Lexer<Token>>) -> Result<Self> {
        // TODO try to generalize, because there will be many of them
        // remove spaces
        loop {
            if let Some(token) = lexer.next() {
                let token = token?;
                match token {
                    Token::Space => continue,
                    Token::Ident(built_in) => match built_in.as_str() {
                        "active_holo" => return Ok(Self::ActiveHoloMember),
                        _ => todo!(),
                    },
                    _ => return Err(()),
                }
            }
        }
    }
}

impl Display for BuiltIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuiltIn::ActiveHoloMember => write!(f, "active_holo"),
        }
    }
}

#[derive(Debug)]
enum Target {
    BuiltIn(BuiltIn),
    Var(String),
}
impl Deserialize for Target {
    fn deserialize(lexer: &mut Peekable<Lexer<Token>>) -> Result<Self> {
        // remove spaces
        loop {
            if let Some(token) = lexer.peek() {
                let token = token.as_ref().map_err(|e| *e)?;
                match token {
                    Token::Space => {
                        lexer.next();
                        continue;
                    }
                    Token::Ident(_) => return Ok(Self::BuiltIn(BuiltIn::deserialize(lexer)?)),
                    _ => return Err(()),
                }
            }
        }
    }
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::BuiltIn(b) => write!(f, "{b}"),
            Target::Var(s) => write!(f, "${s}"),
        }
    }
}

#[derive(Debug)]
enum Action {
    Buff(Target, Buff, LifeTime),
    Debuff(Target, Debuff, LifeTime),
    Heal(Target, Value),
    Let(String, Value),
    When(Condition, Box<Action>),
    Unknown,
}

impl Deserialize for Action {
    fn deserialize(lexer: &mut Peekable<Lexer<Token>>) -> Result<Self> {
        // remove spaces
        loop {
            if let Some(token) = lexer.next() {
                let token = token?;
                match token {
                    Token::Space => continue,
                    Token::Ident(action) => match action.as_str() {
                        "buff" => {
                            return Ok(Self::Buff(
                                Target::deserialize(lexer)?,
                                Buff::deserialize(lexer)?,
                                LifeTime::deserialize(lexer)?,
                            ))
                        }
                        _ => todo!(),
                    },
                    _ => return Err(()),
                }
            }
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Buff(t, b, l) => write!(f, "buff {t} {b} {l}"),
            Action::Debuff(_, _, _) => todo!(),
            Action::Heal(_, _) => todo!(),
            Action::Let(_, _) => todo!(),
            Action::When(_, _) => todo!(),
            Action::Unknown => todo!(),
        }
    }
}

#[derive(Debug)]
enum Value {
    Get(Target, Property),
    Number(i32),
    Var(String),
}
impl Deserialize for Value {
    fn deserialize(lexer: &mut Peekable<Lexer<Token>>) -> Result<Self> {
        // remove spaces
        loop {
            if let Some(token) = lexer.next() {
                let token = token?;
                match token {
                    Token::Space => continue,
                    Token::Number(num) => return Ok(Self::Number(num)),
                    _ => return Err(()),
                }
            }
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Get(_, _) => todo!(),
            Value::Number(n) => write!(f, "{n}"),
            Value::Var(_) => todo!(),
        }
    }
}

#[derive(Debug)]
enum Condition {
    Equals(Value, Value),
    Has(Target, Tag),
    NotEquals(Value, Value),
}

#[derive(Debug)]
enum Tag {
    // colors
    ColorWhite,
    ColorGreen,
    ColorBlue,
    ColorRed,
    ColorPurple,
    ColorYellow,
    // stages
    StageDebut,
    StageFirst,
    StageSecond,
    //abilities
}

#[derive(Debug)]
enum Property {
    HealtPoint,
    RetreatCost,
}

#[derive(Debug)]
enum Buff {
    MoreDefence(Value),
}
impl Deserialize for Buff {
    fn deserialize(lexer: &mut Peekable<Lexer<Token>>) -> Result<Self> {
        // remove spaces
        loop {
            if let Some(token) = lexer.next() {
                let token = token?;
                match token {
                    Token::Space => continue,
                    Token::Ident(buff) => match buff.as_str() {
                        "more_def" => return Ok(Self::MoreDefence(Value::deserialize(lexer)?)),
                        _ => todo!(),
                    },
                    _ => return Err(()),
                }
            }
        }
    }
}

impl Display for Buff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Buff::MoreDefence(v) => write!(f, "more_def {v}"),
        }
    }
}

#[derive(Debug)]
enum Debuff {}

#[derive(Debug)]
enum LifeTime {
    ThisTurn,
    NextTurn,
    Limitless,
}
impl Deserialize for LifeTime {
    fn deserialize(lexer: &mut Peekable<Lexer<Token>>) -> Result<Self> {
        // remove spaces
        loop {
            if let Some(token) = lexer.next() {
                let token = token?;
                match token {
                    Token::Space => continue,
                    Token::Ident(life_time) => match life_time.as_str() {
                        "this_turn" => return Ok(Self::ThisTurn),
                        "next_turn" => return Ok(Self::NextTurn),
                        "_" => return Ok(Self::Limitless),
                        _ => todo!(),
                    },
                    _ => return Err(()),
                }
            }
        }
    }
}

impl Display for LifeTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifeTime::ThisTurn => write!(f, "this_turn"),
            LifeTime::NextTurn => write!(f, "next_turn"),
            LifeTime::Limitless => write!(f, "_"),
        }
    }
}

#[derive(Debug, Logos)]
enum Token {
    #[regex(r"[ \t\r\n\f]+")]
    Space,

    #[token("(")]
    ParanOpen,

    #[token(")")]
    ParanClose,

    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i32>().unwrap())]
    Number(i32),

    #[regex(r"[a-z_]+", |lex| lex.slice().to_owned())]
    Ident(String),

    #[token("$")]
    VarStart,
}
