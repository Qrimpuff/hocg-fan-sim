#![allow(dead_code)]
#![allow(unused_imports)]

mod card_effects;

use std::{env, fmt::Display, iter::Peekable, str::FromStr};

use card_effects::*;

type Result<T> = std::result::Result<T, Error>;

const TEST_TEXT: &str = "for active_holo buff more_def 1 next_turn";

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    // let buff = Action::For(
    //     Target::BuiltIn(BuiltIn::ActiveHoloMember),
    //     Box::new(Action::Buff(
    //         Buff::MoreDefence(Value::For(
    //             Target::BuiltIn(BuiltIn::CurrentCard),
    //             Box::new(Value::Get(Property::HealtPoint)),
    //         )),
    //         LifeTime::NextTurn,
    //     )),
    // );
    // let buff2 = Action::Heal(Value::Get(Property::HealtPoint));

    // let s = "for active_holo buff more_def for self get hp next_turn";
    let s = "let $1 100 let $var for self get r_cost when eq (for self get hp) $1 for active_holo (buff more_def $var next_turn)";
    let a = Vec::<Action>::from_str(s);
    // let a = Action::from_tokens(action.unwrap());
    dbg!(a);
}
