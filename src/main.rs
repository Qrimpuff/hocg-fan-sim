#![allow(dead_code)]

mod card_effects;
mod cards;
mod events;
mod gameplay;
mod modifiers;
mod temp;

use std::{env, iter};

use card_effects::*;
use cards::*;
use gameplay::{Game, RandomPrompter};
use temp::test_library;

type Result<T> = std::result::Result<T, Error>;

const TEST_TEXT: &str = "for active_holo buff more_def 1 next_turn";

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let player_1 = Loadout {
        oshi: "Sora-Oshi".into(),
        main_deck: Vec::from_iter(
            iter::repeat("Sora-Debut".into())
                .take(20)
                .chain(iter::repeat("Sora-1".into()).take(10))
                .chain(iter::repeat("Sora-2".into()).take(5))
                .chain(iter::repeat("Support-1".into()).take(10))
                .chain(iter::repeat("Support-2".into()).take(5)),
        ),
        cheer_deck: Vec::from_iter(
            iter::repeat("White-Cheer".into())
                .take(15)
                .chain(iter::repeat("Yellow-Cheer".into()).take(5)),
        ),
    };
    let player_2 = Loadout {
        oshi: "AZKi-Oshi".into(),
        main_deck: Vec::from_iter(
            iter::repeat("AZKi-Debut".into())
                .take(20)
                .chain(iter::repeat("AZKi-1".into()).take(10))
                .chain(iter::repeat("AZKi-2".into()).take(5))
                .chain(iter::repeat("Support-1".into()).take(10))
                .chain(iter::repeat("Support-2".into()).take(5)),
        ),
        cheer_deck: Vec::from_iter(
            iter::repeat("Green-Cheer".into())
                .take(15)
                .chain(iter::repeat("Blue-Cheer".into()).take(5)),
        ),
    };

    let mut game = Game::setup(
        test_library().clone(),
        &player_1,
        &player_2,
        RandomPrompter::new(),
    );
    // println!("{:#?}", &game);
    game.start_game().unwrap();
    // println!("{:#?}", &game);

    while game.next_step().is_ok() {}
    println!("{:#?}", &game);
}
