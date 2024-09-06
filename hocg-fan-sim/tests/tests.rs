use hocg_fan_sim::{
    gameplay::{CardRef, GameOutcome, GameOverReason, Player, Step, Zone},
    modifiers::{LifeTime, Modifier, ModifierKind},
    prompters::BufferedPrompter,
    tests::*,
};
use pretty_assertions::assert_eq;

#[tokio::test]
/// Goes through setup and multiple simple turns
async fn step_transitions() {
    let p1 = TestGameBoard {
        oshi: Some("hSD01-001".into()),
        center_stage: None,
        hand: [].into(),
        life: [].into(),
        main_deck: [
            "hSD01-003".into(),
            "hSD01-004".into(),
            "hSD01-005".into(),
            "hSD01-006".into(),
            "hSD01-007".into(),
            "hSD01-008".into(),
            "hSD01-009".into(),
            "hSD01-010".into(),
            "hSD01-011".into(),
            "hSD01-012".into(),
        ]
        .into(),
        cheer_deck: [
            "hY02-001".into(),
            "hY02-001".into(),
            "hY02-001".into(),
            "hY02-001".into(),
            "hY02-001".into(),
            "hY02-001".into(),
            "hY02-001".into(),
            "hY02-001".into(),
            "hY02-001".into(),
            "hY02-001".into(),
        ]
        .into(),
        ..Default::default()
    };
    let p2 = p1.clone();

    let state = GameStateBuilder::new()
        .with_player_1(p1)
        .with_player_2(p2)
        .build();

    // player 1 - inputs list
    let p1_p = BufferedPrompter::new(&[
        // - setup
        // rock
        &[0],
        // no mulligan
        &[1],
        // AZKi (Debut)
        &[0],
        // AZKi (Debut), Tokino Sora (Debut)
        &[0, 2],
        // - turn 1
        // cheer - AZKi (Debut)
        &[0],
        // main action - done
        &[5],
        // - turn 3
        // cheer - AZKi (Debut)
        &[0],
        // main action - done
        &[8],
        // performance action - done
        &[1],
        // - turn 5
        // cheer - AZKi (Debut)
        &[0],
        // main action - done
        &[9],
        // performance action - done
        &[1],
    ]);
    // player 2 - inputs list
    let p2_p = BufferedPrompter::new(&[
        // - setup
        // scissor
        &[2],
        // no mulligan
        &[1],
        // AZKi (Debut)
        &[0],
        // IRyS (Debut), Tokino Sora (Debut)
        &[0, 1],
        // - turn 2
        // cheer - AZKi (Debut)
        &[0],
        // main action - done
        &[5],
        // performance action - done
        &[1],
        // - turn 4
        // cheer - AZKi (Debut)
        &[0],
        // main action - done
        &[8],
        // performance action - done
        &[1],
        // - turn 6
        // cheer - AZKi (Debut)
        &[0],
        // main action - done
        &[9],
        // performance action - done
        &[1],
    ]);

    let (mut game, p1_client, p2_client) = setup_test_game(state.clone(), p1_p, p2_p);
    tokio::spawn(p1_client.receive_requests());
    tokio::spawn(p2_client.receive_requests());

    // setup
    game.start_game().await.unwrap();

    let mut expected_state = state.clone();
    // state - player 1
    expected_state.player_1.main_deck = ["c_0211".into(), "c_0611".into(), "c_0911".into()].into();
    expected_state.player_1.center_stage = Some("c_0811".into());
    expected_state.player_1.back_stage = ["c_0711".into(), "c_0311".into()].into();
    expected_state.player_1.life = [
        "c_1411".into(),
        "c_1311".into(),
        "c_0c11".into(),
        "c_0d11".into(),
        "c_1511".into(),
    ]
    .into();
    expected_state.player_1.cheer_deck = [
        "c_1211".into(),
        "c_0f11".into(),
        "c_1011".into(),
        "c_0e11".into(),
        "c_1111".into(),
    ]
    .into();
    expected_state.player_1.hand = [
        "c_0511".into(),
        "c_0b11".into(),
        "c_0a11".into(),
        "c_0411".into(),
    ]
    .into();
    expected_state
        .zone_modifiers
        .entry(Player::One)
        .or_default()
        .extend([
            (
                Zone::All,
                Modifier {
                    id: "m_0001".into(),
                    kind: ModifierKind::SkipStep(Step::Reset),
                    life_time: LifeTime::NextTurn(Player::One),
                },
            ),
            (
                Zone::All,
                Modifier {
                    id: "m_0003".into(),
                    kind: ModifierKind::PreventLimitedSupport,
                    life_time: LifeTime::NextTurn(Player::One),
                },
            ),
            (
                Zone::All,
                Modifier {
                    id: "m_0004".into(),
                    kind: ModifierKind::PreventBloom,
                    life_time: LifeTime::NextTurn(Player::One),
                },
            ),
            (
                Zone::All,
                Modifier {
                    id: "m_0006".into(),
                    kind: ModifierKind::SkipStep(Step::Performance),
                    life_time: LifeTime::NextTurn(Player::One),
                },
            ),
        ]);
    // state - player 2
    expected_state.player_2.main_deck = ["c_0312".into(), "c_0a12".into(), "c_0712".into()].into();
    expected_state.player_2.center_stage = Some("c_0812".into());
    expected_state.player_2.back_stage = ["c_0612".into(), "c_0212".into()].into();
    expected_state.player_2.life = [
        "c_1112".into(),
        "c_1312".into(),
        "c_0c12".into(),
        "c_1212".into(),
        "c_0e12".into(),
    ]
    .into();
    expected_state.player_2.cheer_deck = [
        "c_1512".into(),
        "c_0f12".into(),
        "c_0d12".into(),
        "c_1412".into(),
        "c_1012".into(),
    ]
    .into();
    expected_state.player_2.hand = [
        "c_0912".into(),
        "c_0512".into(),
        "c_0412".into(),
        "c_0b12".into(),
    ]
    .into();
    expected_state
        .zone_modifiers
        .entry(Player::Two)
        .or_default()
        .extend([
            (
                Zone::All,
                Modifier {
                    id: "m_0002".into(),
                    kind: ModifierKind::SkipStep(Step::Reset),
                    life_time: LifeTime::NextTurn(Player::Two),
                },
            ),
            (
                Zone::All,
                Modifier {
                    id: "m_0005".into(),
                    kind: ModifierKind::PreventBloom,
                    life_time: LifeTime::NextTurn(Player::Two),
                },
            ),
        ]);

    assert_eq!(expected_state, game.state);

    // player 1 - turn 1 - reset step
    game.next_step().await.unwrap();

    let mut expected_state = expected_state.clone();
    expected_state.active_step = Step::Reset;
    expected_state.turn_number = 1;
    // state - player 1
    expected_state.zone_modifiers.get_mut(&Player::One).unwrap()[0]
        .1
        .life_time = LifeTime::ThisTurn;
    expected_state.zone_modifiers.get_mut(&Player::One).unwrap()[1]
        .1
        .life_time = LifeTime::ThisTurn;
    expected_state.zone_modifiers.get_mut(&Player::One).unwrap()[2]
        .1
        .life_time = LifeTime::ThisTurn;
    expected_state.zone_modifiers.get_mut(&Player::One).unwrap()[3]
        .1
        .life_time = LifeTime::ThisTurn;

    assert_eq!(expected_state, game.state);

    // player 1 - turn 1 - draw step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Draw;
    // state - player 1
    expected_state.player_1.main_deck = ["c_0611".into(), "c_0911".into()].into();
    expected_state.player_1.hand = [
        "c_0511".into(),
        "c_0b11".into(),
        "c_0a11".into(),
        "c_0411".into(),
        "c_0211".into(),
    ]
    .into();

    assert_eq!(expected_state, game.state);

    // player 1 - turn 1 - cheer step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Cheer;
    // state - player 1
    expected_state.player_1.cheer_deck = [
        "c_0f11".into(),
        "c_1011".into(),
        "c_0e11".into(),
        "c_1111".into(),
    ]
    .into();
    expected_state
        .player_1
        .attachments
        .extend([(CardRef::from("c_1211"), "c_0811".into())]);

    assert_eq!(expected_state, game.state);

    // player 1 - turn 1 - main step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Main;

    assert_eq!(expected_state, game.state);

    // player 1 - turn 1 - performance step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Performance;

    assert_eq!(expected_state, game.state);

    // player 1 - turn 1 - end step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::End;
    // state - player 1
    expected_state.zone_modifiers.insert(Player::One, [].into());

    assert_eq!(expected_state, game.state);

    // player 2 - turn 2 - reset step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Reset;
    expected_state.active_player = Player::Two;
    expected_state.turn_number = 2;
    // state - player 2
    expected_state.zone_modifiers.get_mut(&Player::Two).unwrap()[0]
        .1
        .life_time = LifeTime::ThisTurn;
    expected_state.zone_modifiers.get_mut(&Player::Two).unwrap()[1]
        .1
        .life_time = LifeTime::ThisTurn;

    assert_eq!(expected_state, game.state);

    // player 2 - turn 2 - draw step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Draw;
    // state - player 2
    expected_state.player_2.main_deck = ["c_0a12".into(), "c_0712".into()].into();
    expected_state.player_2.hand = [
        "c_0912".into(),
        "c_0512".into(),
        "c_0412".into(),
        "c_0b12".into(),
        "c_0312".into(),
    ]
    .into();

    assert_eq!(expected_state, game.state);

    // player 2 - turn 2 - cheer step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Cheer;
    // state - player 2
    expected_state.player_2.cheer_deck = [
        "c_0f12".into(),
        "c_0d12".into(),
        "c_1412".into(),
        "c_1012".into(),
    ]
    .into();
    expected_state
        .player_2
        .attachments
        .extend([(CardRef::from("c_1512"), "c_0812".into())]);

    assert_eq!(expected_state, game.state);

    // player 2 - turn 2 - main step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Main;

    assert_eq!(expected_state, game.state);

    // player 2 - turn 2 - performance step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Performance;

    assert_eq!(expected_state, game.state);

    // player 2 - turn 2 - end step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::End;
    // state - player 2
    expected_state.zone_modifiers.insert(Player::Two, [].into());

    assert_eq!(expected_state, game.state);

    // player 1 - turn 3 - reset step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Reset;
    expected_state.active_player = Player::One;
    expected_state.turn_number = 3;

    assert_eq!(expected_state, game.state);

    // player 1 - turn 3 - draw step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Draw;
    // state - player 1
    expected_state.player_1.main_deck = ["c_0911".into()].into();
    expected_state.player_1.hand = [
        "c_0511".into(),
        "c_0b11".into(),
        "c_0a11".into(),
        "c_0411".into(),
        "c_0211".into(),
        "c_0611".into(),
    ]
    .into();

    assert_eq!(expected_state, game.state);

    // player 1 - turn 3 - cheer step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Cheer;
    // state - player 1
    expected_state.player_1.cheer_deck = ["c_1011".into(), "c_0e11".into(), "c_1111".into()].into();
    expected_state
        .player_1
        .attachments
        .extend([(CardRef::from("c_0f11"), "c_0811".into())]);

    assert_eq!(expected_state, game.state);

    // player 1 - turn 3 - main step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Main;

    assert_eq!(expected_state, game.state);

    // player 1 - turn 3 - performance step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Performance;

    assert_eq!(expected_state, game.state);

    // player 1 - turn 3 - end step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::End;

    assert_eq!(expected_state, game.state);

    // player 2 - turn 4 - reset step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Reset;
    expected_state.active_player = Player::Two;
    expected_state.turn_number = 4;

    assert_eq!(expected_state, game.state);

    // player 2 - turn 4 - draw step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Draw;
    // state - player 2
    expected_state.player_2.main_deck = ["c_0712".into()].into();
    expected_state.player_2.hand = [
        "c_0912".into(),
        "c_0512".into(),
        "c_0412".into(),
        "c_0b12".into(),
        "c_0312".into(),
        "c_0a12".into(),
    ]
    .into();

    assert_eq!(expected_state, game.state);

    // player 2 - turn 4 - cheer step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Cheer;
    // state - player 2
    expected_state.player_2.cheer_deck = ["c_0d12".into(), "c_1412".into(), "c_1012".into()].into();
    expected_state
        .player_2
        .attachments
        .extend([(CardRef::from("c_0f12"), "c_0812".into())]);

    assert_eq!(expected_state, game.state);

    // player 2 - turn 4 - main step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Main;

    assert_eq!(expected_state, game.state);

    // player 2 - turn 4 - performance step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Performance;

    assert_eq!(expected_state, game.state);

    // player 2 - turn 4 - end step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::End;

    assert_eq!(expected_state, game.state);

    // player 1 - turn 5 - reset step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Reset;
    expected_state.active_player = Player::One;
    expected_state.turn_number = 5;

    assert_eq!(expected_state, game.state);

    // player 1 - turn 5 - draw step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Draw;
    // state - player 1
    expected_state.player_1.main_deck = [].into();
    expected_state.player_1.hand = [
        "c_0511".into(),
        "c_0b11".into(),
        "c_0a11".into(),
        "c_0411".into(),
        "c_0211".into(),
        "c_0611".into(),
        "c_0911".into(),
    ]
    .into();

    assert_eq!(expected_state, game.state);

    // player 1 - turn 5 - cheer step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Cheer;
    // state - player 1
    expected_state.player_1.cheer_deck = ["c_0e11".into(), "c_1111".into()].into();
    expected_state
        .player_1
        .attachments
        .extend([(CardRef::from("c_1011"), "c_0811".into())]);

    assert_eq!(expected_state, game.state);

    // player 1 - turn 5 - main step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Main;

    assert_eq!(expected_state, game.state);

    // player 1 - turn 5 - performance step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Performance;

    assert_eq!(expected_state, game.state);

    // player 1 - turn 5 - end step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::End;

    assert_eq!(expected_state, game.state);

    // player 2 - turn 6 - reset step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Reset;
    expected_state.active_player = Player::Two;
    expected_state.turn_number = 6;

    assert_eq!(expected_state, game.state);

    // player 2 - turn 6 - draw step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Draw;
    // state - player 2
    expected_state.player_2.main_deck = [].into();
    expected_state.player_2.hand = [
        "c_0912".into(),
        "c_0512".into(),
        "c_0412".into(),
        "c_0b12".into(),
        "c_0312".into(),
        "c_0a12".into(),
        "c_0712".into(),
    ]
    .into();

    assert_eq!(expected_state, game.state);

    // player 2 - turn 6 - cheer step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Cheer;
    // state - player 2
    expected_state.player_2.cheer_deck = ["c_1412".into(), "c_1012".into()].into();
    expected_state
        .player_2
        .attachments
        .extend([(CardRef::from("c_0d12"), "c_0812".into())]);

    assert_eq!(expected_state, game.state);

    // player 2 - turn 6 - main step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Main;

    assert_eq!(expected_state, game.state);

    // player 2 - turn 6 - performance step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Performance;

    assert_eq!(expected_state, game.state);

    // player 2 - turn 6 - end step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::End;

    assert_eq!(expected_state, game.state);

    // player 1 - turn 7 - reset step
    game.next_step().await.unwrap();
    expected_state.active_step = Step::Reset;
    expected_state.active_player = Player::One;
    expected_state.turn_number = 7;

    assert_eq!(expected_state, game.state);

    // player 1 - turn 7 - draw step
    assert_eq!(
        Err(GameOutcome {
            winning_player: Some(Player::Two),
            reason: GameOverReason::EmptyDeckInDrawStep
        }),
        game.next_step().await
    );
}
