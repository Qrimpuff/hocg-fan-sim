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

#[tokio::test]
/// hSD01-001 - Tokino Sora (Oshi)
async fn hsd01_001() {
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
        back_stage: ["hSD01-004".into()].into(),
        life: ["hY01-001".into()].into(),
        holo_power: ["hSD01-005".into(), "hSD01-005".into(), "hSD01-005".into()].into(),
        ..Default::default()
    };
    let p2 = p1.clone();

    let state = GameStateBuilder::new()
        .with_active_player(Player::One)
        .with_active_step(Step::Cheer)
        .with_player_1(p1)
        .with_attachments(
            Player::One,
            Zone::CenterStage,
            0,
            ["hY01-001".into()].into(),
        )
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
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    // expected_state.game_outcome
    // expected_state.card_map
    // expected_state.player_1
    expected_state.player_1.holo_power = [].into();
    expected_state.player_1.archive = ["c_0711".into(), "c_0611".into(), "c_0511".into()].into();
    expected_state
        .player_1
        .attachments
        .insert("c_0831".into(), "c_0311".into());
    // expected_state.player_2
    expected_state.player_2.center_stage = Some("c_0312".into());
    expected_state.player_2.back_stage = ["c_0212".into()].into();
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

#[tokio::test]
/// hSD01-002 - AZKi (Oshi)
async fn hsd01_002() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-002".into()),
        center_stage: Some("hSD01-003".into()),
        back_stage: ["hSD01-009".into()].into(),
        life: ["hY01-001".into()].into(),
        holo_power: [
            "hSD01-005".into(),
            "hSD01-005".into(),
            "hSD01-005".into(),
            "hSD01-005".into(),
            "hSD01-005".into(),
            "hSD01-005".into(),
        ]
        .into(),
        archive: ["hY01-001".into(), "hY01-001".into(), "hY01-001".into()].into(),
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
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    // expected_state.game_outcome
    // expected_state.card_map
    // expected_state.player_1
    expected_state.player_1.collab = Some("c_0311".into());
    expected_state.player_1.back_stage = [].into();
    expected_state.player_1.holo_power = [].into();
    expected_state.player_1.archive = [
        "c_0a11".into(),
        "c_0911".into(),
        "c_0811".into(),
        "c_0711".into(),
        "c_0611".into(),
        "c_0511".into(),
    ]
    .into();
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

#[tokio::test]
/// hSD01-002 - AZKi (Oshi)
async fn hsd01_002_not_enough_holo_power() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-002".into()),
        center_stage: Some("hSD01-003".into()),
        back_stage: ["hSD01-009".into()].into(),
        life: ["hY01-001".into()].into(),
        holo_power: [].into(),
        archive: ["hY01-001".into(), "hY01-001".into(), "hY01-001".into()].into(),
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
        // Can't use, In My Left Hand, a Map
        &[0],
        // done
        &[0],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // main step
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    // expected_state.game_outcome
    // expected_state.card_map
    // expected_state.player_1
    expected_state.player_1.collab = Some("c_0311".into());
    expected_state.player_1.back_stage = [].into();
    expected_state.player_1.holo_power = [].into();
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
    // expected_state.card_damage_markers
    // expected_state.event_span

    assert_eq!(expected_state, game.state);
}

#[tokio::test]
/// hSD01-003 - Tokino Sora (Debut)
async fn hsd01_003() {
    // no need for testing: vanilla card
}

#[tokio::test]
/// hSD01-004 - Tokino Sora (Debut)
async fn hsd01_004() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-004".into()),
        back_stage: ["hSD01-004".into()].into(),
        life: ["hY01-001".into()].into(),
        ..Default::default()
    };
    let p2 = p1.clone();

    let state = GameStateBuilder::new()
        .with_active_player(Player::One)
        .with_active_step(Step::Cheer)
        .with_player_1(p1)
        .with_attachments(
            Player::One,
            Zone::CenterStage,
            0,
            ["hY01-001".into()].into(),
        )
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
    game.next_step().await.unwrap();
    // performance step
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Performance;
    expected_state.player_1.collab = Some("c_0311".into());
    expected_state.player_1.back_stage = [].into();
    expected_state.player_1.holo_power = [].into();
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

#[tokio::test]
/// hSD01-005 - Tokino Sora (First)
async fn hsd01_005() {
    // no need for testing: vanilla card
}

#[tokio::test]
/// hSD01-006 - Tokino Sora (First)
async fn hsd01_006_without_azki() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-006".into()),
        life: ["hY01-001".into()].into(),
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
            ["hY02-001".into(), "hY01-001".into(), "hY01-001".into()].into(),
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
    game.next_step().await.unwrap();

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
#[tokio::test]
/// hSD01-006 - Tokino Sora (First)
async fn hsd01_006_with_azki() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-006".into()),
        back_stage: ["hSD01-011".into()].into(),
        life: ["hY01-001".into()].into(),
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
            ["hY02-001".into(), "hY01-001".into(), "hY01-001".into()].into(),
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
    game.next_step().await.unwrap();

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

#[tokio::test]
/// hSD01-007 - IRyS (Debut)
async fn hsd01_007() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-006".into()),
        back_stage: ["hSD01-007".into()].into(),
        life: ["hY01-001".into()].into(),
        holo_power: ["hSD01-005".into(), "hSD01-006".into(), "hSD01-007".into()].into(),
        hand: ["hSD01-008".into(), "hSD01-009".into(), "hSD01-010".into()].into(),
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
        // HOPE
        &[2],
        &[1],
        &[2],
        // done
        &[4],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Main;
    expected_state.player_1.collab = Some("c_0311".into());
    expected_state.player_1.back_stage = [].into();
    expected_state.player_1.holo_power = ["c_0a11".into(), "c_0511".into(), "c_0711".into()].into();
    expected_state.player_1.hand = ["c_0811".into(), "c_0911".into(), "c_0611".into()].into();
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

    assert_eq!(expected_state, game.state);
}

#[tokio::test]
/// hSD01-008 - AZKi (Debut)
async fn hsd01_008() {
    // no need for testing: vanilla card
}

#[tokio::test]
/// hSD01-009 - AZKi (Debut)
async fn hsd01_009() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-002".into()),
        center_stage: Some("hSD01-006".into()),
        back_stage: ["hSD01-009".into()].into(),
        life: ["hY01-001".into()].into(),
        holo_power: ["hSD01-005".into(), "hSD01-006".into(), "hSD01-007".into()].into(),
        cheer_deck: ["hY02-001".into()].into(),
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
        // Expanding Map
        &[0],
        &[0],
        &[0],
        &[0],
        // done
        &[0],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Main;
    expected_state.player_1.holo_power = [].into();
    expected_state.player_1.archive = ["c_0811".into(), "c_0711".into(), "c_0611".into()].into();
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
    expected_state
        .card_modifiers
        .entry("c_0111".into())
        .or_default()
        .push(Modifier {
            id: "m_0003".into(),
            kind: ModifierKind::PreventOshiSkill(0),
            life_time: LifeTime::ThisTurn,
        });

    assert_eq!(expected_state, game.state);
}

#[tokio::test]
/// hSD01-010 - AZKi (First)
async fn hsd01_010() {
    // no need for testing: vanilla card
}

#[tokio::test]
/// hSD01-011 - AZKi (Second)
async fn hsd01_011_art_1() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-011".into()),
        back_stage: ["hSD01-006".into()].into(),
        life: ["hY01-001".into()].into(),
        cheer_deck: ["hY02-001".into()].into(),
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
            ["hY02-001".into(), "hY01-001".into(), "hY01-001".into()].into(),
        )
        .with_player_2(p2)
        .build();

    let p1_p = BufferedPrompter::new(&[
        // SorAZ Gravity
        &[0],
        &[1],
        // done
        &[0],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Performance;
    expected_state.player_1.cheer_deck = [].into();
    expected_state
        .player_1
        .attachments
        .extend([(CardRef::from("c_0511"), "c_0311".into())]);
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

#[tokio::test]
/// hSD01-011 - AZKi (Second)
async fn hsd01_011_art_2() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-002".into()),
        center_stage: Some("hSD01-011".into()),
        back_stage: ["hSD01-006".into()].into(),
        holo_power: ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
        life: ["hY01-001".into()].into(),
        ..Default::default()
    };
    let mut p2 = p1.clone();
    p2.center_stage = Some("hSD01-006".into());

    let state = GameStateBuilder::new()
        .with_active_player(Player::One)
        .with_active_step(Step::Main)
        .with_player_1(p1)
        .with_attachments(
            Player::One,
            Zone::CenterStage,
            0,
            ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
        )
        .with_zone_modifiers([(Player::One, vec![])].into())
        .with_player_2(p2)
        .build();

    let p1_p = BufferedPrompter::new(&[
        // Destiny Song
        &[1],
        &[0],
        &[0],
        // done
        &[0],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    let _ = game.next_step().await;

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Performance;
    expected_state.player_1.holo_power = [].into();
    expected_state.player_1.archive = ["c_0711".into(), "c_0611".into(), "c_0511".into()].into();
    expected_state
        .card_modifiers
        .entry("c_0111".into())
        .or_default()
        .extend([Modifier {
            id: "m_0002".into(),
            kind: ModifierKind::PreventOshiSkill(0),
            life_time: LifeTime::ThisTurn,
        }]);
    expected_state
        .card_modifiers
        .entry("c_0211".into())
        .or_default()
        .extend([Modifier {
            id: "m_0005".into(),
            kind: ModifierKind::PreventAllArts,
            life_time: LifeTime::ThisTurn,
        }]);
    expected_state
        .card_damage_markers
        .insert("c_0212".into(), DamageMarkers::from_hp(200));
    // expected_state.card_damage_markers

    assert_eq!(expected_state, game.state);
}

#[tokio::test]
/// hSD01-012 - Airani Iofifteen (Debut)
async fn hsd01_012() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-006".into()),
        back_stage: ["hSD01-012".into()].into(),
        life: ["hY01-001".into()].into(),
        archive: ["hY01-001".into()].into(),
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
        // Let's Draw Together!
        &[0],
        &[0],
        // done
        &[0],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Main;
    expected_state.player_1.collab = Some("c_0311".into());
    expected_state.player_1.back_stage = [].into();
    expected_state.player_1.archive = [].into();
    expected_state
        .player_1
        .attachments
        .extend([(CardRef::from("c_0511"), "c_0211".into())]);
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

    assert_eq!(expected_state, game.state);
}

#[tokio::test]
/// hSD01-013 - SorAZ (First)
async fn hsd01_013_odd() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-002".into()),
        center_stage: Some("hSD01-013".into()),
        back_stage: ["hSD01-006".into()].into(),
        holo_power: ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
        life: ["hY01-001".into()].into(),
        main_deck: ["hSD01-006".into()].into(),
        cheer_deck: ["hY01-001".into()].into(),
        ..Default::default()
    };
    let mut p2 = p1.clone();
    p2.center_stage = Some("hSD01-006".into());

    let state = GameStateBuilder::new()
        .with_active_player(Player::One)
        .with_active_step(Step::Main)
        .with_player_1(p1)
        .with_attachments(
            Player::One,
            Zone::CenterStage,
            0,
            ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
        )
        .with_zone_modifiers([(Player::One, vec![])].into())
        .with_player_2(p2)
        .build();

    let p1_p = BufferedPrompter::new(&[
        // The Future We Want to Overcome
        &[0],
        &[0],
        &[0],
        // done
        &[0],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    let _ = game.next_step().await;

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Performance;
    expected_state.player_1.cheer_deck = [].into();
    expected_state.player_1.holo_power = [].into();
    expected_state.player_1.archive = ["c_0911".into(), "c_0811".into(), "c_0711".into()].into();
    expected_state
        .player_1
        .attachments
        .extend([(CardRef::from("c_0611"), "c_0311".into())]);
    expected_state
        .card_modifiers
        .entry("c_0111".into())
        .or_default()
        .extend([Modifier {
            id: "m_0002".into(),
            kind: ModifierKind::PreventOshiSkill(0),
            life_time: LifeTime::ThisTurn,
        }]);
    expected_state
        .card_modifiers
        .entry("c_0311".into())
        .or_default()
        .extend([Modifier {
            id: "m_0003".into(),
            kind: ModifierKind::PreventAllArts,
            life_time: LifeTime::ThisTurn,
        }]);
    expected_state
        .card_damage_markers
        .insert("c_0312".into(), DamageMarkers::from_hp(50));
    // expected_state.card_damage_markers

    assert_eq!(expected_state, game.state);
}

#[tokio::test]
/// hSD01-013 - SorAZ (First)
async fn hsd01_013_even() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-002".into()),
        center_stage: Some("hSD01-013".into()),
        back_stage: ["hSD01-006".into()].into(),
        holo_power: ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
        life: ["hY01-001".into()].into(),
        main_deck: ["hSD01-006".into()].into(),
        cheer_deck: ["hY01-001".into()].into(),
        ..Default::default()
    };
    let mut p2 = p1.clone();
    p2.center_stage = Some("hSD01-006".into());

    let state = GameStateBuilder::new()
        .with_active_player(Player::One)
        .with_active_step(Step::Main)
        .with_player_1(p1)
        .with_attachments(
            Player::One,
            Zone::CenterStage,
            0,
            ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
        )
        .with_zone_modifiers([(Player::One, vec![])].into())
        .with_player_2(p2)
        .build();

    let p1_p = BufferedPrompter::new(&[
        // The Future We Want to Overcome
        &[0],
        &[0],
        &[1],
        // done
        &[0],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    let _ = game.next_step().await;

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Performance;
    expected_state.player_1.main_deck = [].into();
    expected_state.player_1.holo_power = [].into();
    expected_state.player_1.archive = ["c_0911".into(), "c_0811".into(), "c_0711".into()].into();
    expected_state.player_1.hand = ["c_0211".into()].into();
    expected_state
        .card_modifiers
        .entry("c_0111".into())
        .or_default()
        .extend([Modifier {
            id: "m_0002".into(),
            kind: ModifierKind::PreventOshiSkill(0),
            life_time: LifeTime::ThisTurn,
        }]);
    expected_state
        .card_modifiers
        .entry("c_0311".into())
        .or_default()
        .extend([Modifier {
            id: "m_0003".into(),
            kind: ModifierKind::PreventAllArts,
            life_time: LifeTime::ThisTurn,
        }]);
    expected_state
        .card_damage_markers
        .insert("c_0312".into(), DamageMarkers::from_hp(50));
    // expected_state.card_damage_markers

    assert_eq!(expected_state, game.state);
}

#[tokio::test]
/// hSD01-014 - Amane Kanata (Spot)
async fn hsd01_014() {
    // no need for testing: vanilla card
}

#[tokio::test]
/// hSD01-015 - Hakui Koyori (Spot)
async fn hsd01_015_sora() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-006".into()),
        back_stage: ["hSD01-015".into()].into(),
        life: ["hY01-001".into()].into(),
        main_deck: ["hSD01-015".into(), "hSD01-015".into()].into(),
        cheer_deck: ["hY01-001".into()].into(),
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
        // SoAzKo
        &[0],
        // done
        &[1],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Main;
    expected_state.player_1.collab = Some("c_0511".into());
    expected_state.player_1.back_stage = [].into();
    expected_state.player_1.holo_power = ["c_0211".into()].into();
    expected_state.player_1.main_deck = [].into();
    expected_state.player_1.hand = ["c_0311".into()].into();
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

    assert_eq!(expected_state, game.state);
}

#[tokio::test]
/// hSD01-015 - Hakui Koyori (Spot)
async fn hsd01_015_azki() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-009".into()),
        back_stage: ["hSD01-015".into()].into(),
        life: ["hY01-001".into()].into(),
        main_deck: ["hSD01-015".into(), "hSD01-015".into()].into(),
        cheer_deck: ["hY01-001".into()].into(),
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
        // SoAzKo
        &[0],
        // done
        &[1],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Main;
    expected_state.player_1.collab = Some("c_0511".into());
    expected_state.player_1.cheer_deck = [].into();
    expected_state.player_1.back_stage = [].into();
    expected_state.player_1.holo_power = ["c_0211".into()].into();
    expected_state.player_1.main_deck = ["c_0311".into()].into();
    expected_state
        .player_1
        .attachments
        .extend([(CardRef::from("c_0711"), "c_0411".into())]);
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

    assert_eq!(expected_state, game.state);
}
#[tokio::test]
/// hSD01-015 - Hakui Koyori (Spot)
async fn hsd01_015_soraz() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-013".into()),
        back_stage: ["hSD01-015".into()].into(),
        life: ["hY01-001".into()].into(),
        main_deck: ["hSD01-015".into(), "hSD01-015".into()].into(),
        cheer_deck: ["hY01-001".into()].into(),
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
        // SoAzKo
        &[0],
        // done
        &[2],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Main;
    expected_state.player_1.collab = Some("c_0511".into());
    expected_state.player_1.cheer_deck = [].into();
    expected_state.player_1.back_stage = [].into();
    expected_state.player_1.holo_power = ["c_0211".into()].into();
    expected_state.player_1.main_deck = [].into();
    expected_state.player_1.hand = ["c_0311".into()].into();
    expected_state
        .player_1
        .attachments
        .extend([(CardRef::from("c_0711"), "c_0411".into())]);
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

    assert_eq!(expected_state, game.state);
}

#[tokio::test]
/// hSD01-016 - Harusaki Nodoka (Staff)
async fn hsd01_016() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-013".into()),
        hand: ["hSD01-016".into()].into(),
        life: ["hY01-001".into()].into(),
        main_deck: [
            "hSD01-015".into(),
            "hSD01-015".into(),
            "hSD01-015".into(),
            "hSD01-015".into(),
        ]
        .into(),
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
        // Harusaki Nodoka
        &[0],
        // done
        &[3],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Main;
    expected_state.player_1.main_deck = ["c_0511".into()].into();
    expected_state.player_1.hand = ["c_0211".into(), "c_0311".into(), "c_0411".into()].into();
    expected_state.player_1.archive = ["c_0811".into()].into();
    expected_state
        .zone_modifiers
        .entry(Player::One)
        .or_default()
        .extend([(
            Zone::All,
            Modifier {
                id: "m_0001".into(),
                kind: ModifierKind::PreventLimitedSupport,
                life_time: LifeTime::ThisTurn,
            },
        )]);

    assert_eq!(expected_state, game.state);
}

#[tokio::test]
/// hSD01-017 - Mane-chan (Staff)
async fn hsd01_017() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-013".into()),
        hand: [
            "hSD01-017".into(),
            "hSD01-018".into(),
            "hSD01-019".into(),
            "hSD01-020".into(),
        ]
        .into(),
        life: ["hY01-001".into()].into(),
        main_deck: [
            "hSD01-010".into(),
            "hSD01-011".into(),
            "hSD01-012".into(),
            "hSD01-013".into(),
        ]
        .into(),
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
        // Mane-chan
        &[0],
        // done
        &[5],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Main;
    expected_state.player_1.main_deck = ["c_0b11".into(), "c_0a11".into()].into();
    expected_state.player_1.hand = [
        "c_0911".into(),
        "c_0511".into(),
        "c_0311".into(),
        "c_0211".into(),
        "c_0411".into(),
    ]
    .into();
    expected_state.player_1.archive = ["c_0811".into()].into();
    expected_state
        .zone_modifiers
        .entry(Player::One)
        .or_default()
        .extend([(
            Zone::All,
            Modifier {
                id: "m_0001".into(),
                kind: ModifierKind::PreventLimitedSupport,
                life_time: LifeTime::ThisTurn,
            },
        )]);

    assert_eq!(expected_state, game.state);
}

#[tokio::test]
/// hSD01-018 - Second PC (Item)
async fn hsd01_018() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-013".into()),
        hand: ["hSD01-018".into()].into(),
        life: ["hY01-001".into()].into(),
        main_deck: [
            "hSD01-017".into(),
            "hSD01-019".into(),
            "hSD01-020".into(),
            "hSD01-010".into(),
            "hSD01-011".into(),
            "hSD01-017".into(),
            "hSD01-019".into(),
            "hSD01-020".into(),
            "hSD01-010".into(),
            "hSD01-011".into(),
            "hSD01-012".into(),
            "hSD01-013".into(),
        ]
        .into(),
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
        // Second PC
        &[0],
        &[0],
        // done
        &[0],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Main;
    expected_state.player_1.main_deck = [
        "c_0711".into(),
        "c_0811".into(),
        "c_0911".into(),
        "c_0a11".into(),
        "c_0b11".into(),
        "c_0c11".into(),
        "c_0d11".into(),
        "c_0311".into(),
        "c_0411".into(),
        "c_0511".into(),
        "c_0611".into(),
    ]
    .into();
    expected_state.player_1.hand = ["c_0211".into()].into();
    expected_state.player_1.archive = ["c_1011".into()].into();

    assert_eq!(expected_state, game.state);
}

#[tokio::test]
/// hSD01-019 - Amazing PC (Item)
async fn hsd01_019() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-013".into()),
        hand: ["hSD01-019".into()].into(),
        life: ["hY01-001".into()].into(),
        main_deck: [
            "hSD01-010".into(),
            "hSD01-011".into(),
            "hSD01-012".into(),
            "hSD01-013".into(),
            "hSD01-014".into(),
        ]
        .into(),
        ..Default::default()
    };
    let p2 = p1.clone();

    let state = GameStateBuilder::new()
        .with_active_player(Player::One)
        .with_active_step(Step::Cheer)
        .with_player_1(p1)
        .with_attachments(
            Player::One,
            Zone::CenterStage,
            0,
            ["hY01-001".into()].into(),
        )
        .with_player_2(p2)
        .build();

    let p1_p = BufferedPrompter::new(&[
        // Amazing PC
        &[0],
        &[0],
        &[0],
        &[0],
        // done
        &[1],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Main;
    expected_state.player_1.main_deck = [
        "c_0511".into(),
        "c_0311".into(),
        "c_0411".into(),
        "c_0611".into(),
    ]
    .into();
    expected_state.player_1.hand = ["c_0211".into()].into();
    expected_state.player_1.archive = ["c_0911".into(), "c_0a31".into()].into();
    expected_state.player_1.attachments = [].into();
    expected_state
        .zone_modifiers
        .entry(Player::One)
        .or_default()
        .extend([(
            Zone::All,
            Modifier {
                id: "m_0001".into(),
                kind: ModifierKind::PreventLimitedSupport,
                life_time: LifeTime::ThisTurn,
            },
        )]);

    assert_eq!(expected_state, game.state);
}

#[tokio::test]
/// hSD01-020 - hololive Fan Circle (Event)
async fn hsd01_020() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-002".into()),
        center_stage: Some("hSD01-013".into()),
        hand: ["hSD01-020".into()].into(),
        holo_power: ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
        life: ["hY01-001".into()].into(),
        archive: ["hY01-001".into()].into(),
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
        // hololive Fan Circle
        &[0],
        // rolled a 5
        &[0],
        &[0],
        // done
        &[1],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Main;
    expected_state.player_1.hand = [].into();
    expected_state.player_1.archive = ["c_0811".into()].into();
    expected_state
        .player_1
        .attachments
        .extend([(CardRef::from("c_0711"), "c_0211".into())]);

    assert_eq!(expected_state, game.state);
}

#[tokio::test]
/// hSD01-021 - First Gravity (Event)
async fn hsd01_021() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: Some("hSD01-013".into()),
        hand: ["hSD01-021".into()].into(),
        life: ["hY01-001".into()].into(),
        main_deck: [
            "hSD01-003".into(),
            "hSD01-004".into(),
            "hSD01-005".into(),
            "hSD01-006".into(),
            "hSD01-007".into(),
            "hSD01-008".into(),
            "hSD01-009".into(),
            "hSD01-10".into(),
            "hSD01-11".into(),
            "hSD01-12".into(),
            "hSD01-13".into(),
            "hSD01-14".into(),
        ]
        .into(),
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
        // First Gravity
        &[0],
        &[0, 1],
        // done
        &[2],
    ]);
    let p2_p = BufferedPrompter::new(&[]);

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

    // performance step
    game.next_step().await.unwrap();

    // to check the changes, and apply them as checks below
    // assert_eq!(state, game.state);

    let mut expected_state = state.clone();
    expected_state.active_step = Step::Main;
    expected_state.player_1.main_deck = [
        "c_0611".into(),
        "c_0711".into(),
        "c_0811".into(),
        "c_0911".into(),
        "c_0a11".into(),
        "c_0b11".into(),
        "c_0c11".into(),
        "c_0d11".into(),
        "c_0411".into(),
        "c_0511".into(),
    ]
    .into();
    expected_state.player_1.hand = ["c_0211".into(), "c_0311".into()].into();
    expected_state.player_1.archive = ["c_1011".into()].into();
    expected_state
        .zone_modifiers
        .entry(Player::One)
        .or_default()
        .extend([(
            Zone::All,
            Modifier {
                id: "m_0001".into(),
                kind: ModifierKind::PreventLimitedSupport,
                life_time: LifeTime::ThisTurn,
            },
        )]);

    assert_eq!(expected_state, game.state);
}
