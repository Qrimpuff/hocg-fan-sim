#![allow(dead_code)]
#![allow(unused_imports)]

mod card_effects;
mod cards;
mod gameplay;
mod temp;

use std::{
    env,
    fmt::Display,
    iter::{self, Peekable},
    str::FromStr,
};

use card_effects::*;
use cards::*;
use gameplay::Game;
use temp::TEST_LIBRARY;

type Result<T> = std::result::Result<T, Error>;

const TEST_TEXT: &str = "for active_holo buff more_def 1 next_turn";

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let player_1 = Loadout {
        oshi: "Sora-Oshi".into(),
        main_deck: Vec::from_iter(
            iter::repeat("Sora-Debut".into())
                .take(10)
                .chain(iter::repeat("Sora-1".into()).take(10))
                .chain(iter::repeat("Sora-2".into()).take(5))
                .chain(iter::repeat("Support-1".into()).take(25)),
        ),
        cheer_deck: Vec::from_iter(iter::repeat("White-Cheer".into()).take(20)),
    };
    let player_2 = Loadout {
        oshi: "AZKi-Oshi".into(),
        main_deck: Vec::from_iter(
            iter::repeat("AZKi-Debut".into())
                .take(10)
                .chain(iter::repeat("AZKi-1".into()).take(10))
                .chain(iter::repeat("AZKi-2".into()).take(5))
                .chain(iter::repeat("Support-1".into()).take(25)),
        ),
        cheer_deck: Vec::from_iter(iter::repeat("Green-Cheer".into()).take(20)),
    };

    let mut game = Game::setup(TEST_LIBRARY.clone(), &player_1, &player_2);
    // dbg!(&game);
    game.start_game();
    // dbg!(&game);
}
