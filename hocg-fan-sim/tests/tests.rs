use std::{collections::HashMap, sync::OnceLock};

use hocg_fan_sim::{
    cards::CardNumber,
    client::{Client, DefaultEventHandler},
    gameplay::{
        register_card, CardRef, Game, GameBoard, GameOutcome, GameOverReason, GameState, Player,
        Step, Zone,
    },
    modifiers::{DamageMarkers, LifeTime, Modifier, ModifierKind},
    prompters::BufferedPrompter,
};
use pretty_assertions::assert_eq;
use rand::{rngs::StdRng, SeedableRng};

mod sets;

pub fn rng<'a>() -> &'a StdRng {
    static RNG: OnceLock<StdRng> = OnceLock::new();
    RNG.get_or_init(|| StdRng::seed_from_u64(123456))
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct TestGameBoard {
    pub oshi: Option<CardNumber>,
    pub main_deck: Vec<CardNumber>,
    pub center_stage: Option<CardNumber>,
    pub collab: Option<CardNumber>,
    pub back_stage: Vec<CardNumber>,
    pub life: Vec<CardNumber>,
    pub cheer_deck: Vec<CardNumber>,
    pub holo_power: Vec<CardNumber>,
    pub archive: Vec<CardNumber>,
    pub hand: Vec<CardNumber>,
}

impl TestGameBoard {
    pub fn to_game_board(
        &self,
        player: Player,
        next_card_ref: &mut u8,
        state: &mut GameState,
    ) -> GameBoard {
        GameBoard {
            oshi: self
                .oshi
                .iter()
                .map(|c| register_card(player, 1, c, next_card_ref, &mut state.card_map))
                .next(),
            main_deck: self
                .main_deck
                .iter()
                .map(|c| register_card(player, 1, c, next_card_ref, &mut state.card_map))
                .collect(),
            center_stage: self
                .center_stage
                .iter()
                .map(|c| register_card(player, 1, c, next_card_ref, &mut state.card_map))
                .next(),
            collab: self
                .collab
                .iter()
                .map(|c| register_card(player, 1, c, next_card_ref, &mut state.card_map))
                .next(),
            back_stage: self
                .back_stage
                .iter()
                .map(|c| register_card(player, 1, c, next_card_ref, &mut state.card_map))
                .collect(),
            life: self
                .life
                .iter()
                .map(|c| register_card(player, 1, c, next_card_ref, &mut state.card_map))
                .collect(),
            cheer_deck: self
                .cheer_deck
                .iter()
                .map(|c| register_card(player, 1, c, next_card_ref, &mut state.card_map))
                .collect(),
            holo_power: self
                .holo_power
                .iter()
                .map(|c| register_card(player, 1, c, next_card_ref, &mut state.card_map))
                .collect(),
            archive: self
                .archive
                .iter()
                .map(|c| register_card(player, 1, c, next_card_ref, &mut state.card_map))
                .collect(),
            hand: self
                .hand
                .iter()
                .map(|c| register_card(player, 1, c, next_card_ref, &mut state.card_map))
                .collect(),
            ..Default::default()
        }
    }
}

#[derive(Default)]
struct GameStateBuilder {
    next_p1_card_ref: u8,
    next_p2_card_ref: u8,
    state: GameState,
}
impl GameStateBuilder {
    pub fn new() -> Self {
        Self {
            next_p1_card_ref: 1,
            next_p2_card_ref: 1,
            ..Default::default()
        }
    }

    pub fn build(self) -> GameState {
        self.state
    }

    pub fn with_player_1(mut self, player_1: TestGameBoard) -> Self {
        self.state.player_1 =
            player_1.to_game_board(Player::One, &mut self.next_p1_card_ref, &mut self.state);
        self
    }
    pub fn with_player_2(mut self, player_2: TestGameBoard) -> Self {
        self.state.player_2 =
            player_2.to_game_board(Player::Two, &mut self.next_p2_card_ref, &mut self.state);
        self
    }
    pub fn with_attachments(
        mut self,
        player: Player,
        zone: Zone,
        card_idx: usize,
        attachments: Vec<CardNumber>,
    ) -> Self {
        let next_card_ref = {
            match player {
                Player::One => &mut self.next_p1_card_ref,
                Player::Two => &mut self.next_p2_card_ref,
                Player::Both => unreachable!("both players is not valid"),
            }
        };
        let card = self.state.board(player).get_zone(zone).all_cards()[card_idx];
        for att in attachments {
            let att = register_card(player, 3, &att, next_card_ref, &mut self.state.card_map);
            self.state.board_mut(player).attachments.insert(att, card);
        }
        self
    }
    pub fn with_active_player(mut self, active_player: Player) -> Self {
        self.state.active_player = active_player;
        self
    }
    pub fn with_active_step(mut self, active_step: Step) -> Self {
        self.state.active_step = active_step;
        self
    }
    pub fn with_turn_number(mut self, turn_number: u8) -> Self {
        self.state.turn_number = turn_number;
        self
    }
    pub fn with_zone_modifiers(
        mut self,
        zone_modifiers: HashMap<Player, Vec<(Zone, Modifier)>>,
    ) -> Self {
        self.state.zone_modifiers = zone_modifiers;
        self
    }
    pub fn with_card_modifiers(mut self, card_modifiers: HashMap<CardRef, Vec<Modifier>>) -> Self {
        self.state.card_modifiers = card_modifiers;
        self
    }
    pub fn with_card_damage_markers(
        mut self,
        card_damage_markers: HashMap<CardRef, DamageMarkers>,
    ) -> Self {
        self.state.card_damage_markers = card_damage_markers;
        self
    }
}

// will spawn two threads to handle the client connections
pub fn setup_test_game(
    state: GameState,
    player_1_prompt: BufferedPrompter,
    player_2_prompt: BufferedPrompter,
) -> Game {
    let p1_channel_1 = async_channel::bounded(10);
    let p1_channel_2 = async_channel::bounded(10);
    let p2_channel_1 = async_channel::bounded(10);
    let p2_channel_2 = async_channel::bounded(10);

    let game = Game::with_game_state(
        state.clone(),
        (p1_channel_1.0, p1_channel_2.1),
        (p2_channel_1.0, p2_channel_2.1),
        rng().clone(),
    );

    // Player 1
    let mut p1_client = Client::new(
        (p1_channel_2.0, p1_channel_1.1),
        DefaultEventHandler::new(),
        player_1_prompt,
    );
    p1_client.game = state.clone();
    // for the client to stop gracefully
    p1_client.game.game_outcome = Some(GameOutcome {
        winning_player: None,
        reason: GameOverReason::Draw,
    });
    tokio::spawn(p1_client.receive_requests());

    // Player 2
    let mut p2_client = Client::new(
        (p2_channel_2.0, p2_channel_1.1),
        DefaultEventHandler::new(),
        player_2_prompt,
    );
    p2_client.game = state;
    // for the client to stop gracefully
    p2_client.game.game_outcome = Some(GameOutcome {
        winning_player: None,
        reason: GameOverReason::Draw,
    });
    tokio::spawn(p2_client.receive_requests());

    game
}

///////////////////////////////////////////////////////////////////////

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

    let mut game = setup_test_game(state.clone(), p1_p, p2_p);

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
