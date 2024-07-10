#![allow(dead_code)]
#![allow(unused_imports)]

mod ability;

use std::{env, fmt::Display, iter::Peekable, str::FromStr};

use ability::*;
use logos::{Lexer, Logos, Span};
use serde::{Deserialize, Serialize};

// type Error = (String, Span);
type Error = ();

type Result<T> = std::result::Result<T, Error>;

const TEST_TEXT: &str = "buff active_holo more_def $asd next_turn\nbuff active_holo more_def $asd next_turn";

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let buff = Action::Buff(
        Target::BuiltIn(BuiltIn::ActiveHoloMember),
        Buff::MoreDefence(Value::Number(20)),
        LifeTime::NextTurn,
    );
    dbg!(&buff);
    println!("{}", ability::to_string(&buff).unwrap());

    let action = ability::from_str::<Vec<Action>>(TEST_TEXT).unwrap();
    dbg!(&action);
    println!("{}", ability::to_string(&action).unwrap());
}
