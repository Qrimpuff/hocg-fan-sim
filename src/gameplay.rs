use std::{collections::HashMap, fmt::Debug, sync::Arc};

use super::cards::*;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

type CardUuid = u32;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Player {
    One,
    Two,
    Both,
}

#[derive(Debug)]
pub struct Game {
    library: Arc<GlobalLibrary>,
    player_1: GameBoard,
    player_2: GameBoard,
    card_map: HashMap<CardUuid, (Player, CardRef)>,
    active_player: Player,
    active_phase: Phase,
    zone_buffs: Vec<(Player, Zone, Buff)>,
    card_buffs: Vec<(CardUuid, Buff)>,
    prompter: Box<dyn Prompter>,
}

impl Game {
    pub fn setup(library: Arc<GlobalLibrary>, player_1: &Loadout, player_2: &Loadout) -> Game {
        let mut card_map = HashMap::new();
        Game {
            library,
            player_1: GameBoard::setup(Player::One, player_1, &mut card_map),
            player_2: GameBoard::setup(Player::Two, player_2, &mut card_map),
            card_map,
            active_player: Player::One,
            active_phase: Phase::Setup,
            zone_buffs: Vec::new(),
            card_buffs: Vec::new(),
            prompter: Box::new(DefaultPrompter {}),
        }
    }

    fn register_card(
        player: Player,
        card_ref: &CardRef,
        card_map: &mut HashMap<CardUuid, (Player, CardRef)>,
    ) -> CardUuid {
        let uuid: CardUuid = thread_rng().gen();
        card_map.insert(uuid, (player, card_ref.clone()));
        uuid
    }

    fn active_board(&mut self) -> &mut GameBoard {
        match self.active_player {
            Player::One => &mut self.player_1,
            Player::Two => &mut self.player_2,
            _ => panic!("both players cannot be active at the same time"),
        }
    }

    fn shuffle_deck(deck: &mut Vec<CardUuid>) {
        deck.shuffle(&mut thread_rng());
    }

    fn need_mulligan(&self, player: &GameBoard) -> bool {
        // need to have at least one debut member to place on the stage
        !player
            .hand
            .iter()
            .filter_map(|c| {
                if let Card::HoloMember(m) = self.lookup_card(*c) {
                    Some(m)
                } else {
                    None
                }
            })
            .any(|m| m.rank == HoloMemberRank::Debut)
    }

    pub fn start_game(&mut self) {
        const STARTING_HAND_SIZE: usize = 7;

        self.active_phase = Phase::Setup;

        // - shuffle main deck
        Game::shuffle_deck(&mut self.player_1.main_deck);
        Game::shuffle_deck(&mut self.player_2.main_deck);

        // - shuffle cheer deck
        Game::shuffle_deck(&mut self.player_1.cheer_deck);
        Game::shuffle_deck(&mut self.player_2.cheer_deck);

        // - oshi face down
        // TODO oshi hide

        // - rock/paper/scissor to choose first (coin flip?)
        // TODO rps (prompt)
        self.active_player = Player::One;

        // - draw 7 cards from main deck
        let mut player_1_draw = STARTING_HAND_SIZE;
        let mut player_2_draw = STARTING_HAND_SIZE;
        self.player_1.draw(player_1_draw);
        self.player_2.draw(player_2_draw);

        //   - can mulligan for -1 card. at 0 lose the game
        // TODO don't repeat
        // mulligan for player 1
        loop {
            let force_mulligan_1 = self.need_mulligan(&self.player_1);
            // TODO mulligan  (prompt)
            let mulligan_1 = force_mulligan_1 || 1 == 2;
            if !mulligan_1 {
                break;
            }

            println!("player 1 mulligan");
            self.player_1.send_all_from_zone(Zone::Hand, Zone::MainDeck);
            Game::shuffle_deck(&mut self.player_1.main_deck);
            player_1_draw -= 1;
            if player_1_draw == 0 {
                break;
            }
            self.player_1.draw(player_1_draw);
        }
        if player_1_draw == 0 {
            self.active_player = Player::One;
            self.lose_game();
            return;
        }
        // mulligan for player 2
        loop {
            let force_mulligan_2 = self.need_mulligan(&self.player_2);
            // TODO mulligan  (prompt)
            let mulligan_2 = force_mulligan_2 || 1 == 2;
            if !mulligan_2 {
                break;
            }

            println!("player 2 mulligan");
            self.player_2.send_all_from_zone(Zone::Hand, Zone::MainDeck);
            Game::shuffle_deck(&mut self.player_2.main_deck);
            player_2_draw -= 1;
            if player_2_draw == 0 {
                break;
            }
            self.player_2.draw(player_2_draw);
        }
        if player_2_draw == 0 {
            self.active_player = Player::Two;
            self.lose_game();
            return;
        }

        // - place debut member center face down
        // TODO select debut member (prompt)
        let debut_1 = self
            .player_1
            .hand
            .iter()
            .filter_map(|c| {
                if let Card::HoloMember(m) = self.lookup_card(*c) {
                    Some((c, m))
                } else {
                    None
                }
            })
            .find(|(_, m)| m.rank == HoloMemberRank::Debut)
            .unwrap()
            .0;
        self.player_1.send_to_zone(*debut_1, Zone::MainStageCenter);
        let debut_2 = self
            .player_2
            .hand
            .iter()
            .filter_map(|c| {
                if let Card::HoloMember(m) = self.lookup_card(*c) {
                    Some((c, m))
                } else {
                    None
                }
            })
            .find(|(_, m)| m.rank == HoloMemberRank::Debut)
            .unwrap()
            .0;
        self.player_2.send_to_zone(*debut_2, Zone::MainStageCenter);

        // - place other debut members backstage
        // TODO select debut members (prompt)
        let other_debut_1: Vec<_> = self
            .player_1
            .hand
            .iter()
            .filter_map(|c| {
                if let Card::HoloMember(m) = self.lookup_card(*c) {
                    Some((c, m))
                } else {
                    None
                }
            })
            .filter(|(_, m)| m.rank == HoloMemberRank::Debut)
            .map(|(c, _)| c)
            .copied()
            .collect();
        self.player_1
            .send_many_to_zone(other_debut_1, Zone::BackStage);
        let other_debut_2: Vec<_> = self
            .player_2
            .hand
            .iter()
            .filter_map(|c| {
                if let Card::HoloMember(m) = self.lookup_card(*c) {
                    Some((c, m))
                } else {
                    None
                }
            })
            .filter(|(_, m)| m.rank == HoloMemberRank::Debut)
            .map(|(c, _)| c)
            .copied()
            .collect();
        self.player_2
            .send_many_to_zone(other_debut_2, Zone::BackStage);

        // - reveal face down oshi and members
        // TODO oshi reveal

        // - draw life cards face down from cheer
        let Card::OshiHoloMember(oshi_1) = self
            .player_1
            .oshi
            .map(|c| self.lookup_card(c))
            .expect("oshi should always be there")
        else {
            panic!("card should be oshi")
        };
        self.player_1.add_life(oshi_1.life);
        let Card::OshiHoloMember(oshi_2) = self
            .player_2
            .oshi
            .map(|c| self.lookup_card(c))
            .expect("oshi should always be there")
        else {
            panic!("card should be oshi")
        };
        self.player_2.add_life(oshi_2.life);

        // - game start (live start?)
        self.next_phase();
    }

    pub fn next_phase(&mut self) {
        self.active_phase = match self.active_phase {
            Phase::Setup => Phase::Refresh,
            Phase::Refresh => Phase::Draw,
            Phase::Draw => Phase::Cheer,
            Phase::Cheer => Phase::Main,
            Phase::Main => Phase::Art,
            Phase::Art => Phase::End,
            Phase::End => {
                self.active_player = match self.active_player {
                    Player::One => Player::Two,
                    Player::Two => Player::One,
                    _ => panic!("both players cannot be active at the same time"),
                };
                Phase::Refresh
            }
        };
        // TODO trigger phase change effects
    }

    fn refresh_phase(&mut self) {
        // - all members from rest to active
        for mem in self.active_board().all_members_on_stage() {
            self.awake_card(mem);
        }

        // - collab to backstage in rest
        if let Some(mem) = self.active_board().main_collab {
            // TODO send to backstage
            self.rest_card(mem);
        }
        // - if no center, backstage to center
        // TODO select member (prompt)
        //   - no backstage lose game
        if self.active_board().main_center.is_none() {
            self.lose_game();
            // return;
        }
    }
    fn draw_phase(&mut self) {
        // - draw 1 card from main deck
    }
    fn cheer_phase(&mut self) {
        // - draw 1 card from cheer deck, attach it
        // - main stage or backstage
    }
    fn main_phase(&mut self) {
        // - place debut member on backstage
        // - bloom member (evolve e.g. debut -> 1st )
        //   - bloom effect
        //   - can't bloom on same turn as placed
        // - use support card
        //   - only one limited per turn
        //   - otherwise unlimited
        // - put backstage member in collab
        //   - can be done on first turn?
        //   - draw down card from deck into power zone
        // - retreat switch (baton pass)
        //   - switch center with backstage
        //   - remove attached cheer for cost
        // - use abilities (including oshi)
        //   - oshi power uses card in power zone
        //   - once per turn / once per game?
    }
    fn art_phase(&mut self) {
        // - can use 2 attacks (center, collab)
        // - can choose target (center, collab)
        // - need required attached cheers to attack
        // - apply damage and effects
        // - remove member if defeated
        //   - lose 1 life
        //   - attach lost life (cheer)
    }
    fn end_phase(&mut self) {
        // - any end phase effect
    }

    fn win_game(&mut self) {
        match self.active_player {
            Player::One => println!("player 1 wins"),
            Player::Two => println!("player 2 wins"),
            _ => panic!("both players cannot be active at the same time"),
        };
    }
    fn lose_game(&mut self) {
        self.active_player = match self.active_player {
            Player::One => Player::Two,
            Player::Two => Player::One,
            _ => panic!("both players cannot be active at the same time"),
        };
        self.win_game();
    }

    pub fn lookup_card(&self, uuid: CardUuid) -> &Card {
        let (_, card_ref) = self.card_map.get(&uuid).expect("should be in the map");
        self.library
            .lookup_card(card_ref)
            .expect("should be in the library")
    }

    pub fn find_buffs(&self, uuid: CardUuid) -> Vec<&Buff> {
        let (player, _) = self.card_map.get(&uuid).expect("should be in the map");
        let buffs = self
            .card_buffs
            .iter()
            .filter(|(c, _)| *c == uuid)
            .map(|(_, b)| b);
        let zone = match player {
            Player::One => self
                .player_1
                .find_card_zone(uuid)
                .expect("the card should be on player 1 side"),
            Player::Two => self
                .player_2
                .find_card_zone(uuid)
                .expect("the card should be on player 2 side"),
            Player::Both => panic!("a card can't be owned by both player"),
        };
        let buffs: Vec<_> = buffs
            .chain(
                self.zone_buffs
                    .iter()
                    .filter(|(p, z, _)| p == player && *z == zone || *z == Zone::All)
                    .map(|(_, _, b)| b),
            )
            .collect();
        buffs
    }

    pub fn rest_card(&mut self, _uuid: CardUuid) {
        // TODO probably through debuff
    }
    pub fn awake_card(&mut self, _uuid: CardUuid) {
        // TODO probably through debuff
    }
}

#[derive(Debug)]
pub struct GameBoard {
    main_deck: Vec<CardUuid>,
    oshi: Option<CardUuid>,
    main_center: Option<CardUuid>,
    main_collab: Option<CardUuid>,
    back_stage: Vec<CardUuid>,
    life: Vec<CardUuid>,
    cheer_deck: Vec<CardUuid>,
    holo_power: Vec<CardUuid>,
    archive: Vec<CardUuid>,
    hand: Vec<CardUuid>,
}

impl GameBoard {
    pub fn setup(
        player: Player,
        loadout: &Loadout,
        card_map: &mut HashMap<CardUuid, (Player, CardRef)>,
    ) -> GameBoard {
        GameBoard {
            main_deck: loadout
                .main_deck
                .iter()
                .map(|c| Game::register_card(player, c, card_map))
                .collect(),
            oshi: Some(Game::register_card(player, &loadout.oshi, card_map)),
            main_center: None,
            main_collab: None,
            back_stage: Vec::new(),
            life: Vec::new(),
            cheer_deck: loadout
                .cheer_deck
                .iter()
                .map(|c| Game::register_card(player, c, card_map))
                .collect(),
            holo_power: Vec::new(),
            archive: Vec::new(),
            hand: Vec::new(),
        }
    }

    pub fn draw(&mut self, amount: usize) {
        self.send_from_zone(Zone::MainDeck, Zone::Hand, amount);
    }

    pub fn add_life(&mut self, amount: u8) {
        self.send_from_zone(Zone::CheerDeck, Zone::Life, amount.into());
    }

    pub fn all_members_on_stage(&self) -> Vec<CardUuid> {
        self.back_stage
            .iter()
            .chain(&self.main_center)
            .chain(&self.main_collab)
            .cloned()
            .collect()
    }

    pub fn send_to_zone(&mut self, uuid: CardUuid, target_zone: Zone) {
        let current_zone = self.find_card_zone(uuid);
        if let Some(zone) = current_zone {
            self.get_zone(zone).remove_card(uuid);
            self.get_zone(target_zone).add_top_card(uuid);
        }
    }

    pub fn send_many_to_zone(
        &mut self,
        uuids: impl IntoIterator<Item = CardUuid>,
        target_zone: Zone,
    ) {
        uuids
            .into_iter()
            .for_each(|c| self.send_to_zone(c, target_zone));
    }

    pub fn send_from_zone(&mut self, current_zone: Zone, target_zone: Zone, amount: usize) {
        for _ in 0..amount {
            if let Some(card) = self.get_zone(current_zone).peek_top_card() {
                self.send_to_zone(card, target_zone);
            }
        }
    }

    pub fn send_all_from_zone(&mut self, current_zone: Zone, target_zone: Zone) -> usize {
        let amount = self.get_zone(current_zone).count();
        self.send_from_zone(current_zone, target_zone, amount);
        amount
    }

    fn get_zone(&mut self, zone: Zone) -> &mut dyn ZoneControl {
        match zone {
            Zone::All => panic!("a card cannot be in all zones"),
            Zone::MainDeck => &mut self.main_deck,
            Zone::MainStageOshi => &mut self.oshi,
            Zone::MainStageCenter => &mut self.main_center,
            Zone::MainStageCollab => &mut self.main_collab,
            Zone::BackStage => &mut self.back_stage,
            Zone::Life => &mut self.life,
            Zone::CheerDeck => &mut self.cheer_deck,
            Zone::HoloPower => &mut self.holo_power,
            Zone::Archive => &mut self.archive,
            Zone::Hand => &mut self.hand,
        }
    }

    pub fn find_card_zone(&self, uuid: CardUuid) -> Option<Zone> {
        if self.main_deck.is_in_zone(uuid) {
            Some(Zone::MainDeck)
        } else if self.oshi.is_in_zone(uuid) {
            Some(Zone::MainStageOshi)
        } else if self.main_center.is_in_zone(uuid) {
            Some(Zone::MainStageCenter)
        } else if self.main_collab.is_in_zone(uuid) {
            Some(Zone::MainStageCollab)
        } else if self.back_stage.is_in_zone(uuid) {
            Some(Zone::BackStage)
        } else if self.life.is_in_zone(uuid) {
            Some(Zone::Life)
        } else if self.cheer_deck.is_in_zone(uuid) {
            Some(Zone::CheerDeck)
        } else if self.holo_power.is_in_zone(uuid) {
            Some(Zone::HoloPower)
        } else if self.archive.is_in_zone(uuid) {
            Some(Zone::Archive)
        } else if self.hand.is_in_zone(uuid) {
            Some(Zone::Hand)
        } else {
            None
        }
    }
}

trait ZoneControl {
    fn count(&self) -> usize;
    fn peek_top_card(&self) -> Option<CardUuid>;
    fn remove_card(&mut self, uuid: CardUuid);
    fn add_top_card(&mut self, uuid: CardUuid);
    fn add_bottom_card(&mut self, uuid: CardUuid);
    fn is_in_zone(&self, uuid: CardUuid) -> bool;
}

impl ZoneControl for Option<CardUuid> {
    fn count(&self) -> usize {
        if self.is_some() {
            1
        } else {
            0
        }
    }

    fn peek_top_card(&self) -> Option<CardUuid> {
        *self
    }

    fn remove_card(&mut self, uuid: CardUuid) {
        if self.is_in_zone(uuid) {
            self.take();
        }
    }

    fn add_top_card(&mut self, uuid: CardUuid) {
        if !self.is_in_zone(uuid) {
            self.replace(uuid);
        } else {
            panic!("there is already a card in this zone");
        }
    }

    fn add_bottom_card(&mut self, uuid: CardUuid) {
        self.add_top_card(uuid)
    }

    fn is_in_zone(&self, uuid: CardUuid) -> bool {
        *self == Some(uuid)
    }
}

impl ZoneControl for Vec<CardUuid> {
    fn count(&self) -> usize {
        self.len()
    }

    fn peek_top_card(&self) -> Option<CardUuid> {
        self.first().copied()
    }

    fn remove_card(&mut self, uuid: CardUuid) {
        if let Some(index) = self.iter().position(|c| *c == uuid) {
            self.remove(index);
        }
    }

    fn add_top_card(&mut self, uuid: CardUuid) {
        if !self.is_in_zone(uuid) {
            self.insert(0, uuid);
        } else {
            panic!("there is already a card in this zone");
        }
    }

    fn add_bottom_card(&mut self, uuid: CardUuid) {
        if !self.is_in_zone(uuid) {
            self.push(uuid);
        } else {
            panic!("there is already a card in this zone");
        }
    }

    fn is_in_zone(&self, uuid: CardUuid) -> bool {
        self.iter().any(|c| *c == uuid)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Zone {
    All,
    MainDeck,
    // MainStage,
    MainStageOshi,
    MainStageCenter,
    MainStageCollab,
    BackStage,
    Life,
    CheerDeck,
    HoloPower,
    Archive,
    Hand,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Phase {
    Setup,
    Refresh,
    Draw,
    Cheer,
    Main,
    Art,
    End,
}

#[derive(Debug)]
pub struct Buff {}

#[derive(Debug)]
pub struct DefaultPrompter {}

impl Prompter for DefaultPrompter {}

pub trait Prompter: Debug {
    // fn prompt_rps_choice();
    // fn prompt_mulligan_choice();
    // fn prompt_card_in_hand_choice();
    // fn prompt_card_on_stage_choice();
    // fn prompt_zone_choice();
    // fn prompt_main_phase_action_choice();
    //     // place debut member on backstage
    //     // bloom member (evolve e.g. debut -> 1st )
    //     // use support card
    //     // put backstage member in collab
    //     // retreat switch (baton pass)
    //     // use abilities (including oshi)
    // fn prompt_ability_choice();
    // fn prompt_attack_choice();
}
