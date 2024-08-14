use std::env;

use hocg_fan_sim::{
    card_effects::Condition,
    gameplay::{CardRef, Player, Step, Zone},
    modifiers::{DamageMarkers, LifeTime, Modifier, ModifierKind},
    prompters::BufferedPrompter,
};
use pretty_assertions::assert_eq;
use time::macros::format_description;
use tracing_subscriber::{fmt::time::LocalTime, EnvFilter};

use crate::{setup_test_game, GameStateBuilder, TestGameBoard};

#[test]
/// hSD01-001 - Tokino Sora (Oshi)
fn hsd01_001() {
    // // --------------- setup logs ---------------------
    // env::set_var("RUST_BACKTRACE", "1");
    // env::set_var("RUST_LOG", "DEBUG");

    // let file_appender = tracing_appender::rolling::daily("logs", "hocg-fan-sim.log");
    // let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    // tracing_subscriber::fmt()
    //     .with_timer(LocalTime::new(format_description!(
    //         "[year]-[month]-[day] [hour repr:24]:[minute]:[second].[subsecond digits:4]"
    //     )))
    //     .with_writer(non_blocking)
    //     .with_ansi(false)
    //     // enable thread id to be emitted
    //     .with_thread_ids(true)
    //     .with_env_filter(EnvFilter::from_default_env())
    //     .init();
    // // -------------- end setup logs -------------------

    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-003".into()),
        back_stage: vec!["hSD01-004".into()],
        life: vec!["hY01-001".into()],
        holo_power: vec!["hSD01-005".into(), "hSD01-005".into(), "hSD01-005".into()],
        ..Default::default()
    };
    let p2 = p1.clone();

    let state = GameStateBuilder::new()
        .with_active_player(Player::One)
        .with_active_step(Step::Cheer)
        .with_player_1(p1)
        .with_attachments(Player::One, Zone::CenterStage, 0, vec!["hY01-001".into()])
        .with_player_2(p2)
        .build();

    let p1_p = BufferedPrompter::new(&[
        // Replacement
        &[2],
        &[0],
        &[0],
        &[0],
        // So You're the Enemy?
        &[1],
        &[0],
        // done
        &[1],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // main step
    game.next_step().unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    // expected_state.game_outcome
    // expected_state.card_map
    // expected_state.player_1
    expected_state.player_1.holo_power = vec![];
    expected_state.player_1.archive = vec!["c_0711".into(), "c_0611".into(), "c_0511".into()];
    expected_state
        .player_1
        .attachments
        .insert("c_0831".into(), "c_0311".into());
    // expected_state.player_2
    expected_state.player_2.center_stage = Some("c_0312".into());
    expected_state.player_2.back_stage = vec!["c_0212".into()];
    // expected_state.active_player
    // expected_state.active_step
    expected_state.active_step = Step::Main;
    // expected_state.turn_number
    // expected_state.zone_modifiers
    expected_state
        .zone_modifiers
        .entry(Player::One)
        .or_default()
        .push((
            Zone::CenterStage,
            Modifier {
                id: "m_0002".into(),
                kind: ModifierKind::Conditional(
                    Condition::IsColorWhite,
                    Box::new(ModifierKind::MoreDamage(50)),
                ),
                life_time: LifeTime::ThisTurn,
            },
        ));
    // expected_state.card_modifiers
    expected_state
        .card_modifiers
        .entry("c_0111".into())
        .or_default()
        .push(Modifier {
            id: "m_0001".into(),
            kind: ModifierKind::PreventOshiSkill(0),
            life_time: LifeTime::ThisTurn,
        });
    expected_state
        .card_modifiers
        .entry("c_0111".into())
        .or_default()
        .push(Modifier {
            id: "m_0003".into(),
            kind: ModifierKind::PreventOshiSkill(1),
            life_time: LifeTime::ThisGame,
        });
    // expected_state.card_damage_markers
    // expected_state.event_span

    assert_eq!(expected_state, game.state);
}

#[test]
/// hSD01-002 - AZKi (Oshi)
fn hsd01_002() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-002".into()),
        center_stage: Some("hSD01-003".into()),
        back_stage: vec!["hSD01-009".into()],
        life: vec!["hY01-001".into()],
        holo_power: vec![
            "hSD01-005".into(),
            "hSD01-005".into(),
            "hSD01-005".into(),
            "hSD01-005".into(),
            "hSD01-005".into(),
            "hSD01-005".into(),
        ],
        archive: vec!["hY01-001".into(), "hY01-001".into(), "hY01-001".into()],
        ..Default::default()
    };
    let p2 = p1.clone();

    let state = GameStateBuilder::new()
        .with_active_player(Player::One)
        .with_active_step(Step::Cheer)
        .with_player_1(p1)
        .with_player_2(p2)
        .build();

    let p1_p = BufferedPrompter::new(&[
        // In My Left Hand, a Map
        &[0],
        &[0],
        &[5],
        // In My Right Hand, a Mic
        &[0],
        &[0, 1, 2],
        &[0],
        // done
        &[0],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // main step
    game.next_step().unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    // expected_state.game_outcome
    // expected_state.card_map
    // expected_state.player_1
    expected_state.player_1.collab = Some("c_0311".into());
    expected_state.player_1.back_stage = vec![];
    expected_state.player_1.holo_power = vec![];
    expected_state.player_1.archive = vec![
        "c_0a11".into(),
        "c_0911".into(),
        "c_0811".into(),
        "c_0711".into(),
        "c_0611".into(),
        "c_0511".into(),
    ];
    expected_state.player_1.attachments.extend([
        (CardRef::from("c_0d11"), "c_0311".into()),
        ("c_0b11".into(), "c_0311".into()),
        ("c_0c11".into(), "c_0311".into()),
    ]);
    // expected_state.player_2
    // expected_state.active_player
    // expected_state.active_step
    expected_state.active_step = Step::Main;
    // expected_state.turn_number
    // expected_state.zone_modifiers
    expected_state
        .zone_modifiers
        .entry(Player::One)
        .or_default()
        .extend([(
            Zone::All,
            Modifier {
                id: "m_0001".into(),
                kind: ModifierKind::PreventCollab,
                life_time: LifeTime::ThisTurn,
            },
        )]);
    // expected_state.card_modifiers
    expected_state
        .card_modifiers
        .entry("c_0111".into())
        .or_default()
        .extend([
            Modifier {
                id: "m_0003".into(),
                kind: ModifierKind::PreventOshiSkill(0),
                life_time: LifeTime::ThisTurn,
            },
            Modifier {
                id: "m_0004".into(),
                kind: ModifierKind::PreventOshiSkill(1),
                life_time: LifeTime::ThisGame,
            },
        ]);
    // expected_state.card_damage_markers
    // expected_state.event_span

    assert_eq!(expected_state, game.state);
}

#[test]
/// hSD01-003 - Tokino Sora (Debut)
fn hsd01_003() {
    // no need for testing: vanilla card
}

#[test]
/// hSD01-004 - Tokino Sora (Debut)
fn hsd01_004() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-004".into()),
        back_stage: vec!["hSD01-004".into()],
        life: vec!["hY01-001".into()],
        ..Default::default()
    };
    let p2 = p1.clone();

    let state = GameStateBuilder::new()
        .with_active_player(Player::One)
        .with_active_step(Step::Cheer)
        .with_player_1(p1)
        .with_attachments(Player::One, Zone::CenterStage, 0, vec!["hY01-001".into()])
        .with_player_2(p2)
        .build();

    let p1_p = BufferedPrompter::new(&[
        // Let's Dance!
        &[0],
        // done
        &[0],
        // performance step
        &[0],
        // done
        &[0],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // main step
    game.next_step().unwrap();
    // performance step
    game.next_step().unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Performance;
    expected_state.player_1.collab = Some("c_0311".into());
    expected_state.player_1.back_stage = vec![];
    expected_state.player_1.holo_power = vec![];
    expected_state
        .zone_modifiers
        .entry(Player::One)
        .or_default()
        .extend([
            (
                Zone::All,
                Modifier {
                    id: "m_0001".into(),
                    kind: ModifierKind::PreventCollab,
                    life_time: LifeTime::ThisTurn,
                },
            ),
            (
                Zone::CenterStage,
                Modifier {
                    id: "m_0002".into(),
                    kind: ModifierKind::MoreDamage(20),
                    life_time: LifeTime::ThisTurn,
                },
            ),
        ]);
    expected_state
        .card_modifiers
        .entry("c_0211".into())
        .or_default()
        .extend([Modifier {
            id: "m_0003".into(),
            kind: ModifierKind::PreventAllArts,
            life_time: LifeTime::ThisTurn,
        }]);
    expected_state
        .card_damage_markers
        .insert("c_0212".into(), DamageMarkers::from_hp(40));
    // expected_state.card_damage_markers

    assert_eq!(expected_state, game.state);
}

#[test]
/// hSD01-005 - Tokino Sora (First)
fn hsd01_005() {
    // no need for testing: vanilla card
}

#[test]
/// hSD01-006 - Tokino Sora (First)
fn hsd01_006_without_azki() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-006".into()),
        life: vec!["hY01-001".into()],
        ..Default::default()
    };
    let p2 = p1.clone();

    let state = GameStateBuilder::new()
        .with_active_player(Player::One)
        .with_active_step(Step::Main)
        .with_player_1(p1)
        .with_attachments(
            Player::One,
            Zone::CenterStage,
            0,
            vec!["hY02-001".into(), "hY01-001".into(), "hY01-001".into()],
        )
        .with_player_2(p2)
        .build();

    let p1_p = BufferedPrompter::new(&[
        // SorAZ Sympathy
        &[1],
        // done
        &[0],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    game.next_step().unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Performance;
    expected_state
        .card_modifiers
        .entry("c_0211".into())
        .or_default()
        .extend([Modifier {
            id: "m_0001".into(),
            kind: ModifierKind::PreventAllArts,
            life_time: LifeTime::ThisTurn,
        }]);
    expected_state
        .card_damage_markers
        .insert("c_0212".into(), DamageMarkers::from_hp(60));
    // expected_state.card_damage_markers

    assert_eq!(expected_state, game.state);
}
#[test]
/// hSD01-006 - Tokino Sora (First)
fn hsd01_006_with_azki() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-006".into()),
        back_stage: vec!["hSD01-011".into()],
        life: vec!["hY01-001".into()],
        ..Default::default()
    };
    let p2 = p1.clone();

    let state = GameStateBuilder::new()
        .with_active_player(Player::One)
        .with_active_step(Step::Main)
        .with_player_1(p1)
        .with_attachments(
            Player::One,
            Zone::CenterStage,
            0,
            vec!["hY02-001".into(), "hY01-001".into(), "hY01-001".into()],
        )
        .with_player_2(p2)
        .build();

    let p1_p = BufferedPrompter::new(&[
        // SorAZ Sympathy
        &[1],
        // done
        &[0],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    game.next_step().unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Performance;
    expected_state
        .card_modifiers
        .entry("c_0211".into())
        .or_default()
        .extend([Modifier {
            id: "m_0002".into(),
            kind: ModifierKind::PreventAllArts,
            life_time: LifeTime::ThisTurn,
        }]);
    expected_state
        .card_damage_markers
        .insert("c_0212".into(), DamageMarkers::from_hp(110));
    // expected_state.card_damage_markers

    assert_eq!(expected_state, game.state);
}

#[test]
/// hSD01-007 - IRyS (Debut)
fn hsd01_007() {
    // TODO testing
}

#[test]
/// hSD01-008 - AZKi (Debut)
fn hsd01_008() {
    // no need for testing: vanilla card
}

#[test]
/// hSD01-009 - AZKi (Debut)
fn hsd01_009() {
    // TODO testing
}

#[test]
/// hSD01-010 - AZKi (First)
fn hsd01_010() {
    // no need for testing: vanilla card
}

#[test]
/// hSD01-011 - AZKi (Second)
fn hsd01_011() {
    // TODO testing
}

#[test]
/// hSD01-012 - Airani Iofifteen (Debut)
fn hsd01_012() {
    // TODO testing
}

#[test]
/// hSD01-013 - SorAZ (First)
fn hsd01_013() {
    // TODO testing
}

#[test]
/// hSD01-014 - Amane Kanata (Spot)
fn hsd01_014() {
    // no need for testing: vanilla card
}

#[test]
/// hSD01-015 - Hakui Koyori (Spot)
fn hsd01_015() {
    // TODO testing
}

#[test]
/// hSD01-016 - Harusaki Nodoka (Staff)
fn hsd01_016() {
    // TODO testing
}

#[test]
/// hSD01-017 - Mane-chan (Staff)
fn hsd01_017() {
    // TODO testing
}

#[test]
/// hSD01-018 - Second PC (Item)
fn hsd01_018() {
    // TODO testing
}

#[test]
/// hSD01-019 - Amazing PC (Item)
fn hsd01_019() {
    // TODO testing
}

#[test]
/// hSD01-020 - hololive Fan Circle (Event)
fn hsd01_020() {
    // TODO testing
}

#[test]
/// hSD01-021 - First Gravity (Event)
fn hsd01_021() {
    // TODO testing
}
