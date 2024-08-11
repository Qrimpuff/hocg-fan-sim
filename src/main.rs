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
use time::macros::format_description;
use tracing::info;
use tracing_subscriber::{fmt::time::LocalTime, EnvFilter};

type Result<T> = std::result::Result<T, Error>;

const TEST_TEXT: &str = "for active_holo buff more_def 1 next_turn";

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", "DEBUG");

    // setup logs
    let file_appender = tracing_appender::rolling::daily("logs", "hocg-fan-sim.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_timer(LocalTime::new(format_description!(
            "[year]-[month]-[day] [hour repr:24]:[minute]:[second].[subsecond digits:4]"
        )))
        .with_writer(non_blocking)
        .with_ansi(false)
        // enable thread id to be emitted
        .with_thread_ids(true)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    info!("\n\n\n\n\n\n\n-- Hololive OCG - Fan Simulator is running --");

    let _cond = r"
        let $center_mem = ((from_zone center_stage) where is_member)
        if $center_mem any is_named_tokino_sora (
            draw 1
        )
        if $center_mem any is_named_azki (
            let $cheer = from_zone_top 1 cheer_deck
            reveal $cheer
            attach_cards $cheer $center_mem
        )
    ";

    // dbg!(_cond.parse_effect::<Vec<Action>>().expect("IN MAIN"));

    let main_deck_hsd01 = Vec::from_iter(
        None.into_iter()
            .chain(iter::repeat("hSD01-003".into()).take(4))
            .chain(iter::repeat("hSD01-004".into()).take(3))
            .chain(iter::repeat("hSD01-005".into()).take(3))
            .chain(iter::repeat("hSD01-006".into()).take(2))
            .chain(iter::repeat("hSD01-007".into()).take(2))
            .chain(iter::repeat("hSD01-008".into()).take(4))
            .chain(iter::repeat("hSD01-009".into()).take(3))
            .chain(iter::repeat("hSD01-010".into()).take(3))
            .chain(iter::repeat("hSD01-011".into()).take(2))
            .chain(iter::repeat("hSD01-012".into()).take(2))
            .chain(iter::repeat("hSD01-013".into()).take(2))
            .chain(iter::repeat("hSD01-014".into()).take(2))
            .chain(iter::repeat("hSD01-015".into()).take(2))
            .chain(iter::repeat("hSD01-016".into()).take(3))
            .chain(iter::repeat("hSD01-017".into()).take(3))
            .chain(iter::repeat("hSD01-018".into()).take(3))
            .chain(iter::repeat("hSD01-019".into()).take(3))
            .chain(iter::repeat("hSD01-020".into()).take(2))
            .chain(iter::repeat("hSD01-021".into()).take(2)),
    );
    let cheer_deck_hsd01 = Vec::from_iter(
        None.into_iter()
            .chain(iter::repeat("hY01-001".into()).take(10))
            .chain(iter::repeat("hY02-001".into()).take(10)),
    );

    let player_1 = Loadout {
        oshi: "hSD01-001".into(), // Tokino Sora
        main_deck: main_deck_hsd01.clone(),
        cheer_deck: cheer_deck_hsd01.clone(),
    };
    let player_2 = Loadout {
        oshi: "hSD01-002".into(), // AZKi
        main_deck: main_deck_hsd01,
        cheer_deck: cheer_deck_hsd01,
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
