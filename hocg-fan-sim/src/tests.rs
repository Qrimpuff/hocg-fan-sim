use std::{collections::HashMap, env};

use crate::{
    cards::CardNumber, client::*, gameplay::*, library::load_library, modifiers::*,
    prompters::BufferedPrompter,
};
use rand::{rngs::StdRng, SeedableRng};
use time::macros::format_description;
use tracing_subscriber::{fmt::time::LocalTime, EnvFilter};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TestGameBoard {
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
pub struct GameStateBuilder {
    next_modifier_ref: u16,
    next_p1_card_ref: u8,
    next_p2_card_ref: u8,
    state: GameState,
}
impl GameStateBuilder {
    pub fn new() -> Self {
        Self {
            // add modifiers at the end, to avoid conflicts
            next_modifier_ref: u16::MAX - 100,
            next_p1_card_ref: 1,
            next_p2_card_ref: 1,
            ..Default::default()
        }
    }

    pub fn build(mut self) -> GameState {
        // and the empty zone modifiers, for test that needs them
        self.state.zone_modifiers.entry(Player::One).or_default();
        self.state.zone_modifiers.entry(Player::Two).or_default();

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
        player: Player,
        zone: Zone,
        kind: ModifierKind,
        life_time: LifeTime,
    ) -> Self {
        self.state.zone_modifiers.entry(player).or_default().push((
            zone,
            Modifier::for_zone(player, zone, kind, life_time, &mut self.next_modifier_ref),
        ));
        self
    }
    pub fn with_card_modifiers(mut self, card_modifiers: HashMap<CardRef, Vec<Modifier>>) -> Self {
        self.state.card_modifiers = card_modifiers;
        self
    }
    pub fn with_damage_markers(
        mut self,
        player: Player,
        zone: Zone,
        card_idx: usize,
        damage_markers: DamageMarkers,
    ) -> Self {
        let card = self.state.board(player).get_zone(zone).all_cards()[card_idx];
        self.state.card_damage_markers.insert(card, damage_markers);
        self
    }
}

// will spawn two threads to handle the client connections
pub async fn setup_test_game(
    state: GameState,
    player_1_prompt: BufferedPrompter,
    player_2_prompt: BufferedPrompter,
) -> (
    GameDirector,
    Client<DefaultEventHandler, BufferedPrompter>,
    Client<DefaultEventHandler, BufferedPrompter>,
) {
    load_library(&include_bytes!("../../hocg-fan-lib.gz")[..]).await;

    let p1_channel_1 = async_channel::bounded(10);
    let p1_channel_2 = async_channel::bounded(10);
    let p2_channel_1 = async_channel::bounded(10);
    let p2_channel_2 = async_channel::bounded(10);

    let game = GameDirector::with_game_state(
        state.clone(),
        (p1_channel_1.0, p1_channel_2.1),
        (p2_channel_1.0, p2_channel_2.1),
        StdRng::seed_from_u64(123456),
    )
    .await;

    // Player 1
    let mut p1_client: Client<DefaultEventHandler, BufferedPrompter> = Client::new(
        (p1_channel_2.0, p1_channel_1.1),
        DefaultEventHandler::new(),
        player_1_prompt,
    )
    .await;
    p1_client.game.state = state.clone();
    // // for the client to stop gracefully (not sure if it's needed anymore)
    // p1_client.game.state.game_outcome = Some(GameOutcome {
    //     winning_player: None,
    //     reason: GameOverReason::Draw,
    // });
    // tokio::spawn(p1_client.receive_requests());

    // Player 2
    let mut p2_client = Client::new(
        (p2_channel_2.0, p2_channel_1.1),
        DefaultEventHandler::new(),
        player_2_prompt,
    )
    .await;
    p2_client.game.state = state;
    // // for the client to stop gracefully (not sure if it's needed anymore)
    // p2_client.game.state.game_outcome = Some(GameOutcome {
    //     winning_player: None,
    //     reason: GameOverReason::Draw,
    // });
    // tokio::spawn(p2_client.receive_requests());

    (game, p1_client, p2_client)
}

pub fn setup_test_logs() -> tracing_appender::non_blocking::WorkerGuard {
    // --------------- setup logs ---------------------
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", "DEBUG");

    let file_appender = tracing_appender::rolling::daily("logs", "hocg-fan-sim.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let _ = tracing_subscriber::fmt()
        .with_timer(LocalTime::new(format_description!(
            "[year]-[month]-[day] [hour repr:24]:[minute]:[second].[subsecond digits:4]"
        )))
        .with_writer(non_blocking)
        .with_ansi(false)
        // enable thread id to be emitted
        .with_thread_ids(true)
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
    // -------------- end setup logs -------------------
    guard
}
