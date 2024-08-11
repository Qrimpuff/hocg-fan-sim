use std::fmt::Display;
use std::num::NonZeroU16;
use std::sync::atomic::AtomicU8;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

use crate::evaluate::{EvaluateContext, EvaluateEffect};
use crate::events::EventSpan;
use crate::{Condition, Trigger};

use super::cards::*;
use super::modifiers::*;
use iter_tools::Itertools;
use rand::seq::IteratorRandom;
use rand::{thread_rng, Rng};
use tracing::debug;
use ModifierKind::*;

pub const STARTING_HAND_SIZE: usize = 7;
pub const MAX_MEMBERS_ON_STAGE: usize = 6;

pub static PRIVATE_CARD: CardRef = CardRef(NonZeroU16::MAX);
static NEXT_P1_CARD_REF: AtomicU8 = AtomicU8::new(1);
static NEXT_P2_CARD_REF: AtomicU8 = AtomicU8::new(1);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardRef(pub(crate) NonZeroU16);

impl Debug for CardRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "c_{:04x}", self.0)
    }
}
impl Display for CardRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub fn register_card(
    player: Player,
    card_type_id: u16,
    card_number: &CardNumber,
    card_map: &mut HashMap<CardRef, (Player, CardNumber)>,
) -> CardRef {
    let next_ref = match player {
        Player::One => NEXT_P1_CARD_REF.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        Player::Two => NEXT_P2_CARD_REF.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        Player::Both => todo!(),
    } as u16;
    let card = CardRef(
        NonZeroU16::new((next_ref << 8) + (card_type_id << 4) + (player as u16) + 1)
            .expect("that plus one makes it non zero"),
    );
    card_map.insert(card, (player, card_number.clone()));
    card
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Player {
    One,
    Two,
    Both,
}

impl Player {
    pub fn opponent(self) -> Player {
        match self {
            Player::One => Player::Two,
            Player::Two => Player::One,
            Player::Both => panic!("both players is not valid"),
        }
    }
}

pub type GameResult = Result<GameContinue, GameOutcome>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GameContinue;
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GameOutcome {
    pub winning_player: Option<Player>,
    pub reason: GameOverReason,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GameOverReason {
    MulliganToZeroCards,
    EmptyDeckInDrawStep,
    EmptyStage,
    EmptyLife,
    Draw,
}

#[derive(Debug)]
pub struct Game {
    pub game_outcome: Option<GameOutcome>,
    pub library: Arc<GlobalLibrary>,
    pub card_map: HashMap<CardRef, (Player, CardNumber)>, // TODO use a different pair because rarity is not include in card number
    pub player_1: GameBoard,
    pub player_2: GameBoard,
    pub active_player: Player,
    pub active_step: Step,
    pub turn_number: u8,
    pub zone_modifiers: HashMap<Player, Vec<(Zone, Modifier)>>,
    pub card_modifiers: HashMap<CardRef, Vec<Modifier>>,
    pub card_damage_markers: HashMap<CardRef, DamageMarkers>,
    pub prompter: RandomPrompter, // will probably be replace by 2 player send/receive channels
    pub event_span: EventSpan,
}

impl Game {
    pub fn setup(
        library: Arc<GlobalLibrary>,
        player_1: &Loadout,
        player_2: &Loadout,
        prompter: RandomPrompter,
    ) -> Game {
        let mut card_map = HashMap::new();
        Game {
            game_outcome: None,
            library,
            player_1: GameBoard::setup(Player::One, player_1, &mut card_map),
            player_2: GameBoard::setup(Player::Two, player_2, &mut card_map),
            card_map,
            active_player: Player::One,
            active_step: Step::Setup,
            turn_number: 0,
            zone_modifiers: HashMap::new(),
            card_modifiers: HashMap::new(),
            card_damage_markers: HashMap::new(),
            prompter,
            event_span: EventSpan::new(),
        }
    }

    pub fn active_board(&self) -> &GameBoard {
        self.board(self.active_player)
    }
    pub fn active_board_mut(&mut self) -> &mut GameBoard {
        self.board_mut(self.active_player)
    }
    pub fn board(&self, player: Player) -> &GameBoard {
        match player {
            Player::One => &self.player_1,
            Player::Two => &self.player_2,
            _ => unreachable!("both players cannot be active at the same time"),
        }
    }
    pub fn board_mut(&mut self, player: Player) -> &mut GameBoard {
        match player {
            Player::One => &mut self.player_1,
            Player::Two => &mut self.player_2,
            _ => unreachable!("both players cannot be active at the same time"),
        }
    }

    pub fn player_for_card(&self, card: CardRef) -> Player {
        self.card_map
            .get(&card)
            .expect("the card should be registered")
            .0
    }
    pub fn board_for_card(&self, card: CardRef) -> &GameBoard {
        let player = self.player_for_card(card);
        self.board(player)
    }
    pub fn board_for_card_mut(&mut self, card: CardRef) -> &mut GameBoard {
        let player = self.player_for_card(card);
        self.board_mut(player)
    }

    pub fn group_by_player_and_zone(
        &self,
        cards: Vec<CardRef>,
    ) -> HashMap<(Player, Zone), Vec<CardRef>> {
        let mut map: HashMap<_, Vec<_>> = HashMap::new();
        for card in cards {
            let player = self.player_for_card(card);
            let zone = self
                .board(player)
                .find_card_zone(card)
                .expect("card should be in zone");
            map.entry((player, zone)).or_default().push(card);
        }
        map
    }

    pub fn need_mulligan(&self, player: &GameBoard) -> bool {
        // need to have at least one debut member to place on the stage
        !player
            .hand()
            .filter_map(|c| self.lookup_holo_member(c))
            .any(|m| m.level == HoloMemberLevel::Debut)
    }

    pub fn handle_mulligan(&mut self, player: Player) -> GameResult {
        // - draw 7 cards from main deck
        let mut player_draw = STARTING_HAND_SIZE;
        println!("player {player:?} draws {player_draw}");
        self.draw_from_main_deck(None, player, player_draw)?;

        //   - can mulligan once for 7 cards, then any forced is -1
        //     - at 0 lose the game
        loop {
            let voluntary = player_draw == STARTING_HAND_SIZE;
            let force_mulligan = self.need_mulligan(self.board(player));
            println!("prompt mulligan: {player:?}");
            let mulligan = force_mulligan || voluntary && self.prompt_for_mulligan();
            if !mulligan {
                break;
            }

            println!(
                "player {player:?} mulligan{}",
                if force_mulligan { " by force" } else { "" }
            );

            if force_mulligan && !voluntary {
                // reveal hand
                self.reveal_all_cards_in_zone(None, player, Zone::Hand)?;
            }

            self.send_full_hand_to_main_deck(None, player)?;
            self.shuffle_main_deck(None, player)?;

            println!("player {player:?} draws {player_draw}");
            self.draw_from_main_deck(None, player, player_draw)?;

            player_draw -= 1;
            if player_draw == 0 {
                break;
            }
        }

        if player_draw == 0 {
            self.active_player = player;
            println!("player {player:?} cannot draw anymore card");
            self.lose_game(GameOverReason::MulliganToZeroCards)?;
        }

        Ok(GameContinue)
    }

    pub fn start_game(&mut self) -> GameResult {
        debug!("card_map: {:?}", self.card_map);

        // - game setup
        self.setup_game(None)?;

        // - game start
        self.report_start_game(None, self.active_player)?;

        Ok(GameContinue)
    }

    pub fn next_step(&mut self) -> GameResult {
        if let Some(game_outcome) = self.game_outcome {
            return Err(game_outcome);
        }

        self.active_step = match self.active_step {
            Step::Setup => Step::Reset,
            Step::Reset => Step::Draw,
            Step::Draw => Step::Cheer,
            Step::Cheer => Step::Main,
            Step::Main => Step::Performance,
            Step::Performance => Step::End,
            Step::End => {
                self.active_player = match self.active_player {
                    Player::One => Player::Two,
                    Player::Two => Player::One,
                    _ => unreachable!("both players cannot be active at the same time"),
                };
                Step::Reset
            }
            Step::GameOver => {
                // already returned above
                return Err(self.game_outcome.expect("there should be an outcome"));
            }
        };

        // start turn
        if self.active_step == Step::Reset {
            self.start_turn()?;
        }

        // skip the current step, used to skip the first performance of the game
        if self.player_has_modifier(self.active_player, SkipStep(self.active_step)) {
            // don't skip end turn
            if self.active_step == Step::End {
                return self.end_turn();
            } else {
                return Ok(GameContinue);
            }
        }

        println!("- active step: {:?}", self.active_step);
        self.report_enter_step(None, self.active_player, self.active_step)?;

        match self.active_step {
            Step::Setup => panic!("should not setup more than once"),
            Step::Reset => self.reset_step(),
            Step::Draw => self.draw_step(),
            Step::Cheer => self.cheer_step(),
            Step::Main => self.main_step(),
            Step::Performance => self.performance_step(),
            Step::End => self.end_step(),
            Step::GameOver => {
                // already returned above
                return Err(self.game_outcome.expect("there should be an outcome"));
            }
        }?;

        self.report_exit_step(None, self.active_player, self.active_step)?;

        // end turn
        if self.active_step == Step::End {
            self.end_turn()?;
        }

        Ok(GameContinue)
    }

    pub fn start_turn(&mut self) -> GameResult {
        self.turn_number += 1;

        println!("active player: {:?}", self.active_player);
        self.report_start_turn(None, self.active_player)?;

        Ok(GameContinue)
    }

    pub fn end_turn(&mut self) -> GameResult {
        self.report_end_turn(None, self.active_player)?;

        Ok(GameContinue)
    }

    pub fn reset_step(&mut self) -> GameResult {
        // - all members from rest to active
        for mem in self.active_board().stage().collect_vec() {
            self.remove_all_modifiers(None, mem, Resting)?;
        }

        // - collab to back stage in rest
        if let Some(mem) = self.active_board().collab {
            self.add_modifier(None, mem, Resting, LifeTime::UntilRemoved)?;
            self.send_from_collab_to_back_stage(None, self.active_player, mem)?;
        }
        // - if no center, back stage to center
        if self.active_board().center_stage.is_none() && !self.active_board().back_stage.is_empty()
        {
            println!("prompt new center member");
            // TODO request (intent)
            let back = self.prompt_for_back_stage_to_center(self.active_player, false);
            self.send_from_back_stage_to_center_stage(None, self.active_player, back)?;
        }

        Ok(GameContinue)
    }

    pub fn draw_step(&mut self) -> GameResult {
        // - deck is 0 on draw step
        if self.active_board().main_deck.count() == 0 {
            println!(
                "player {:?} has no card in their main deck",
                self.active_player
            );
            return self.lose_game(GameOverReason::EmptyDeckInDrawStep);
        }

        // - draw 1 card from main deck
        self.draw_from_main_deck(None, self.active_player, 1)?;

        Ok(GameContinue)
    }

    pub fn cheer_step(&mut self) -> GameResult {
        // - draw 1 card from cheer deck, attach it
        // TODO request (intent) select member
        self.attach_cheers_from_zone(None, self.active_player, Zone::CheerDeck, 1)?;

        Ok(GameContinue)
    }

    pub fn main_step(&mut self) -> GameResult {
        loop {
            println!("prompt main step action");
            println!("{} cards in hand", self.active_board().hand().count());

            // TODO request (intent) main action, all possible actions
            let action = self.prompt_for_main_action(self.active_player);
            debug!("ACTION = {action:?}");
            match action {
                MainStepAction::BackStageMember(card) => {
                    println!("- action: Back stage member");
                    // - place debut member on back stage
                    self.send_from_hand_to_back_stage(None, self.active_player, vec![card])?;

                    // cannot bloom member you just played
                    self.add_modifier(None, card, PreventBloom, LifeTime::ThisTurn)?;

                    // TODO maybe register for any abilities that could trigger?
                    // TODO remove the registration once they leave the board?
                }
                MainStepAction::BloomMember(bloom) => {
                    println!("- action: Bloom member");
                    // - bloom member (evolve e.g. debut -> 1st )
                    //   - bloom effect
                    //   - can't bloom on same turn as placed
                    // TODO request bloom target (intent)
                    let card = self.prompt_for_bloom(self.active_player, bloom);
                    self.bloom_holo_member(None, self.active_player, bloom, card)?;
                }
                MainStepAction::UseSupportCard(card) => {
                    println!("- action: Use support card");
                    // - use support card
                    //   - only one limited per turn
                    //   - otherwise unlimited
                    self.use_support_card(None, self.active_player, card)?;
                }
                MainStepAction::CollabMember(card) => {
                    println!("- action: Collab member");
                    // - put back stage member in collab
                    //   - can be done on first turn?
                    //   - draw down card from deck into power zone
                    self.send_from_back_stage_to_collab(None, self.active_player, card)?;
                }
                MainStepAction::BatonPass(card) => {
                    println!("- action: Baton pass");
                    // TODO verify baton pass action

                    // - retreat switch (baton pass)
                    //   - switch center with back stage
                    //   - remove attached cheer for cost
                    let center = self
                        .active_board()
                        .center_stage
                        .expect("there should always be a card in the center");
                    // only center stage can baton pass
                    assert_eq!(center, card);
                    // TODO request (intent) select back stage member
                    let back = self.prompt_for_back_stage_to_center(self.active_player, true);
                    // swap members
                    self.baton_pass_center_stage_to_back_stage(
                        None,
                        self.active_player,
                        center,
                        back,
                    )?;
                }
                MainStepAction::UseOshiSkill(card, i) => {
                    println!("- action: Use skill");
                    // - use oshi skill
                    //   - oshi power uses card in power zone
                    //   - once per turn / once per game
                    self.use_oshi_skill(None, self.active_player, card, i)?;
                }
                MainStepAction::Done => {
                    println!("- action: Done");
                    break;
                }
            }
        }

        // attack can be preloaded at this point

        Ok(GameContinue)
    }

    pub fn performance_step(&mut self) -> GameResult {
        // TODO request (intent) select art
        // TODO request (intent) select target, can be preloaded
        // TODO or request (intent) perform art action, all possible actions

        // TODO have art step actions, similar to main step
        // that way the player can choose the attack order. also allow to skip

        // - can use 2 attacks (center, collab)
        // - can choose target (center, collab)
        // - need required attached cheers to attack
        // - apply damage and effects
        // - remove member if defeated
        //   - lose 1 life
        //   - attach lost life (cheer)
        let op = self.active_player.opponent();
        let main_stage: Vec<_> = self.active_board().main_stage().collect();
        for card in main_stage {
            if self.board(op).main_stage().count() < 1 {
                println!("no more member to target");
                continue;
            }

            if let Some(art_idx) = self.prompt_for_art(card) {
                let target = self.prompt_for_art_target(op);

                self.perform_art(None, self.active_player, card, art_idx, Some(target))?;
            }
        }

        Ok(GameContinue)
    }

    pub fn end_step(&mut self) -> GameResult {
        // - any end step effect

        // - if no center, back stage to center
        if self.active_board().center_stage.is_none() && !self.active_board().back_stage.is_empty()
        {
            println!("prompt new center member");
            // TODO request (intent)
            let back = self.prompt_for_back_stage_to_center(self.active_player, false);
            self.send_from_back_stage_to_center_stage(None, self.active_player, back)?;
        }

        Ok(GameContinue)
    }

    pub fn win_game(&mut self, reason: GameOverReason) -> GameResult {
        match self.active_player {
            Player::One => println!("player 1 wins"),
            Player::Two => println!("player 2 wins"),
            _ => unreachable!("both players cannot be active at the same time"),
        };
        // stop the game
        let game_outcome = GameOutcome {
            winning_player: Some(self.active_player),
            reason,
        };
        self.report_game_over(None, game_outcome)?;

        Err(game_outcome)
    }
    pub fn lose_game(&mut self, reason: GameOverReason) -> GameResult {
        self.active_player = match self.active_player {
            Player::One => Player::Two,
            Player::Two => Player::One,
            _ => unreachable!("both players cannot be active at the same time"),
        };
        self.win_game(reason)
    }

    pub fn check_loss_conditions(&mut self) -> GameResult {
        // cannot lose in setup, except from mulligan
        if self.active_step == Step::Setup {
            return Ok(GameContinue);
        }

        let mut loss = None;
        for (player, board) in [(Player::One, &self.player_1), (Player::Two, &self.player_2)] {
            // - life is 0
            if board.life.count() == 0 {
                println!("player {player:?} has no life remaining");
                loss = Some((player, GameOverReason::EmptyLife));
            }

            // - 0 member in play
            if board.main_stage().count() + board.back_stage().count() == 0 {
                println!("player {player:?} has no more members on stage");
                loss = Some((player, GameOverReason::EmptyStage));
            }

            if loss.is_some() {
                break;
            }
        }

        if let Some(lose_player) = loss {
            self.active_player = lose_player.0;
            return self.lose_game(lose_player.1);
        }

        Ok(GameContinue)
    }

    pub fn lookup_card_number(&self, card: CardRef) -> &CardNumber {
        let (_, card_number) = self.card_map.get(&card).expect("should be in the map");
        card_number
    }
    pub fn lookup_card(&self, card: CardRef) -> &Card {
        let card_number = self.lookup_card_number(card);
        self.library
            .lookup_card(card_number)
            .expect("should be in the library")
    }
    pub fn lookup_oshi(&self, card: CardRef) -> Option<&OshiHoloMemberCard> {
        if let Card::OshiHoloMember(o) = self.lookup_card(card) {
            Some(o)
        } else {
            None
        }
    }
    pub fn lookup_holo_member(&self, card: CardRef) -> Option<&HoloMemberCard> {
        if let Card::HoloMember(m) = self.lookup_card(card) {
            Some(m)
        } else {
            None
        }
    }
    pub fn lookup_support(&self, card: CardRef) -> Option<&SupportCard> {
        if let Card::Support(s) = self.lookup_card(card) {
            Some(s)
        } else {
            None
        }
    }
    pub fn lookup_cheer(&self, card: CardRef) -> Option<&CheerCard> {
        if let Card::Cheer(c) = self.lookup_card(card) {
            Some(c)
        } else {
            None
        }
    }

    pub fn attached_cheers(&self, card: CardRef) -> impl Iterator<Item = CardRef> + '_ {
        self.board_for_card(card)
            .attachments(card)
            .into_iter()
            .filter(|a| self.lookup_cheer(*a).is_some())
    }

    pub fn required_attached_cheers(&self, card: CardRef, cheers: &[Color]) -> bool {
        // TODO modify if there is ever a double cheer
        // count the cheers
        let mut required = cheers.iter().fold(HashMap::new(), |mut acc, c| {
            *acc.entry(c).or_insert(0) += 1;
            acc
        });

        // remove the required cheers
        for at_cheer in self
            .attached_cheers(card)
            .filter_map(|c| self.lookup_cheer(c))
        {
            if let Some(v) = required.get_mut(&at_cheer.color) {
                *v -= 1;
                if *v == 0 {
                    required.remove(&at_cheer.color);
                }
            } else if let Some(v) = required.get_mut(&Color::ColorLess) {
                *v -= 1;
                if *v == 0 {
                    required.remove(&Color::ColorLess);
                }
            }
        }

        // removed all the required cheers
        required.is_empty()
    }

    pub fn prompt_for_rps(&mut self) -> Rps {
        self.prompter.prompt_choice(
            "choose rock, paper or scissor:",
            vec![Rps::Rock, Rps::Paper, Rps::Scissor],
        )
    }

    pub fn prompt_for_mulligan(&mut self) -> bool {
        self.prompter
            .prompt_choice("do you want to mulligan?", vec!["Yes", "No"])
            == "Yes"
    }

    pub fn prompt_for_first_debut(&mut self, player: Player) -> CardRef {
        // TODO extract that filtering to a reusable function
        let debuts: Vec<_> = self
            .board(player)
            .hand()
            .filter_map(|c| self.lookup_holo_member(c).map(|m| (c, m)))
            .filter(|(_, m)| m.level == HoloMemberLevel::Debut)
            .map(|(c, _)| CardDisplay::new(c, self))
            .collect();

        assert!(!debuts.is_empty());
        self.prompter
            .prompt_choice("choose first debut:", debuts)
            .card
    }

    pub fn prompt_for_first_back_stage(&mut self, player: Player) -> Vec<CardRef> {
        // TODO extract that filtering to a reusable function
        let debuts: Vec<_> = self
            .board(player)
            .hand()
            .filter_map(|c| self.lookup_holo_member(c).map(|m| (c, m)))
            .filter(|(_, m)| m.level == HoloMemberLevel::Debut || m.level == HoloMemberLevel::Spot)
            .map(|(c, _)| CardDisplay::new(c, self))
            .collect();

        if !debuts.is_empty() {
            self.prompter
                .prompt_multi_choices(
                    "choose first back stage:",
                    debuts,
                    0,
                    MAX_MEMBERS_ON_STAGE - 1,
                )
                .into_iter()
                .map(|c| c.card)
                .collect()
        } else {
            vec![]
        }
    }

    pub fn prompt_for_back_stage_to_center(&mut self, player: Player, baton_pass: bool) -> CardRef {
        // TODO extract that filtering to a reusable function
        let mut back = self
            .board(player)
            .back_stage()
            .filter(|b| !self.has_modifier(*b, Resting))
            .filter_map(|c| self.lookup_holo_member(c).map(|m| (c, m)))
            .collect_vec();

        // if there are only resting members, select one of them
        // baton pass cannot be resting
        if back.is_empty() && !baton_pass {
            back = self
                .board(player)
                .back_stage()
                .filter_map(|c| self.lookup_holo_member(c).map(|m| (c, m)))
                .collect_vec();
        }

        let back = back
            .into_iter()
            .map(|(c, _)| CardDisplay::new(c, self))
            .collect_vec();

        assert!(!back.is_empty());
        self.prompter
            .prompt_choice("choose send to center stage:", back)
            .card
    }

    pub fn prompt_for_cheer(&mut self, player: Player) -> Option<CardRef> {
        // TODO extract that filtering to a reusable function
        let mems: Vec<_> = self
            .board(player)
            .stage()
            .filter_map(|c| self.lookup_holo_member(c).map(|m| (c, m)))
            .map(|(c, _)| CardDisplay::new(c, self))
            .collect();

        if !mems.is_empty() {
            Some(
                self.prompter
                    .prompt_choice("choose receive cheer:", mems)
                    .card,
            )
        } else {
            None
        }
    }

    pub fn prompt_for_main_action(&mut self, player: Player) -> MainStepAction {
        // actions from hand
        let mut actions: Vec<_> = self
            .board(player)
            .hand()
            .filter_map(|c| match self.lookup_card(c) {
                Card::OshiHoloMember(_) => unreachable!("oshi cannot be in hand"),
                Card::HoloMember(m) => match m.level {
                    HoloMemberLevel::Debut | HoloMemberLevel::Spot => {
                        // check condition for back stage
                        let count = self
                            .board(player)
                            .stage()
                            .filter_map(|c| self.lookup_holo_member(c))
                            .count();
                        (count < MAX_MEMBERS_ON_STAGE).then_some(MainStepAction::BackStageMember(c))
                    }
                    HoloMemberLevel::First | HoloMemberLevel::Second => {
                        let can_bloom = self
                            .board(player)
                            .stage()
                            .filter_map(|s| self.lookup_holo_member(s).map(|m| (s, m)))
                            .any(|target| m.can_bloom_target(c, self, target));
                        can_bloom.then_some(MainStepAction::BloomMember(c))
                    }
                },
                // check condition to play support
                Card::Support(s) => s
                    .can_use_support(c, self)
                    .then_some(MainStepAction::UseSupportCard(c)),
                Card::Cheer(_) => unreachable!("cheer cannot be in hand"),
            })
            .map(|a| MainStepActionDisplay::new(a, self))
            .collect();

        // actions from board
        // collab
        actions.extend(
            self.board(player)
                .back_stage()
                // check condition for collab
                .filter(|c| !self.has_modifier(*c, Resting))
                .filter(|c| !self.has_modifier(*c, PreventCollab))
                .filter_map(|c| match self.lookup_card(c) {
                    Card::OshiHoloMember(_) => unreachable!("oshi cannot be in the back stage"),
                    Card::HoloMember(_) => self
                        .board(player)
                        .collab
                        .is_none()
                        .then_some(MainStepAction::CollabMember(c)),
                    Card::Support(_) => unreachable!("support cannot be in the back stage"),
                    Card::Cheer(_) => unreachable!("cheer cannot be in the back stage"),
                })
                .map(|a| MainStepActionDisplay::new(a, self)),
        );
        // baton pass
        actions.extend(
            self.board(player)
                .center_stage()
                .filter_map(|c| match self.lookup_card(c) {
                    Card::OshiHoloMember(_) => {
                        unreachable!("oshi cannot be in the center stage")
                    }
                    Card::HoloMember(m) => m
                        .can_baton_pass(c, self)
                        .then_some(MainStepAction::BatonPass(c)),
                    Card::Support(_) => unreachable!("support cannot be in the center stage"),
                    Card::Cheer(_) => unreachable!("cheer cannot be in the center stage"),
                })
                .map(|a| MainStepActionDisplay::new(a, self)),
        );
        // skills
        actions.extend(
            self.board(player)
                .oshi
                .iter()
                .flat_map(|c| match self.lookup_card(*c) {
                    Card::OshiHoloMember(o) => o
                        .skills
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| {
                            s.triggers.iter().any(|t| *t == Trigger::ActivateInMainStep)
                        })
                        .filter(|(i, _)| o.can_use_skill(*c, *i, self))
                        .map(|(i, _)| MainStepAction::UseOshiSkill(*c, i))
                        .collect_vec(),
                    Card::HoloMember(_) => todo!("members are not in oshi position"),
                    Card::Support(_) => todo!("supports are not in oshi position"),
                    Card::Cheer(_) => todo!("cheers are not in oshi position"),
                })
                .map(|a| MainStepActionDisplay::new(a, self)),
        );

        actions.push(MainStepActionDisplay::new(MainStepAction::Done, self));
        actions.sort();

        assert!(!actions.is_empty());
        self.prompter
            .prompt_choice("main step action:", actions)
            .action
    }

    pub fn prompt_for_bloom(&mut self, player: Player, card: CardRef) -> CardRef {
        let bloom = self
            .lookup_holo_member(card)
            .expect("can only bloom from member");

        let stage: Vec<_> = self
            .board(player)
            .stage()
            .filter_map(|c| self.lookup_holo_member(c).map(|m| (c, m)))
            .filter(|target| bloom.can_bloom_target(card, self, *target))
            .map(|(c, _)| CardDisplay::new(c, self))
            .collect();

        assert!(!stage.is_empty());
        self.prompter.prompt_choice("choose for bloom:", stage).card
    }

    pub fn prompt_for_baton_pass(
        &mut self,
        card: CardRef,
        cost: HoloMemberBatonPassCost,
    ) -> Vec<CardRef> {
        // TODO extract that filtering to a reusable function
        let cheers: Vec<_> = self
            .attached_cheers(card)
            .map(|c| CardDisplay::new(c, self))
            .collect();

        if !cheers.is_empty() {
            self.prompter
                .prompt_multi_choices("choose cheers to remove:", cheers, cost.into(), cost.into())
                .into_iter()
                .map(|c| c.card)
                .collect()
        } else {
            panic!("baton pass should not be an option, if there is no cheers")
        }
    }

    pub fn prompt_for_art(&mut self, card: CardRef) -> Option<usize> {
        if let Some(mem) = self.lookup_holo_member(card) {
            let arts: Vec<_> = mem
                .arts
                .iter()
                .enumerate()
                .filter(|(i, _)| mem.can_use_art(card, *i, self))
                .map(|(i, _)| ArtDisplay::new(card, i, self))
                .collect();
            // TODO add skip art, it's not required to use an art

            if !arts.is_empty() {
                Some(self.prompter.prompt_choice("choose for art:", arts).idx)
            } else {
                None
            }
        } else {
            panic!("only members can have arts")
        }
    }

    pub fn prompt_for_art_target(&mut self, player: Player) -> CardRef {
        // TODO extract that filtering to a reusable function
        let targets: Vec<_> = self
            .board(player)
            .main_stage()
            .filter_map(|c| self.lookup_holo_member(c).map(|m| (c, m)))
            // TODO check for rest?
            .map(|(c, _)| CardDisplay::new(c, self))
            .collect();

        assert!(!targets.is_empty());
        self.prompter
            .prompt_choice("choose target for art:", targets)
            .card
    }

    pub fn prompt_for_select(
        &mut self,
        cards: Vec<CardRef>,
        condition: Condition,
        ctx: &EvaluateContext,
        min: usize,
        max: usize,
    ) -> Vec<CardRef> {
        let choices: Vec<_> = cards
            .into_iter()
            .filter(|c| {
                let cond = condition.evaluate_with_context(&ctx.for_card(*c), self);
                if !cond {
                    println!("viewed: {}", CardDisplay::new(*c, self))
                }
                cond
            })
            .map(|c| CardDisplay::new(c, self))
            .collect();

        if !choices.is_empty() {
            self.prompter
                .prompt_multi_choices("choose cards:", choices, min, max)
                .into_iter()
                .map(|c| c.card)
                .collect()
        } else {
            vec![]
        }
    }

    pub fn prompt_for_optional_activate(&mut self) -> bool {
        self.prompter
            .prompt_choice("do you want to activate the effect?", vec!["Yes", "No"])
            == "Yes"
    }

    pub fn prompt_for_number(&mut self, min: usize, max: usize) -> usize {
        self.prompter
            .prompt_choice("choose a number:", (min..=max).collect_vec())
    }
}

#[derive(Debug)]
pub struct GameBoard {
    oshi: Option<CardRef>,
    main_deck: Vec<CardRef>,
    center_stage: Option<CardRef>,
    collab: Option<CardRef>,
    back_stage: Vec<CardRef>,
    life: Vec<CardRef>,
    cheer_deck: Vec<CardRef>,
    holo_power: Vec<CardRef>,
    archive: Vec<CardRef>,
    hand: Vec<CardRef>,
    activate_support: Vec<CardRef>,
    attachments: HashMap<CardRef, CardRef>,
}

impl GameBoard {
    pub fn setup(
        player: Player,
        loadout: &Loadout,
        card_map: &mut HashMap<CardRef, (Player, CardNumber)>,
    ) -> GameBoard {
        GameBoard {
            oshi: Some(register_card(player, 0, &loadout.oshi, card_map)),
            main_deck: loadout
                .main_deck
                .iter()
                .map(|c| register_card(player, 1, c, card_map))
                .collect(),
            center_stage: None,
            collab: None,
            back_stage: Vec::new(),
            life: Vec::new(),
            cheer_deck: loadout
                .cheer_deck
                .iter()
                .map(|c| register_card(player, 2, c, card_map))
                .collect(),
            holo_power: Vec::new(),
            archive: Vec::new(),
            hand: Vec::new(),
            activate_support: Vec::new(),
            attachments: HashMap::new(),
        }
    }

    pub fn oshi(&self) -> Option<CardRef> {
        self.oshi
    }
    pub fn hand(&self) -> impl Iterator<Item = CardRef> + '_ {
        self.hand.iter().copied()
    }
    pub fn stage(&self) -> impl Iterator<Item = CardRef> + '_ {
        self.main_stage().chain(self.back_stage())
    }
    pub fn center_stage(&self) -> impl Iterator<Item = CardRef> + '_ {
        self.center_stage.iter().copied()
    }
    pub fn collab(&self) -> impl Iterator<Item = CardRef> + '_ {
        self.collab.iter().copied()
    }
    pub fn main_stage(&self) -> impl Iterator<Item = CardRef> + '_ {
        self.center_stage().chain(self.collab())
    }
    pub fn back_stage(&self) -> impl Iterator<Item = CardRef> + '_ {
        self.back_stage.iter().copied()
    }

    pub fn is_attached(&self, attachment: CardRef) -> bool {
        self.attachments.contains_key(&attachment)
    }
    pub fn is_attached_to(&self, attachment: CardRef, card: CardRef) -> bool {
        self.attachments.get(&attachment) == Some(&card)
    }
    pub fn attached_to(&self, attachment: CardRef) -> Option<CardRef> {
        self.attachments.get(&attachment).copied()
    }

    pub fn attach_to_card(&mut self, attachment: CardRef, card: CardRef) {
        let current_zone = self.find_card_zone(attachment);
        if let Some(zone) = current_zone {
            self.get_zone_mut(zone).remove_card(attachment);
        } else if self.attachments.contains_key(&card) {
            panic!("cannot attach to attachment");
        }

        self.attachments.insert(attachment, card);
    }

    pub fn remove_attachment(&mut self, attachment: CardRef) {
        self.send_to_zone(
            attachment,
            Zone::Archive,
            Zone::Archive.default_add_location(),
        );
    }
    pub fn remove_many_attachments(&mut self, attachments: impl IntoIterator<Item = CardRef>) {
        self.send_many_to_zone(
            attachments,
            Zone::Archive,
            Zone::Archive.default_add_location(),
        );
    }

    pub fn send_all_attachments_to_zone(
        &mut self,
        card: CardRef,
        target_zone: Zone,
        location: ZoneAddLocation,
    ) {
        let attached = self.attachments(card);
        self.send_many_to_zone(attached, target_zone, location)
    }

    /// Mainly used for Bloom
    pub fn promote_attachment(&mut self, attachment: CardRef, parent: CardRef) {
        if let Some(current_zone) = self.find_card_zone(parent) {
            // replace with attachment
            self.get_zone_mut(current_zone)
                .replace_card(parent, attachment);

            // remove from attachments
            self.attachments.remove(&attachment);

            // attach the parent
            self.attach_to_card(parent, attachment);

            // change the parent of other attachments
            for (_, v) in self.attachments.iter_mut().filter(|(_, v)| **v == parent) {
                *v = attachment;
            }
        }
    }

    pub fn attachments(&self, card: CardRef) -> Vec<CardRef> {
        self.attachments
            .iter()
            .filter_map(|(k, v)| if *v == card { Some(k) } else { None })
            .copied()
            .collect()
    }

    pub fn send_to_zone(&mut self, card: CardRef, target_zone: Zone, location: ZoneAddLocation) {
        let current_zone = self.find_card_zone(card);
        if let Some(zone) = current_zone {
            self.get_zone_mut(zone).remove_card(card);
            match location {
                ZoneAddLocation::Top => self.get_zone_mut(target_zone).add_top_card(card),
                ZoneAddLocation::Bottom => self.get_zone_mut(target_zone).add_bottom_card(card),
            }
        } else if self.is_attached(card) {
            self.attachments.remove(&card);
            match location {
                ZoneAddLocation::Top => self.get_zone_mut(target_zone).add_top_card(card),
                ZoneAddLocation::Bottom => self.get_zone_mut(target_zone).add_bottom_card(card),
            }
        }
    }

    pub fn send_many_to_zone(
        &mut self,
        cards: impl IntoIterator<Item = CardRef>,
        target_zone: Zone,
        location: ZoneAddLocation,
    ) {
        cards
            .into_iter()
            .for_each(|c| self.send_to_zone(c, target_zone, location));
    }

    pub fn send_from_zone(
        &mut self,
        current_zone: Zone,
        target_zone: Zone,
        location: ZoneAddLocation,
        amount: usize,
    ) {
        for _ in 0..amount {
            if let Some(card) = self.get_zone(current_zone).peek_top_card() {
                self.send_to_zone(card, target_zone, location);
            }
        }
    }

    pub fn send_all_from_zone(
        &mut self,
        current_zone: Zone,
        target_zone: Zone,
        location: ZoneAddLocation,
    ) -> usize {
        let amount = self.get_zone(current_zone).count();
        self.send_from_zone(current_zone, target_zone, location, amount);
        amount
    }

    pub fn all_cards(&self, zone: Zone) -> Vec<CardRef> {
        let cards = match zone {
            Zone::All => unreachable!("that unreasonable"),
            Zone::MainDeck => self.main_deck.iter().copied().collect_vec(),
            Zone::Oshi => self.oshi.iter().copied().collect_vec(),
            Zone::Stage => self.stage().collect_vec(),
            Zone::MainStage => self.main_stage().collect_vec(),
            Zone::CenterStage => self.center_stage.iter().copied().collect_vec(),
            Zone::Collab => self.collab.iter().copied().collect_vec(),
            Zone::BackStage => self.back_stage.iter().copied().collect_vec(),
            Zone::Life => self.life.iter().copied().collect_vec(),
            Zone::CheerDeck => self.cheer_deck.iter().copied().collect_vec(),
            Zone::HoloPower => self.holo_power.iter().copied().collect_vec(),
            Zone::Archive => self.archive.iter().copied().collect_vec(),
            Zone::Hand => self.hand.iter().copied().collect_vec(),
            Zone::ActivateSupport => self.activate_support.iter().copied().collect_vec(),
        };
        cards
    }

    pub fn get_zone(&self, zone: Zone) -> &dyn ZoneControl {
        match zone {
            Zone::All => unreachable!("cards cannot be in all zones"),
            Zone::MainDeck => &self.main_deck,
            Zone::Oshi => &self.oshi,
            Zone::Stage => unreachable!("cards will be in the specific zone"),
            Zone::MainStage => unreachable!("cards will be in the specific zone"),
            Zone::CenterStage => &self.center_stage,
            Zone::Collab => &self.collab,
            Zone::BackStage => &self.back_stage,
            Zone::Life => &self.life,
            Zone::CheerDeck => &self.cheer_deck,
            Zone::HoloPower => &self.holo_power,
            Zone::Archive => &self.archive,
            Zone::Hand => &self.hand,
            Zone::ActivateSupport => &self.activate_support,
        }
    }

    pub fn get_zone_mut(&mut self, zone: Zone) -> &mut dyn ZoneControl {
        match zone {
            Zone::All => unreachable!("cards cannot be in all zones"),
            Zone::MainDeck => &mut self.main_deck,
            Zone::Oshi => &mut self.oshi,
            Zone::Stage => unreachable!("cards will be in the specific zone"),
            Zone::MainStage => unreachable!("cards will be in the specific zone"),
            Zone::CenterStage => &mut self.center_stage,
            Zone::Collab => &mut self.collab,
            Zone::BackStage => &mut self.back_stage,
            Zone::Life => &mut self.life,
            Zone::CheerDeck => &mut self.cheer_deck,
            Zone::HoloPower => &mut self.holo_power,
            Zone::Archive => &mut self.archive,
            Zone::Hand => &mut self.hand,
            Zone::ActivateSupport => &mut self.activate_support,
        }
    }

    pub fn find_card_zone(&self, card: CardRef) -> Option<Zone> {
        if self.main_deck.is_in_zone(card) {
            Some(Zone::MainDeck)
        } else if self.oshi.is_in_zone(card) {
            Some(Zone::Oshi)
        } else if self.center_stage.is_in_zone(card) {
            Some(Zone::CenterStage)
        } else if self.collab.is_in_zone(card) {
            Some(Zone::Collab)
        } else if self.back_stage.is_in_zone(card) {
            Some(Zone::BackStage)
        } else if self.life.is_in_zone(card) {
            Some(Zone::Life)
        } else if self.cheer_deck.is_in_zone(card) {
            Some(Zone::CheerDeck)
        } else if self.holo_power.is_in_zone(card) {
            Some(Zone::HoloPower)
        } else if self.archive.is_in_zone(card) {
            Some(Zone::Archive)
        } else if self.hand.is_in_zone(card) {
            Some(Zone::Hand)
        } else if self.activate_support.is_in_zone(card) {
            Some(Zone::ActivateSupport)
        } else {
            None
        }
    }

    pub fn group_cards_by_zone(&self, cards: &[CardRef]) -> HashMap<Zone, Vec<CardRef>> {
        let group = cards
            .iter()
            .fold(HashMap::new(), |mut acc: HashMap<_, Vec<_>>, c| {
                let zone = self
                    .find_card_zone(*c)
                    .expect("the card should be in a zone");
                acc.entry(zone).or_default().push(*c);
                acc
            });
        group
    }
}

pub trait ZoneControl {
    fn count(&self) -> usize;
    fn peek_top_card(&self) -> Option<CardRef>;
    fn peek_top_cards(&self, amount: usize) -> Vec<CardRef>;
    fn all_cards(&self) -> Vec<CardRef>;
    fn remove_card(&mut self, card: CardRef);
    fn add_top_card(&mut self, card: CardRef);
    fn add_bottom_card(&mut self, card: CardRef);
    fn replace_card(&mut self, from_card: CardRef, to_card: CardRef);
    fn is_in_zone(&self, card: CardRef) -> bool;
    fn shuffle(&mut self);
}

impl ZoneControl for Option<CardRef> {
    fn count(&self) -> usize {
        if self.is_some() {
            1
        } else {
            0
        }
    }

    fn peek_top_card(&self) -> Option<CardRef> {
        *self
    }

    fn peek_top_cards(&self, amount: usize) -> Vec<CardRef> {
        self.iter().copied().take(amount).collect()
    }

    fn all_cards(&self) -> Vec<CardRef> {
        self.iter().copied().collect()
    }

    fn remove_card(&mut self, card: CardRef) {
        if self.is_in_zone(card) {
            self.take();
        }
    }

    fn add_top_card(&mut self, card: CardRef) {
        if self.is_none() {
            self.replace(card);
        } else {
            panic!("there is already a card in this zone");
        }
    }

    fn add_bottom_card(&mut self, card: CardRef) {
        self.add_top_card(card)
    }

    fn replace_card(&mut self, from_card: CardRef, to_card: CardRef) {
        if self.is_in_zone(from_card) {
            self.replace(to_card);
        } else {
            panic!("card is not already in this zone");
        }
    }

    fn is_in_zone(&self, card: CardRef) -> bool {
        *self == Some(card)
    }

    fn shuffle(&mut self) {
        // nothing to shuffle
    }
}

impl ZoneControl for Vec<CardRef> {
    fn count(&self) -> usize {
        self.len()
    }

    fn peek_top_card(&self) -> Option<CardRef> {
        self.first().copied()
    }

    fn peek_top_cards(&self, amount: usize) -> Vec<CardRef> {
        self.iter().copied().take(amount).collect()
    }

    fn all_cards(&self) -> Vec<CardRef> {
        self.to_vec()
    }

    fn remove_card(&mut self, card: CardRef) {
        if let Some(index) = self.iter().position(|c| *c == card) {
            self.remove(index);
        }
    }

    fn add_top_card(&mut self, card: CardRef) {
        if !self.is_in_zone(card) {
            self.insert(0, card);
        } else {
            panic!("there is already a card in this zone");
        }
    }

    fn add_bottom_card(&mut self, card: CardRef) {
        if !self.is_in_zone(card) {
            self.push(card);
        } else {
            panic!("there is already a card in this zone");
        }
    }

    fn replace_card(&mut self, from_card: CardRef, to_card: CardRef) {
        if let Some(index) = self.iter().position(|c| *c == from_card) {
            *self.get_mut(index).expect("card is already in the zone") = to_card;
        } else {
            panic!("card is not already in this zone");
        }
    }

    fn is_in_zone(&self, card: CardRef) -> bool {
        self.iter().any(|c| *c == card)
    }

    fn shuffle(&mut self) {
        rand::seq::SliceRandom::shuffle(&mut self[..], &mut rand::thread_rng());
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Zone {
    All,
    MainDeck,
    Oshi,
    Stage,     // contains MainStage and CenterStage
    MainStage, // contains CenterStage and Collab
    CenterStage,
    Collab,
    BackStage,
    Life,
    CheerDeck,
    HoloPower,
    Archive,
    Hand,
    ActivateSupport,
}

impl Zone {
    pub fn includes(&self, zone: Zone) -> bool {
        if *self == zone {
            return true;
        }
        match self {
            Zone::All => true,
            Zone::Stage => matches!(
                zone,
                Zone::MainStage | Zone::CenterStage | Zone::Collab | Zone::BackStage
            ),
            Zone::MainStage => matches!(zone, Zone::CenterStage | Zone::Collab),
            _ => false,
        }
    }

    pub fn default_add_location(&self) -> ZoneAddLocation {
        use ZoneAddLocation::*;
        match self {
            Zone::All => unreachable!("cannot add to all zone"),
            Zone::MainDeck => Bottom,
            Zone::Oshi => Top,
            Zone::Stage => unreachable!("cannot add to stage"),
            Zone::MainStage => unreachable!("cannot add to main stage"),
            Zone::CenterStage => Top,
            Zone::Collab => Top,
            Zone::BackStage => Bottom,
            Zone::Life => Top,
            Zone::CheerDeck => Bottom,
            Zone::HoloPower => Top,
            Zone::Archive => Top,
            Zone::Hand => Bottom,
            Zone::ActivateSupport => Top,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ZoneAddLocation {
    Top,
    Bottom,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Step {
    Setup,
    Reset,
    Draw,
    Cheer,
    Main,
    Performance,
    End,
    GameOver,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Rps {
    Rock,
    Paper,
    Scissor,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RpsOutcome {
    Win,
    Lose,
    Draw,
}

impl Rps {
    pub fn vs(&self, other: Rps) -> RpsOutcome {
        match (self, other) {
            (Rps::Rock, Rps::Rock) => RpsOutcome::Draw,
            (Rps::Rock, Rps::Paper) => RpsOutcome::Lose,
            (Rps::Rock, Rps::Scissor) => RpsOutcome::Win,
            (Rps::Paper, Rps::Rock) => RpsOutcome::Win,
            (Rps::Paper, Rps::Paper) => RpsOutcome::Draw,
            (Rps::Paper, Rps::Scissor) => RpsOutcome::Lose,
            (Rps::Scissor, Rps::Rock) => RpsOutcome::Lose,
            (Rps::Scissor, Rps::Paper) => RpsOutcome::Win,
            (Rps::Scissor, Rps::Scissor) => RpsOutcome::Draw,
        }
    }
}

impl Display for Rps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]

pub enum MainStepAction {
    BackStageMember(CardRef),
    BloomMember(CardRef),
    UseSupportCard(CardRef),
    CollabMember(CardRef),
    BatonPass(CardRef),
    UseOshiSkill(CardRef, usize),
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct MainStepActionDisplay {
    action: MainStepAction,
    text: String,
}

impl MainStepActionDisplay {
    pub fn new(action: MainStepAction, game: &Game) -> MainStepActionDisplay {
        let text = match action {
            MainStepAction::BackStageMember(card) => {
                let display = CardDisplay::new(card, game);
                format!("put on back stage: {display}")
            }
            MainStepAction::BloomMember(card) => {
                let display = CardDisplay::new(card, game);
                format!("bloom member with: {display}")
            }
            MainStepAction::UseSupportCard(card) => {
                let display = CardDisplay::new(card, game);
                format!("use support: {display}")
            }
            MainStepAction::CollabMember(card) => {
                let display = CardDisplay::new(card, game);
                format!("move to collab: {display}")
            }
            MainStepAction::BatonPass(card) => {
                let display = CardDisplay::new(card, game);
                format!("baton pass (retreat): {display}")
            }
            MainStepAction::UseOshiSkill(card, idx) => {
                let display = CardDisplay::new(card, game);
                let oshi = game.lookup_oshi(card).expect("it should be an oshi");
                format!(
                    "use skill: [-{}] {} - {display}",
                    oshi.skills[idx].cost, oshi.skills[idx].name
                )
            }
            MainStepAction::Done => "done".into(),
        };

        MainStepActionDisplay { action, text }
    }
}

impl Display for MainStepActionDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

#[derive(Debug)]
pub struct DefaultPrompter {}
impl DefaultPrompter {
    pub fn new() -> Self {
        DefaultPrompter {}
    }
}

impl Prompter for DefaultPrompter {
    fn prompt_choice<'a, T: ToString>(&mut self, text: &str, choices: Vec<T>) -> T {
        println!("choosing first choice for: {text}");
        self.print_choices(&choices);

        let c = choices
            .into_iter()
            .next()
            .expect("always at least one choice");
        println!("{}", c.to_string());
        c
    }

    fn prompt_multi_choices<'a, T: ToString>(
        &mut self,
        text: &str,
        choices: Vec<T>,
        min: usize,
        _max: usize,
    ) -> Vec<T> {
        println!("choosing first choices for: {text}");
        self.print_choices(&choices);

        let c: Vec<_> = choices.into_iter().take(min).collect();
        println!("{}", c.iter().map(T::to_string).collect_vec().join(", "));
        c
    }
}

#[derive(Debug)]
pub struct RandomPrompter {}
impl RandomPrompter {
    pub fn new() -> Self {
        RandomPrompter {}
    }
}

impl Prompter for RandomPrompter {
    fn prompt_choice<'a, T: ToString>(&mut self, text: &str, choices: Vec<T>) -> T {
        println!("choosing random choice for: {text}");
        self.print_choices(&choices);

        let c = choices
            .into_iter()
            .choose(&mut thread_rng())
            .expect("always at least one choice");
        println!("{}", c.to_string());
        c
    }

    fn prompt_multi_choices<'a, T: ToString>(
        &mut self,
        text: &str,
        choices: Vec<T>,
        min: usize,
        max: usize,
    ) -> Vec<T> {
        println!("choosing random choices for: {text}");
        self.print_choices(&choices);

        let max = max.min(choices.len());

        let c = choices
            .into_iter()
            .choose_multiple(&mut thread_rng(), thread_rng().gen_range(min..=max));
        println!("{}", c.iter().map(T::to_string).collect_vec().join(", "));
        c
    }
}

pub trait Prompter: Debug {
    fn prompt_choice<T: ToString>(&mut self, text: &str, choices: Vec<T>) -> T;
    fn prompt_multi_choices<T: ToString>(
        &mut self,
        text: &str,
        choices: Vec<T>,
        min: usize,
        max: usize,
    ) -> Vec<T>;

    fn print_choices<T: ToString>(&mut self, choices: &[T]) {
        println!(
            "options:\n{}",
            choices
                .iter()
                .map(|c| format!("  - {}", c.to_string()))
                .collect_vec()
                .join("\n")
        );
    }
    // fn prompt_rps_choice();
    // fn prompt_mulligan_choice();
    // fn prompt_card_in_hand_choice();
    // fn prompt_card_on_stage_choice();
    // fn prompt_zone_choice();
    // fn prompt_main_step_action_choice();
    //     // place debut member on back stage
    //     // bloom member (evolve e.g. debut -> 1st )
    //     // use support card
    //     // put back stage member in collab
    //     // retreat switch (baton pass)
    //     // use abilities (including oshi)
    // fn prompt_ability_choice();
    // fn prompt_attack_choice();
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CardDisplay {
    card: CardRef,
    text: String,
}

impl CardDisplay {
    pub fn new(card: CardRef, game: &Game) -> CardDisplay {
        let text = match game.lookup_card(card) {
            Card::OshiHoloMember(o) => {
                let life_remaining = game.board_for_card(card).life.count();
                format!(
                    "{} (Oshi) ({}/{} life) ({}) {}",
                    o.name, life_remaining, o.life, o.card_number, card,
                )
            }
            Card::HoloMember(m) => {
                format!(
                    "{} ({:?}) ({}/{}){} ({}) {}",
                    m.name,
                    m.level,
                    game.remaining_hp(card),
                    m.hp,
                    if game.attached_cheers(card).any(|_| true) {
                        format!(
                            " (cheers: {})",
                            game.attached_cheers(card)
                                .filter_map(|c| game.lookup_cheer(c))
                                .map(|c| format!("{:?}", c.color))
                                .collect_vec()
                                .join(", ")
                        )
                    } else {
                        "".into()
                    },
                    m.card_number,
                    card,
                )
            }
            Card::Support(s) => format!("{} ({:?}) ({}) {}", s.name, s.kind, s.card_number, card),
            Card::Cheer(c) => format!("{} ({:?}) ({}) {}", c.name, c.color, c.card_number, card),
        };
        CardDisplay { card, text }
    }
}

impl Display for CardDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArtDisplay {
    card: CardRef,
    idx: usize,
    text: String,
}

impl ArtDisplay {
    pub fn new(card: CardRef, idx: usize, game: &Game) -> ArtDisplay {
        let text = if let Some(m) = game.lookup_holo_member(card) {
            let art = &m.arts[idx];
            format!(
                "{} ({:?}) ({}) ({}) {}",
                art.name,
                art.damage,
                art.cost
                    .iter()
                    .map(|c| format!("{c:?}"))
                    .collect_vec()
                    .join(", "),
                m.card_number,
                card,
            )
        } else {
            unreachable!("only members can have arts")
        };
        ArtDisplay { card, idx, text }
    }
}

impl Display for ArtDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}
