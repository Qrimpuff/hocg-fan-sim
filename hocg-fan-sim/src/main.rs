#![allow(dead_code)]

use std::{env, iter};

use hocg_fan_sim::client::DefaultEventHandler;
use hocg_fan_sim::gameplay::Game;
use hocg_fan_sim::prompters::RandomPrompter;
use hocg_fan_sim::temp::test_library;
use hocg_fan_sim::{cards::*, client::Client};
use time::macros::format_description;
use tracing::info;
use tracing_subscriber::{fmt::time::LocalTime, EnvFilter};

const TEST_TEXT: &str = "for active_holo buff more_def 1 next_turn";

#[tokio::main]
async fn main() {
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
    info!("\n\n\n\n\n\n\n-- hololive OCG - Fan Simulator is running --");

    let _cond = r"
        let $mem = select_one from stage is_member and has_cheers
        let $cheer = select_one attached $mem is_cheer
        send_to archive $cheer
        let $cond = ((is_level_first or is_level_second) and not is_attribute_buzz) 
        let $choice = select_one from main_deck $cond
        reveal $choice
        send_to hand $choice
        shuffle main_deck
    ";

    // let c = dbg!(_cond.parse_effect::<Vec<Action>>().expect("IN MAIN"));
    let toml = toml::to_string(
        &test_library()
            .lookup_card(&"hSD01-002".to_string())
            .unwrap(),
    )
    .unwrap();
    println!("{toml}");

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

    let p1_channel_1 = async_channel::bounded(10);
    let p1_channel_2 = async_channel::bounded(10);
    let p2_channel_1 = async_channel::bounded(10);
    let p2_channel_2 = async_channel::bounded(10);

    let mut game = Game::setup(
        &player_1,
        &player_2,
        (p1_channel_1.0, p1_channel_2.1),
        (p2_channel_1.0, p2_channel_2.1),
    );

    // Player 1
    let p1_client = Client::new(
        (p1_channel_2.0, p1_channel_1.1),
        DefaultEventHandler::new(),
        RandomPrompter::new(),
    );
    tokio::spawn(p1_client.receive_requests());

    // Player 2
    let p2_client = Client::new(
        (p2_channel_2.0, p2_channel_1.1),
        DefaultEventHandler::new(),
        RandomPrompter::new(),
    );
    tokio::spawn(p2_client.receive_requests());

    // println!("{:#?}", &game);
    game.start_game().await.unwrap();
    // println!("{:#?}", &game);

    while game.next_step().await.is_ok() {}
    println!("{:#?}", &game);
}
