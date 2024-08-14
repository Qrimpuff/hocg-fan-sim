use std::{
    collections::HashMap,
    sync::{mpsc, OnceLock},
    thread,
};

use hocg_fan_sim::{
    cards::CardNumber,
    client::{Client, DefaultEventHandler},
    gameplay::{
        register_card, CardRef, Game, GameBoard, GameOutcome, GameOverReason, GameState, Player,
        Step, Zone,
    },
    modifiers::{DamageMarkers, Modifier},
    prompters::BufferedPrompter,
};
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
    let p1_channel_1 = mpsc::channel();
    let p1_channel_2 = mpsc::channel();
    let p2_channel_1 = mpsc::channel();
    let p2_channel_2 = mpsc::channel();

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
    thread::spawn(move || {
        p1_client.receive_requests();
    });

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
    thread::spawn(move || {
        p2_client.receive_requests();
    });

    game
}
