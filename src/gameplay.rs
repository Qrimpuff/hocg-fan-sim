use std::fmt::Display;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

use super::cards::*;
use super::modifiers::*;
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

const STARTING_HAND_SIZE: usize = 7;
const MAX_MEMBERS_ON_STAGE: usize = 6;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardUuid(usize);

impl Debug for CardUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "c_{:016x}", self.0)
    }
}
impl Display for CardUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

fn register_card(
    player: Player,
    card_ref: &CardRef,
    card_map: &mut HashMap<CardUuid, (Player, CardRef)>,
) -> CardUuid {
    let card = CardUuid((thread_rng().gen::<usize>() << 4) + (player as usize));
    card_map.insert(card, (player, card_ref.clone()));
    card
}

fn shuffle_deck(deck: &mut Vec<CardUuid>) {
    deck.shuffle(&mut thread_rng());
}

#[derive(Debug)]
pub struct Game<P: Prompter> {
    pub(crate) library: Arc<GlobalLibrary>,
    pub(crate) player_1: GameBoard,
    pub(crate) player_2: GameBoard,
    pub(crate) card_map: HashMap<CardUuid, (Player, CardRef)>,
    pub(crate) active_player: Player,
    pub(crate) active_phase: Phase,
    pub(crate) zone_modifiers: Vec<(Player, Zone, Modifier)>,
    pub(crate) card_modifiers: Vec<(CardUuid, Modifier)>,
    pub(crate) prompter: P,
}

impl<P: Prompter> Game<P> {
    pub fn setup(
        library: Arc<GlobalLibrary>,
        player_1: &Loadout,
        player_2: &Loadout,
        prompter: P,
    ) -> Game<P> {
        let mut card_map = HashMap::new();
        Game {
            library,
            player_1: GameBoard::setup(Player::One, player_1, &mut card_map),
            player_2: GameBoard::setup(Player::Two, player_2, &mut card_map),
            card_map,
            active_player: Player::One,
            active_phase: Phase::Setup,
            zone_modifiers: Vec::new(),
            card_modifiers: Vec::new(),
            prompter,
        }
    }

    fn active_board(&self) -> &GameBoard {
        self.board(self.active_player)
    }
    fn active_board_mut(&mut self) -> &mut GameBoard {
        self.board_mut(self.active_player)
    }
    fn board(&self, player: Player) -> &GameBoard {
        match player {
            Player::One => &self.player_1,
            Player::Two => &self.player_2,
            _ => unreachable!("both players cannot be active at the same time"),
        }
    }
    fn board_mut(&mut self, player: Player) -> &mut GameBoard {
        match player {
            Player::One => &mut self.player_1,
            Player::Two => &mut self.player_2,
            _ => unreachable!("both players cannot be active at the same time"),
        }
    }

    pub(crate) fn player_for_card(&self, card: CardUuid) -> Player {
        self.card_map
            .get(&card)
            .expect("the card should be registered")
            .0
    }
    fn board_for_card(&self, card: CardUuid) -> &GameBoard {
        let player = self.player_for_card(card);
        self.board(player)
    }
    fn board_for_card_mut(&mut self, card: CardUuid) -> &mut GameBoard {
        let player = self.player_for_card(card);
        self.board_mut(player)
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

    fn handle_mulligan(&mut self, player: Player) {
        // - draw 7 cards from main deck
        let mut player_draw = STARTING_HAND_SIZE;
        self.board_mut(player).draw(player_draw);

        //   - can mulligan for -1 card. at 0 lose the game
        loop {
            let force_mulligan = self.need_mulligan(self.board(player));
            println!("prompt mulligan: {player:?}");
            let mulligan = force_mulligan || self.prompt_for_mulligan();
            if !mulligan {
                break;
            }

            println!(
                "player {player:?} mulligan{}",
                if force_mulligan { " by force" } else { "" }
            );
            self.board_mut(player)
                .send_all_from_zone(Zone::Hand, Zone::MainDeck);
            shuffle_deck(&mut self.board_mut(player).main_deck);
            player_draw -= 1;
            if player_draw == 0 {
                break;
            }

            println!("player {player:?} draws {player_draw}");
            self.board_mut(player).draw(player_draw);
        }

        if player_draw == 0 {
            self.active_player = player;
            self.lose_game();
        }
    }

    pub fn start_game(&mut self) {
        self.active_phase = Phase::Setup;

        // - shuffle main deck
        shuffle_deck(&mut self.player_1.main_deck);
        shuffle_deck(&mut self.player_2.main_deck);

        // - shuffle cheer deck
        shuffle_deck(&mut self.player_1.cheer_deck);
        shuffle_deck(&mut self.player_2.cheer_deck);

        // - oshi face down
        // TODO oshi hide

        // - rock/paper/scissor to choose first (coin flip?)
        loop {
            println!("prompt rps");
            let rps_1 = self.prompt_for_rps();
            let rps_2 = self.prompt_for_rps();
            match rps_1.vs(rps_2) {
                RpsOutcome::Win => {
                    println!("player 1 win rps");
                    // TODO choose first or second
                    self.active_player = Player::One;
                    break;
                }
                RpsOutcome::Lose => {
                    println!("player 2 win rps");
                    // TODO choose first or second
                    self.active_player = Player::Two;
                    break;
                }
                RpsOutcome::Draw => {
                    println!("draw rps");
                    continue;
                }
            }
        }

        // - draw 7 cards from main deck
        //   - can mulligan for -1 card. at 0 lose the game
        self.handle_mulligan(Player::One);
        if self.active_phase == Phase::GameOver {
            return;
        }

        self.handle_mulligan(Player::Two);
        if self.active_phase == Phase::GameOver {
            return;
        }

        // - place debut member center face down
        // TODO member hide
        println!("prompt debut 1");
        let debut_1 = self.prompt_for_first_debut(Player::One);
        self.player_1.send_to_zone(debut_1, Zone::MainStageCenter);

        println!("prompt debut 2");
        let debut_2 = self.prompt_for_first_debut(Player::Two);
        self.player_2.send_to_zone(debut_2, Zone::MainStageCenter);

        // - place other debut members back stage
        println!("prompt other debut 1");
        let other_debut_1: Vec<_> = self.prompt_for_first_back_stage(Player::One);
        self.player_1
            .send_many_to_zone(other_debut_1, Zone::BackStage);
        println!("prompt other debut 2");
        let other_debut_2: Vec<_> = self.prompt_for_first_back_stage(Player::Two);
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
    }

    pub fn next_phase(&mut self) -> bool {
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
                    _ => unreachable!("both players cannot be active at the same time"),
                };
                Phase::Refresh
            }
            Phase::GameOver => {
                println!("the game is over");
                return false;
            }
        };

        // start turn
        if self.active_phase == Phase::Refresh {
            self.start_turn();
        }

        println!("- active phase: {:?}", self.active_phase);

        // after phase change
        self.check_loss_conditions();

        match self.active_phase {
            Phase::Setup => panic!("should not setup more than once"),
            Phase::Refresh => self.refresh_phase(),
            Phase::Draw => self.draw_phase(),
            Phase::Cheer => self.cheer_phase(),
            Phase::Main => self.main_phase(),
            Phase::Art => self.art_phase(),
            Phase::End => self.end_phase(),
            Phase::GameOver => return false,
        }

        // end turn
        if self.active_phase == Phase::End {
            self.end_turn();
        }

        true
    }

    fn start_turn(&mut self) {
        println!("active player: {:?}", self.active_player);

        // TODO trigger more turn change effects
        self.start_turn_modifiers(self.active_player);
    }

    fn end_turn(&mut self) {
        // TODO trigger more turn change effects
        self.end_turn_modifiers();
    }

    fn refresh_phase(&mut self) {
        // TODO trigger phase change effects

        // - all members from rest to active
        for mem in self.active_board().stages().collect::<Vec<_>>() {
            self.awake_card(mem);
        }

        // - collab to back stage in rest
        if let Some(mem) = self.active_board().main_collab {
            self.rest_card(mem);
            self.active_board_mut().send_to_zone(mem, Zone::BackStage);
        }
        // - if no center, back stage to center
        if self.active_board().main_center.is_none() && !self.active_board().back_stage.is_empty() {
            println!("prompt new center member");
            let back = self.prompt_for_back_stage_to_center(self.active_player);
            self.active_board_mut()
                .send_to_zone(back, Zone::MainStageCenter);
        }

        //   - no back stage lose game
        if self.active_board().main_center.is_none() {
            self.lose_game();
            // return;
        }
    }

    fn draw_phase(&mut self) {
        // TODO trigger phase change effects

        // - draw 1 card from main deck
        self.active_board_mut().draw(1);
    }

    fn cheer_phase(&mut self) {
        // TODO trigger phase change effects

        // - draw 1 card from cheer deck, attach it
        if let Some(cheer) = self
            .active_board_mut()
            .get_zone(Zone::CheerDeck)
            .peek_top_card()
        {
            // TODO show card
            println!("prompt member for cheer");
            //   - main stage or back stage
            let mem = self.prompt_for_cheer(self.active_player);
            self.active_board_mut().attach_to_card(cheer, mem);
        }
    }

    fn main_phase(&mut self) {
        // TODO trigger phase change effects

        loop {
            println!("prompt main phase action");
            match self.prompt_for_main_action(self.active_player) {
                MainPhaseAction::BackStageMember(card) => {
                    println!("- action: Back stage member");
                    // TODO verify back stage member action

                    // - place debut member on back stage
                    self.active_board_mut().send_to_zone(card, Zone::BackStage);
                }
                MainPhaseAction::BloomMember(bloom) => {
                    println!("- action: Bloom member");
                    // TODO verify bloom member action

                    // - bloom member (evolve e.g. debut -> 1st )
                    //   - bloom effect
                    //   - can't bloom on same turn as placed
                    let card = self.prompt_for_bloom(self.active_player, bloom);
                    self.bloom_member(bloom, card);
                }
                MainPhaseAction::UseSupportCard(card) => {
                    println!("- action: Use support card");
                    // TODO verify use support card action

                    // - use support card
                    //   - only one limited per turn
                    //   - otherwise unlimited

                    // TODO implement effect
                    self.send_to_archive(card);
                }
                MainPhaseAction::CollabMember(card) => {
                    println!("- action: Collab member");
                    // TODO verify collab member action
                    if self.is_rested(card) {
                        panic!("cannot collab a rested member");
                    }

                    // - put back stage member in collab
                    //   - can be done on first turn?
                    //   - draw down card from deck into power zone
                    self.active_board_mut()
                        .send_to_zone(card, Zone::MainStageCollab);
                    self.active_board_mut()
                        .send_from_zone(Zone::MainDeck, Zone::HoloPower, 1);
                }
                MainPhaseAction::BatonPass(card) => {
                    println!("- action: Baton pass");
                    // TODO verify baton pass action

                    // - retreat switch (baton pass)
                    //   - switch center with back stage
                    //   - remove attached cheer for cost
                    let center = self
                        .active_board()
                        .main_center
                        .expect("there should always be a card in the center");
                    // only center stage can baton pass
                    assert_eq!(center, card);
                    self.pay_baton_pass_cost(center);
                    let card = self.prompt_for_back_stage_to_center(self.active_player);
                    self.active_board_mut()
                        .send_to_zone(center, Zone::BackStage);
                    self.active_board_mut()
                        .send_to_zone(card, Zone::MainStageCenter);
                }
                MainPhaseAction::UseAbilities(_card, _) => {
                    println!("- action: Use abilities");
                    // TODO verify use abilities action

                    // - use abilities (including oshi)
                    //   - oshi power uses card in power zone
                    //   - once per turn / once per game?
                    // TODO prevent duplicate ability use with (buff)
                    // TODO ability cost
                    // TODO implement effect
                }
                MainPhaseAction::Done => {
                    println!("- action: Done");
                    break;
                }
            }

            self.check_loss_conditions();
            if self.active_phase == Phase::GameOver {
                break;
            }
        }

        // attack can be preloaded at this point
    }

    fn art_phase(&mut self) {
        // TODO trigger phase change effects

        // - can use 2 attacks (center, collab)
        // - can choose target (center, collab)
        // - need required attached cheers to attack
        // - apply damage and effects
        // - remove member if defeated
        //   - lose 1 life
        //   - attach lost life (cheer)
        // TODO implement effect

        // TODO only for testing
        let op = self.active_player.opponent();
        let target = self
            .board(op)
            .main_center
            .expect("there should always be a member in the center");
        self.add_damage(target, DamageCounters(1));

        println!("deals damage");

        if self.remaining_hp(target) == 0 {
            self.send_to_archive(target);
        }
    }

    fn end_phase(&mut self) {
        // TODO trigger phase change effects

        // - any end phase effect
    }

    fn win_game(&mut self) {
        match self.active_player {
            Player::One => println!("player 1 wins"),
            Player::Two => println!("player 2 wins"),
            _ => unreachable!("both players cannot be active at the same time"),
        };
        // stop the game
        self.active_phase = Phase::GameOver;
    }
    fn lose_game(&mut self) {
        self.active_player = match self.active_player {
            Player::One => Player::Two,
            Player::Two => Player::One,
            _ => unreachable!("both players cannot be active at the same time"),
        };
        self.win_game();
    }

    fn check_loss_conditions(&mut self) {
        let mut lose_player = None;
        for (player, board) in [(Player::One, &self.player_1), (Player::Two, &self.player_2)] {
            // - life is 0
            if board.life.count() == 0 {
                lose_player = Some(player);
            }

            // - deck is 0 on draw phase
            if board.main_deck.count() == 0
                && self.active_phase == Phase::Draw
                && self.active_player == player
            {
                lose_player = Some(player);
            }

            // - 0 member in play
            if board.back_stage.count() + board.main_center.count() + board.main_collab.count() == 0
            {
                lose_player = Some(player);
            }

            if lose_player.is_some() {
                break;
            }
        }

        if let Some(lose_player) = lose_player {
            self.active_player = lose_player;
            self.lose_game();
        }
    }

    pub fn lookup_card(&self, card: CardUuid) -> &Card {
        let (_, card_ref) = self.card_map.get(&card).expect("should be in the map");
        self.library
            .lookup_card(card_ref)
            .expect("should be in the library")
    }
    pub fn lookup_holo_member(&self, card: CardUuid) -> Option<&HoloMemberCard> {
        if let Card::HoloMember(m) = self.lookup_card(card) {
            Some(m)
        } else {
            None
        }
    }

    pub fn bloom_member(&mut self, bloom: CardUuid, card: CardUuid) {
        self.active_board_mut().attach_to_card(bloom, card);
        self.active_board_mut().promote_attachment(bloom, card);
        self.promote_modifiers(bloom, card);
    }

    pub fn pay_baton_pass_cost(&mut self, card: CardUuid) {
        if let Card::HoloMember(mem) = self.lookup_card(card) {
            // TODO cost should automatic when there is a single cheers color
            let cheers = self.prompt_for_baton_pass(card, mem.baton_pass_cost);
            self.active_board_mut().remove_many_attachments(cheers);
        } else {
            panic!("cannot pay baton pass cost for non member");
        }
    }

    pub fn attached_cheers(&self, card: CardUuid) -> Vec<CardUuid> {
        self.board_for_card(card)
            .attachments(card)
            .into_iter()
            .filter(|a| matches!(self.lookup_card(*a), Card::Cheer(_)))
            .collect()
    }

    pub fn send_to_archive(&mut self, card: CardUuid) {
        self.board_for_card_mut(card)
            .send_to_zone(card, Zone::Archive);
        self.board_for_card_mut(card)
            .send_all_attachments_to_zone(card, Zone::Archive);
        self.clear_all_modifiers(card);
    }

    fn prompt_for_rps(&mut self) -> Rps {
        self.prompter.prompt_choice(
            "choose rock, paper or scissor:",
            vec![Rps::Rock, Rps::Paper, Rps::Scissor],
        )
    }

    fn prompt_for_mulligan(&mut self) -> bool {
        self.prompter
            .prompt_choice("do you want to mulligan?", vec!["Yes", "No"])
            == "Yes"
    }

    fn prompt_for_first_debut(&mut self, player: Player) -> CardUuid {
        // TODO extract that filtering to a reusable function
        let debuts: Vec<_> = self
            .board(player)
            .hand()
            .filter_map(|c| self.lookup_holo_member(c).map(|m| (c, m)))
            .filter(|(_, m)| m.rank == HoloMemberRank::Debut)
            .map(|(c, _)| CardDisplay::new(c, self))
            .collect();

        assert!(!debuts.is_empty());
        self.prompter
            .prompt_choice("choose first debut:", debuts)
            .card
    }

    fn prompt_for_first_back_stage(&mut self, player: Player) -> Vec<CardUuid> {
        // TODO extract that filtering to a reusable function
        let debuts: Vec<_> = self
            .board(player)
            .hand()
            .filter_map(|c| self.lookup_holo_member(c).map(|m| (c, m)))
            .filter(|(_, m)| m.rank == HoloMemberRank::Debut)
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

    fn prompt_for_back_stage_to_center(&mut self, player: Player) -> CardUuid {
        // TODO extract that filtering to a reusable function
        let back: Vec<_> = self
            .board(player)
            .back_stage()
            .filter_map(|c| self.lookup_holo_member(c).map(|m| (c, m)))
            // TODO check for rest?
            .map(|(c, _)| CardDisplay::new(c, self))
            .collect();

        assert!(!back.is_empty());
        self.prompter
            .prompt_choice("choose send to center stage:", back)
            .card
    }

    fn prompt_for_cheer(&mut self, player: Player) -> CardUuid {
        // TODO extract that filtering to a reusable function
        let cheer: Vec<_> = self
            .board(player)
            .stages()
            .filter_map(|c| self.lookup_holo_member(c).map(|m| (c, m)))
            // TODO check for rest?
            .map(|(c, _)| CardDisplay::new(c, self))
            .collect();

        assert!(!cheer.is_empty());
        self.prompter
            .prompt_choice("choose receive cheer:", cheer)
            .card
    }

    #[allow(clippy::unnecessary_filter_map)]
    fn prompt_for_main_action(&mut self, player: Player) -> MainPhaseAction {
        // actions from hand
        let mut actions: Vec<_> = self
            .board(player)
            .hand()
            .filter_map(|c| match self.lookup_card(c) {
                Card::OshiHoloMember(_) => unreachable!("oshi cannot be in hand"),
                Card::HoloMember(m) => match m.rank {
                    HoloMemberRank::Debut => {
                        // check condition for back stage
                        let count = self
                            .board(player)
                            .stages()
                            .filter_map(|c| self.lookup_holo_member(c))
                            .count();
                        if count < MAX_MEMBERS_ON_STAGE {
                            Some(MainPhaseAction::BackStageMember(c))
                        } else {
                            None
                        }
                    }
                    HoloMemberRank::First | HoloMemberRank::Second => {
                        // TODO only once per turn (buff)
                        // check condition for bloom
                        let bloom_lookup = match m.rank {
                            HoloMemberRank::Debut => panic!("can only bloom from first or second"),
                            // TODO better match, maybe with (tags)
                            HoloMemberRank::First => (HoloMemberRank::Debut, &m.name),
                            HoloMemberRank::Second => (HoloMemberRank::First, &m.name),
                        };
                        let can_bloom = self
                            .board(player)
                            .stages()
                            .filter_map(|c| self.lookup_holo_member(c))
                            .any(|m| bloom_lookup == (m.rank, &m.name));
                        if can_bloom {
                            Some(MainPhaseAction::BloomMember(c))
                        } else {
                            None
                        }
                    }
                },
                // TODO check condition to play support
                Card::Support(_) => Some(MainPhaseAction::UseSupportCard(c)),
                Card::Cheer(_) => unreachable!("cheer cannot be in hand"),
            })
            .map(|a| MainPhaseActionDisplay::new(a, self))
            .collect();

        // actions from board
        // collab
        actions.extend(
            self.board(player)
                .back_stage()
                .filter_map(|c| {
                    match self.lookup_card(c) {
                        Card::OshiHoloMember(_) => unreachable!("oshi cannot be in the back stage"),
                        Card::HoloMember(_) => {
                            // check condition for collab
                            if self.is_rested(c) {
                                return None;
                            }

                            if self.board(player).main_collab.is_none() {
                                Some(MainPhaseAction::CollabMember(c))
                            } else {
                                None
                            }
                        }
                        Card::Support(_) => unreachable!("support cannot be in the back stage"),
                        Card::Cheer(_) => unreachable!("cheer cannot be in the back stage"),
                    }
                })
                .map(|a| MainPhaseActionDisplay::new(a, self)),
        );
        // baton pass
        actions.extend(
            self.board(player)
                .main_center
                .iter()
                .copied()
                .filter_map(|c| {
                    match self.lookup_card(c) {
                        Card::OshiHoloMember(_) => {
                            unreachable!("oshi cannot be in the center stage")
                        }
                        Card::HoloMember(m) => {
                            // check condition for baton pass
                            let cheer_count = self.attached_cheers(c).len();
                            let back_stage_count = self
                                .board(player)
                                .back_stage()
                                .filter_map(|c| self.lookup_holo_member(c))
                                .count();

                            if cheer_count >= m.baton_pass_cost as usize && back_stage_count > 0 {
                                Some(MainPhaseAction::BatonPass(c))
                            } else {
                                None
                            }
                        }
                        Card::Support(_) => unreachable!("support cannot be in the center stage"),
                        Card::Cheer(_) => unreachable!("cheer cannot be in the center stage"),
                    }
                })
                .map(|a| MainPhaseActionDisplay::new(a, self)),
        );
        // abilities
        actions.extend(
            self.board(player)
                .stages()
                .flat_map(|c| {
                    match self.lookup_card(c) {
                        Card::OshiHoloMember(o) => o
                            .abilities
                            .iter()
                            .enumerate()
                            // TODO check condition for ability
                            // TODO prevent duplicate ability use with (buff)
                            .map(|(i, _a)| MainPhaseAction::UseAbilities(c, i))
                            .collect::<Vec<_>>(),
                        Card::HoloMember(m) => m
                            .abilities
                            .iter()
                            .enumerate()
                            // TODO check condition for ability
                            // TODO prevent duplicate ability use with (buff)
                            .map(|(i, _a)| MainPhaseAction::UseAbilities(c, i))
                            .collect::<Vec<_>>(),
                        Card::Support(_) => todo!("support could maybe have ability once attached"),
                        Card::Cheer(_) => todo!("cheer could maybe have ability once attached"),
                    }
                })
                .map(|a| MainPhaseActionDisplay::new(a, self)),
        );

        actions.push(MainPhaseActionDisplay::new(MainPhaseAction::Done, self));
        actions.sort();

        assert!(!actions.is_empty());
        self.prompter
            .prompt_choice("main phase action:", actions)
            .action
    }

    fn prompt_for_bloom(&mut self, player: Player, card: CardUuid) -> CardUuid {
        // TODO extract that filtering to a reusable function
        let Card::HoloMember(bloom) = self.lookup_card(card) else {
            panic!("can only bloom from member")
        };

        let bloom_lookup = match bloom.rank {
            HoloMemberRank::Debut => panic!("can only bloom from first or second"),
            // TODO better match, maybe with (tags)
            HoloMemberRank::First => (HoloMemberRank::Debut, &bloom.name),
            HoloMemberRank::Second => (HoloMemberRank::First, &bloom.name),
        };

        let stage: Vec<_> = self
            .board(player)
            .stages()
            .filter_map(|c| self.lookup_holo_member(c).map(|m| (c, m)))
            .filter(|(_, m)| bloom_lookup == (m.rank, &m.name))
            // TODO check for rest?
            .map(|(c, _)| CardDisplay::new(c, self))
            .collect();

        assert!(!stage.is_empty());
        self.prompter.prompt_choice("choose for bloom:", stage).card
    }

    fn prompt_for_baton_pass(
        &mut self,
        card: CardUuid,
        cost: HoloMemberBatonPassCost,
    ) -> Vec<CardUuid> {
        // TODO extract that filtering to a reusable function
        let cheers: Vec<_> = self
            .attached_cheers(card)
            .into_iter()
            .map(|c| CardDisplay::new(c, self))
            .collect();

        if !cheers.is_empty() {
            self.prompter
                .prompt_multi_choices("choose cheers to remove:", cheers, cost.into(), cost.into())
                .into_iter()
                .map(|c| c.card)
                .collect()
        } else {
            vec![]
        }
    }

    fn prompt_for_art(&mut self, card: CardUuid) -> CardUuid {
        // TODO extract that filtering to a reusable function
        todo!()
    }

    fn prompt_for_art_target(&mut self, card: CardUuid) -> CardUuid {
        // TODO extract that filtering to a reusable function
        todo!()
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
    attachments: HashMap<CardUuid, CardUuid>,
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
                .map(|c| register_card(player, c, card_map))
                .collect(),
            oshi: Some(register_card(player, &loadout.oshi, card_map)),
            main_center: None,
            main_collab: None,
            back_stage: Vec::new(),
            life: Vec::new(),
            cheer_deck: loadout
                .cheer_deck
                .iter()
                .map(|c| register_card(player, c, card_map))
                .collect(),
            holo_power: Vec::new(),
            archive: Vec::new(),
            hand: Vec::new(),
            attachments: HashMap::new(),
        }
    }

    pub fn hand(&self) -> impl Iterator<Item = CardUuid> + '_ {
        self.hand.iter().copied()
    }
    pub fn stages(&self) -> impl Iterator<Item = CardUuid> + '_ {
        self.main_stage().chain(self.back_stage())
    }
    pub fn main_stage(&self) -> impl Iterator<Item = CardUuid> + '_ {
        self.main_center
            .iter()
            .chain(self.main_collab.iter())
            .chain(self.oshi.iter())
            .copied()
    }
    pub fn back_stage(&self) -> impl Iterator<Item = CardUuid> + '_ {
        self.back_stage.iter().copied()
    }

    pub fn draw(&mut self, amount: usize) {
        self.send_from_zone(Zone::MainDeck, Zone::Hand, amount);
    }

    pub fn add_life(&mut self, amount: u8) {
        self.send_from_zone(Zone::CheerDeck, Zone::Life, amount.into());
    }

    pub fn is_attached(&self, attachment: CardUuid) -> bool {
        self.attachments.contains_key(&attachment)
    }

    pub fn attach_to_card(&mut self, attachment: CardUuid, card: CardUuid) {
        let current_zone = self.find_card_zone(attachment);
        if let Some(zone) = current_zone {
            self.get_zone(zone).remove_card(attachment);
        } else if self.attachments.contains_key(&card) {
            panic!("cannot attach to attachment");
        }

        self.attachments.insert(attachment, card);
    }

    pub fn remove_attachment(&mut self, attachment: CardUuid) {
        self.send_to_zone(attachment, Zone::Archive);
    }
    pub fn remove_many_attachments(&mut self, attachments: impl IntoIterator<Item = CardUuid>) {
        self.send_many_to_zone(attachments, Zone::Archive);
    }

    pub fn send_all_attachments_to_zone(&mut self, card: CardUuid, target_zone: Zone) {
        let attached = self.attachments(card);
        self.send_many_to_zone(attached, target_zone)
    }

    pub fn promote_attachment(&mut self, attachment: CardUuid, parent: CardUuid) {
        if let Some(current_zone) = self.find_card_zone(parent) {
            // replace with attachment
            self.get_zone(current_zone).replace_card(parent, attachment);

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

    pub fn attachments(&self, card: CardUuid) -> Vec<CardUuid> {
        self.attachments
            .iter()
            .filter_map(|(k, v)| if *v == card { Some(k) } else { None })
            .copied()
            .collect()
    }

    pub fn send_to_zone(&mut self, card: CardUuid, target_zone: Zone) {
        let current_zone = self.find_card_zone(card);
        if let Some(zone) = current_zone {
            self.get_zone(zone).remove_card(card);
            self.get_zone(target_zone).add_top_card(card);
        } else if self.is_attached(card) {
            self.attachments.remove(&card);
            self.get_zone(target_zone).add_top_card(card);
        }
    }

    pub fn send_many_to_zone(
        &mut self,
        cards: impl IntoIterator<Item = CardUuid>,
        target_zone: Zone,
    ) {
        cards
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
            Zone::All => unreachable!("a card cannot be in all zones"),
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

    pub fn find_card_zone(&self, card: CardUuid) -> Option<Zone> {
        if self.main_deck.is_in_zone(card) {
            Some(Zone::MainDeck)
        } else if self.oshi.is_in_zone(card) {
            Some(Zone::MainStageOshi)
        } else if self.main_center.is_in_zone(card) {
            Some(Zone::MainStageCenter)
        } else if self.main_collab.is_in_zone(card) {
            Some(Zone::MainStageCollab)
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
        } else {
            None
        }
    }
}

trait ZoneControl {
    fn count(&self) -> usize;
    fn peek_top_card(&self) -> Option<CardUuid>;
    fn remove_card(&mut self, card: CardUuid);
    fn add_top_card(&mut self, card: CardUuid);
    fn add_bottom_card(&mut self, card: CardUuid);
    fn replace_card(&mut self, from_card: CardUuid, to_card: CardUuid);
    fn is_in_zone(&self, card: CardUuid) -> bool;
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

    fn remove_card(&mut self, card: CardUuid) {
        if self.is_in_zone(card) {
            self.take();
        }
    }

    fn add_top_card(&mut self, card: CardUuid) {
        if self.is_none() {
            self.replace(card);
        } else {
            panic!("there is already a card in this zone");
        }
    }

    fn add_bottom_card(&mut self, card: CardUuid) {
        self.add_top_card(card)
    }

    fn replace_card(&mut self, from_card: CardUuid, to_card: CardUuid) {
        if self.is_in_zone(from_card) {
            self.replace(to_card);
        } else {
            panic!("card is not already in this zone");
        }
    }

    fn is_in_zone(&self, card: CardUuid) -> bool {
        *self == Some(card)
    }
}

impl ZoneControl for Vec<CardUuid> {
    fn count(&self) -> usize {
        self.len()
    }

    fn peek_top_card(&self) -> Option<CardUuid> {
        self.first().copied()
    }

    fn remove_card(&mut self, card: CardUuid) {
        if let Some(index) = self.iter().position(|c| *c == card) {
            self.remove(index);
        }
    }

    fn add_top_card(&mut self, card: CardUuid) {
        if !self.is_in_zone(card) {
            self.insert(0, card);
        } else {
            panic!("there is already a card in this zone");
        }
    }

    fn add_bottom_card(&mut self, card: CardUuid) {
        if !self.is_in_zone(card) {
            self.push(card);
        } else {
            panic!("there is already a card in this zone");
        }
    }

    fn replace_card(&mut self, from_card: CardUuid, to_card: CardUuid) {
        if let Some(index) = self.iter().position(|c| *c == from_card) {
            *self.get_mut(index).expect("card is already in the zone") = to_card;
        } else {
            panic!("card is not already in this zone");
        }
    }

    fn is_in_zone(&self, card: CardUuid) -> bool {
        self.iter().any(|c| *c == card)
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
    GameOver,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Rps {
    Rock,
    Paper,
    Scissor,
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

pub enum MainPhaseAction {
    BackStageMember(CardUuid),
    BloomMember(CardUuid),
    UseSupportCard(CardUuid),
    CollabMember(CardUuid),
    BatonPass(CardUuid),
    UseAbilities(CardUuid, usize),
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct MainPhaseActionDisplay {
    action: MainPhaseAction,
    text: String,
}

impl MainPhaseActionDisplay {
    pub fn new<P: Prompter>(action: MainPhaseAction, game: &Game<P>) -> MainPhaseActionDisplay {
        let text = match action {
            MainPhaseAction::BackStageMember(card) => {
                let display = CardDisplay::new(card, game);
                format!("put on back stage: {display}")
            }
            MainPhaseAction::BloomMember(card) => {
                let display = CardDisplay::new(card, game);
                format!("bloom member with: {display}")
            }
            MainPhaseAction::UseSupportCard(card) => {
                let display = CardDisplay::new(card, game);
                format!("use support: {display}")
            }
            MainPhaseAction::CollabMember(card) => {
                let display = CardDisplay::new(card, game);
                format!("move to collab: {display}")
            }
            MainPhaseAction::BatonPass(card) => {
                let display = CardDisplay::new(card, game);
                format!("baton pass (retreat): {display}")
            }
            MainPhaseAction::UseAbilities(card, idx) => {
                let display = CardDisplay::new(card, game);
                let ability = match game.lookup_card(card) {
                    Card::OshiHoloMember(o) => o
                        .abilities
                        .get(idx)
                        .expect("the index should be in range")
                        .name
                        .clone(),
                    Card::HoloMember(m) => m
                        .abilities
                        .get(idx)
                        .expect("the index should be in range")
                        .name
                        .clone(),
                    Card::Support(_) => todo!("support could maybe have ability once attached"),
                    Card::Cheer(_) => todo!("cheer could maybe have ability once attached"),
                };
                format!("use ability: {ability} from {display}")
            }
            MainPhaseAction::Done => "done".into(),
        };

        MainPhaseActionDisplay { action, text }
    }
}

impl Display for MainPhaseActionDisplay {
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
        println!(
            "{}",
            c.iter().map(T::to_string).collect::<Vec<_>>().join(", ")
        );
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

        let c = choices
            .into_iter()
            .choose_multiple(&mut thread_rng(), thread_rng().gen_range(min..=max));
        println!(
            "{}",
            c.iter().map(T::to_string).collect::<Vec<_>>().join(", ")
        );
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
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
    // fn prompt_rps_choice();
    // fn prompt_mulligan_choice();
    // fn prompt_card_in_hand_choice();
    // fn prompt_card_on_stage_choice();
    // fn prompt_zone_choice();
    // fn prompt_main_phase_action_choice();
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
    card: CardUuid,
    text: String,
}

impl CardDisplay {
    pub fn new<P: Prompter>(card: CardUuid, game: &Game<P>) -> CardDisplay {
        let text = match game.lookup_card(card) {
            Card::OshiHoloMember(o) => format!("{} ({} life)", o.name, o.life),
            Card::HoloMember(m) => {
                format!(
                    "{} ({:?}) ({}/{}) ({} cheers) (?) ({})",
                    m.name,
                    m.rank,
                    game.remaining_hp(card),
                    m.hp,
                    game.attached_cheers(card).len(),
                    m.id
                )
            }
            Card::Support(s) => format!("{} ({:?})", s.name, s.kind),
            Card::Cheer(c) => format!("{} ({:?})", c.name, c.color),
        };
        CardDisplay { card, text }
    }
}

impl Display for CardDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}
