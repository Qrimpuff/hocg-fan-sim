use std::fmt::Display;
use std::num::NonZeroUsize;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

use crate::card_effects::evaluate::EvaluateEffect;

use super::cards::*;
use super::modifiers::*;
use iter_tools::Itertools;
use rand::seq::IteratorRandom;
use rand::{thread_rng, Rng};
use ModifierKind::*;

const STARTING_HAND_SIZE: usize = 7;
const MAX_MEMBERS_ON_STAGE: usize = 6;

static PRIVATE_CARD: CardRef = CardRef(NonZeroUsize::MAX);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardRef(pub(crate) NonZeroUsize);

impl Debug for CardRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "c_{:016x}", self.0)
    }
}
impl Display for CardRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
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
    winning_player: Option<Player>,
}

pub fn register_card(
    player: Player,
    card_number: &CardNumber,
    card_map: &mut HashMap<CardRef, (Player, CardNumber)>,
) -> CardRef {
    let card = CardRef(
        NonZeroUsize::new((thread_rng().gen::<usize>() << 4) + (player as usize) + 1)
            .expect("that plus one makes it non zero"),
    );
    card_map.insert(card, (player, card_number.clone()));
    card
}

#[derive(Debug)]
pub struct Game {
    pub library: Arc<GlobalLibrary>,
    pub card_map: HashMap<CardRef, (Player, CardNumber)>, // TODO use a different pair because rarity is not include in card number
    pub player_1: GameBoard,
    pub player_2: GameBoard,
    pub active_player: Player,
    pub active_step: Step,
    pub turn_number: usize,
    pub zone_modifiers: HashMap<Player, Vec<(Zone, Modifier)>>,
    pub card_modifiers: HashMap<CardRef, Vec<Modifier>>,
    pub card_damage_markers: HashMap<CardRef, DamageMarkers>,
    pub prompter: RandomPrompter, // will probably be replace by 2 player send/receive channels
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
        self.draw_from_main_deck(player, player_draw)?;
        // self.board_mut(player).draw(player_draw);

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
                self.reveal_all_cards_in_zone(player, Zone::Hand)?;
            }

            // self.board_mut(player)
            //     .send_all_from_zone(Zone::Hand, Zone::MainDeck);
            self.send_full_hand_to_main_deck(player)?;
            // shuffle_deck(&mut self.board_mut(player).main_deck);
            self.shuffle_main_deck(player)?;

            println!("player {player:?} draws {player_draw}");
            // self.board_mut(player).draw(player_draw);
            self.draw_from_main_deck(player, player_draw)?;

            player_draw -= 1;
            if player_draw == 0 {
                break;
            }
        }

        if player_draw == 0 {
            self.active_player = player;
            println!("player {player:?} cannot draw anymore card");
            self.lose_game()?;
        }

        Ok(GameContinue)
    }

    pub fn start_game(&mut self) -> GameResult {
        self.active_step = Step::Setup;

        // - shuffle main deck
        // TODO send (event) shuffle
        // shuffle_deck(&mut self.player_1.main_deck);
        // shuffle_deck(&mut self.player_2.main_deck);
        self.shuffle_main_deck(Player::One)?;
        self.shuffle_main_deck(Player::Two)?;

        // - shuffle cheer deck
        // TODO send (event) shuffle
        // shuffle_deck(&mut self.player_1.cheer_deck);
        // shuffle_deck(&mut self.player_2.cheer_deck);
        self.shuffle_cheer_deck(Player::One)?;
        self.shuffle_cheer_deck(Player::Two)?;

        // - oshi face down
        // TODO oshi hide
        // TODO send (event) put oshi, not needed ? starts hidden

        // - rock/paper/scissor to choose first
        // TODO send (event) win / loss
        // TODO request (intent)
        loop {
            println!("prompt rps");
            let rps_1 = self.prompt_for_rps();
            let rps_2 = self.prompt_for_rps();
            match rps_1.vs(rps_2) {
                RpsOutcome::Win => {
                    println!("player 1 win rps");
                    self.report_rps_win(Player::One)?;
                    // TODO choose first or second
                    self.active_player = Player::One;
                    break;
                }
                RpsOutcome::Lose => {
                    println!("player 2 win rps");
                    self.report_rps_win(Player::Two)?;
                    // TODO choose first or second
                    self.active_player = Player::Two;
                    break;
                }
                RpsOutcome::Draw => {
                    println!("draw rps");
                    self.report_rps_draw()?;
                    continue;
                }
            }
        }
        // TODO choose first or second
        self.report_player_going_first(self.active_player)?;

        // - draw 7 cards from main deck
        //   - can mulligan once, forced for -1 card. at 0 lose the game
        // TODO send (event) draw
        // TODO request (intent)
        self.handle_mulligan(Player::One)?;
        // if self.active_step == Step::GameOver {
        //     // TODO send (event) game over
        //     return;
        // }

        // TODO send (event) draw
        // TODO request (intent)
        self.handle_mulligan(Player::Two)?;
        // if self.active_step == Step::GameOver {
        //     // TODO send (event) game over
        //     return;
        // }

        // - place debut member center face down
        // TODO member hide
        // TODO send (event) put main stage
        // TODO request (intent)
        println!("prompt debut 1");
        let debut_1 = self.prompt_for_first_debut(Player::One);
        // self.player_1.send_to_zone(debut_1, Zone::MainStageCenter);
        self.send_from_hand_to_center_stage(Player::One, debut_1)?;

        // TODO member hide
        // TODO send (event) put main stage
        // TODO request (intent)
        println!("prompt debut 2");
        let debut_2 = self.prompt_for_first_debut(Player::Two);
        // self.player_2.send_to_zone(debut_2, Zone::MainStageCenter);
        self.send_from_hand_to_center_stage(Player::Two, debut_2)?;

        // - place other debut / spot members back stage
        // TODO member hide
        // TODO send (event) put back stage
        // TODO request (intent)
        println!("prompt other debut 1");
        let other_debut_1: Vec<_> = self.prompt_for_first_back_stage(Player::One);
        // self.player_1
        //     .send_many_to_zone(other_debut_1, Zone::BackStage);
        self.send_from_hand_to_back_stage(Player::One, other_debut_1)?;

        // TODO member hide
        // TODO send (event) put back stage
        // TODO request (intent)
        println!("prompt other debut 2");
        let other_debut_2: Vec<_> = self.prompt_for_first_back_stage(Player::Two);
        // self.player_2
        //     .send_many_to_zone(other_debut_2, Zone::BackStage);
        self.send_from_hand_to_back_stage(Player::Two, other_debut_2)?;

        // - reveal face down oshi and members
        // oshi and members reveal
        // TODO send (event) reveal
        self.reveal_all_cards_in_zone(Player::One, Zone::MainStageOshi)?;
        self.reveal_all_cards_in_zone(Player::Two, Zone::MainStageOshi)?;
        self.reveal_all_cards_in_zone(Player::One, Zone::MainStageCenter)?;
        self.reveal_all_cards_in_zone(Player::Two, Zone::MainStageCenter)?;
        self.reveal_all_cards_in_zone(Player::One, Zone::BackStage)?;
        self.reveal_all_cards_in_zone(Player::Two, Zone::BackStage)?;

        // - draw life cards face down from cheer
        // TODO send (event) put life
        let oshi_1 = self
            .player_1
            .oshi
            .and_then(|c| self.lookup_oshi(c))
            .expect("oshi should always be there");
        // self.player_1.add_life(oshi_1.life);
        self.send_cheers_to_life(Player::One, oshi_1.life as usize)?;

        // TODO send (event) put life
        let oshi_2 = self
            .player_2
            .oshi
            .and_then(|c| self.lookup_oshi(c))
            .expect("oshi should always be there");
        // self.player_2.add_life(oshi_2.life);
        self.send_cheers_to_life(Player::Two, oshi_2.life as usize)?;

        // - game start
        // TODO send (event) game start
        self.report_start_game(self.active_player)?;

        Ok(GameContinue)
    }

    pub fn next_step(&mut self) -> GameResult {
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
                println!("the game is over");
                // TODO send (event) game over
                return Err(GameOutcome {
                    // TODO report draws, or stored outcome
                    winning_player: Some(self.active_player),
                });
            }
        };

        // start turn
        if self.active_step == Step::Reset {
            self.start_turn()?;
        }

        // could maybe lose between start turn and start step, because of an effect

        println!("- active step: {:?}", self.active_step);
        self.report_enter_step(self.active_player, self.active_step)?;

        // after step change, before the step starts
        // self.check_loss_conditions();
        // if self.active_step == Step::GameOver {
        //     // TODO send (event) game over
        //     return false;
        // }

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
                return Err(GameOutcome {
                    // TODO report draws, or stored outcome
                    winning_player: Some(self.active_player),
                });
            }
        }?;

        // after each step
        // self.check_loss_conditions();
        // if self.active_step == Step::GameOver {
        //     // TODO send (event) game over
        //     return false;
        // }

        self.report_exit_step(self.active_player, self.active_step)?;

        // end turn
        if self.active_step == Step::End {
            self.end_turn()?;
        }

        // could lose after the end of the turn
        // self.check_loss_conditions();
        // if self.active_step == Step::GameOver {
        //     // TODO send (event) game over
        //     return false;
        // }

        Ok(GameContinue)
    }

    pub fn start_turn(&mut self) -> GameResult {
        self.turn_number += 1;

        // TODO send (event) start turn, before or after modifiers?
        println!("active player: {:?}", self.active_player);
        self.report_start_turn(self.active_player)?;

        Ok(GameContinue)
    }

    pub fn end_turn(&mut self) -> GameResult {
        // TODO send (event) end turn, before or after modifiers?
        self.report_end_turn(self.active_player)?;

        Ok(GameContinue)
    }

    pub fn reset_step(&mut self) -> GameResult {
        // TODO send (event) start step
        // TODO (trigger) step change effects

        // - all members from rest to active
        // TODO send (event) awake card, remove modifiers
        for mem in self.active_board().stage().collect_vec() {
            self.remove_all_modifiers(mem, Resting)?;
        }

        // - collab to back stage in rest
        // TODO send (event) rest card, add modifiers
        if let Some(mem) = self.active_board().collab_stage {
            self.add_modifier(mem, Resting, LifeTime::UntilRemoved)?;
            // self.active_board_mut().send_to_zone(mem, Zone::BackStage);
            self.send_from_collab_to_back_stage(self.active_player, mem)?;
        }
        // - if no center, back stage to center
        if self.active_board().center_stage.is_none() && !self.active_board().back_stage.is_empty()
        {
            println!("prompt new center member");
            // TODO send (event) put main stage
            // TODO request (intent)
            let back = self.prompt_for_back_stage_to_center(self.active_player);
            // self.active_board_mut()
            //     .send_to_zone(back, Zone::MainStageCenter);
            self.send_from_back_stage_to_center_stage(self.active_player, back)?;
        }

        //   - no back stage lose game
        if self.active_board().center_stage.is_none() {
            // self.check_loss_conditions();
            // return;
        }

        // TODO send (event) end step

        Ok(GameContinue)
    }

    pub fn draw_step(&mut self) -> GameResult {
        // TODO send (event) start step
        // TODO (trigger) step change effects

        // - deck is 0 on draw step
        if self.active_board().main_deck.count() == 0 {
            println!(
                "player {:?} has no card in their main deck",
                self.active_player
            );
            return self.lose_game();
        }

        // - draw 1 card from main deck
        // TODO send (event) draw card
        // self.active_board_mut().draw(1);
        self.draw_from_main_deck(self.active_player, 1)?;

        // TODO send (event) end step

        Ok(GameContinue)
    }

    pub fn cheer_step(&mut self) -> GameResult {
        // TODO send (event) start step
        // TODO (trigger) step change effects

        // - draw 1 card from cheer deck, attach it
        // TODO send (event) peek cheer ?
        // TODO request (intent) select member
        // TODO send (event) attach / move cheer (player, card, from zone / card, target zone / card)
        // if let Some(cheer) = self.active_board().cheer_deck.peek_top_card() {
        //     println!("prompt member for cheer");
        //     println!("attach a cheer: {}", CardDisplay::new(cheer, self));
        //     //   - main stage or back stage
        //     if let Some(mem) = self.prompt_for_cheer(self.active_player) {
        //         self.active_board_mut().attach_to_card(cheer, mem);
        //     }
        // }
        self.attach_cheers_from_zone(self.active_player, Zone::CheerDeck, 1)?;

        // TODO send (event) end step

        Ok(GameContinue)
    }

    pub fn main_step(&mut self) -> GameResult {
        // TODO send (event) start step
        // TODO (trigger) step change effects

        loop {
            println!("prompt main step action");
            println!("{} cards in hand", self.active_board().hand().count());

            // TODO request (intent) main action, all possible actions
            match self.prompt_for_main_action(self.active_player) {
                MainStepAction::BackStageMember(card) => {
                    // TODO send (event) put back stage
                    // TODO request (intent) ? not if it's already in action intent
                    println!("- action: Back stage member");
                    // TODO verify back stage member action

                    // - place debut member on back stage
                    // self.active_board_mut().send_to_zone(card, Zone::BackStage);
                    self.send_from_hand_to_back_stage(self.active_player, vec![card])?;

                    // TODO maybe register for any abilities that could trigger?
                    // TODO remove the registration once they leave the board?
                }
                MainStepAction::BloomMember(bloom) => {
                    // TODO send (event) bloom member, specific event or attach? 2 events? flag? (player, card, from zone, target)
                    // TODO request (intent) ? not if it's already in action intent
                    println!("- action: Bloom member");
                    // TODO verify bloom member action

                    // - bloom member (evolve e.g. debut -> 1st )
                    //   - bloom effect
                    //   - can't bloom on same turn as placed
                    let card = self.prompt_for_bloom(self.active_player, bloom);
                    // self.bloom_member(bloom, card);
                    self.bloom_holo_member(self.active_player, bloom, card)?;
                }
                MainStepAction::UseSupportCard(card) => {
                    println!("- action: Use support card");
                    // TODO send (event) use card effect, support
                    // TODO verify use support card action

                    // - use support card
                    //   - only one limited per turn
                    //   - otherwise unlimited
                    let sup = self
                        .lookup_support(card)
                        .expect("only support should be allowed here");

                    let condition = sup.condition.clone();
                    let effect = sup.effect.clone();
                    if condition.start_evaluate(self, card) {
                        effect.start_evaluate(self, card);

                        // send the used card to the archive
                        // TODO send (event) put in archive
                        self.send_cards_to_archive(self.active_player, vec![card])?;
                    } else {
                        unreachable!("support should not be an option, if it's not allowed")
                    }
                }
                MainStepAction::CollabMember(card) => {
                    println!("- action: Collab member");
                    // TODO verify collab member action
                    if self.has_modifier(card, Resting) {
                        panic!("cannot collab a resting member");
                    }
                    if self.has_modifier(card, PreventCollab) {
                        panic!("cannot collab this member");
                    }

                    // can only collab from back stage
                    assert_eq!(
                        self.board_for_card(card).find_card_zone(card),
                        Some(Zone::BackStage)
                    );

                    // - put back stage member in collab
                    //   - can be done on first turn?
                    //   - draw down card from deck into power zone
                    // TODO send (event) put in collab
                    // self.active_board_mut()
                    //     .send_to_zone(card, Zone::MainStageCollab);
                    // TODO send (event) draw to holo power
                    // self.active_board_mut()
                    //     .send_from_zone(Zone::MainDeck, Zone::HoloPower, 1);
                    self.send_from_back_stage_to_collab(self.active_player, card)?;
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
                    // TODO request (intent) select attached cheers
                    // TODO send (event) send attached cheer to archive
                    // self.pay_baton_pass_cost(center);
                    // TODO request (intent) select back stage member
                    let back = self.prompt_for_back_stage_to_center(self.active_player);
                    // swap members
                    // TODO send (event) send member to back stage
                    // self.active_board_mut()
                    //     .send_to_zone(center, Zone::BackStage);
                    // TODO send (event) send member to center stage
                    // self.active_board_mut()
                    //     .send_to_zone(card, Zone::MainStageCenter);
                    self.baton_pass_center_stage_to_back_stage(self.active_player, center, back)?;
                }
                MainStepAction::UseAbilities(card, i) => {
                    println!("- action: Use abilities");
                    // TODO verify use abilities action

                    let oshi = self
                        .lookup_oshi(card)
                        .expect("only oshi should be using abilities");

                    // - use abilities (including oshi)
                    //   - oshi power uses card in power zone
                    //   - once per turn / once per game?
                    // TODO prevent duplicate ability use with (buff) (condition)
                    let condition = oshi.skills[i].condition.clone();
                    let cost = oshi.skills[i].cost.into();
                    let effect = oshi.skills[i].effect.clone();
                    let prevent_life_time = match oshi.skills[i].kind {
                        OshiSkillKind::Normal => LifeTime::ThisTurn,
                        OshiSkillKind::Special => LifeTime::ThisGame,
                    };

                    // TODO could have a buff that could pay for the skill
                    if self.active_board().holo_power.count() < cost {
                        unreachable!(
                            "oshi ability should not be an option, if the cost could not be payed"
                        )
                    }

                    if condition.start_evaluate(self, card) {
                        // pay the cost of the oshi ability
                        // TODO send (event) send holo power to archive
                        self.active_board_mut().send_from_zone(
                            Zone::HoloPower,
                            Zone::Archive,
                            cost,
                        );

                        effect.start_evaluate(self, card);

                        self.add_modifier(card, PreventAbility(i), prevent_life_time)?;
                    } else {
                        unreachable!("oshi ability should not be an option, if it's not allowed")
                    }
                }
                MainStepAction::Done => {
                    println!("- action: Done");
                    break;
                }
            }

            // self.check_loss_conditions();
            // if self.active_step == Step::GameOver {
            //     // TODO send (event) game over
            //     return;
            // }
        }

        // attack can be preloaded at this point

        // TODO send (event) end step

        Ok(GameContinue)
    }

    pub fn performance_step(&mut self) -> GameResult {
        // TODO send (event) start step
        // TODO (trigger) step change effects

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
                let mem = self
                    .lookup_holo_member(card)
                    .expect("this should be a valid member");

                let condition = mem.arts[art_idx].condition.clone();
                let effect = mem.arts[art_idx].effect.clone();

                // FIXME check cheer requirement

                // FIXME evaluate damage number
                let damage = match mem.arts[art_idx].damage {
                    HoloMemberArtDamage::Basic(dmg) => dmg,
                    HoloMemberArtDamage::Plus(_) => todo!(),
                    HoloMemberArtDamage::Minus(_) => todo!(),
                    HoloMemberArtDamage::Uncertain => todo!(),
                };

                // can the art be performed
                if condition.start_evaluate(self, card) {
                    // FIXME evaluate damage number
                    println!("deals {:?} damage", damage);
                    self.add_damage(target, DamageMarkers::from_hp(damage))?;

                    effect.start_evaluate(self, card);
                }

                if self.remaining_hp(target) == 0 {
                    // TODO send (event) send member to archive, from attack
                    self.send_cards_to_archive(self.player_for_card(target), vec![target])?;

                    // TODO send (event) lose life
                    // TODO request (intent) select member
                    // TODO send (event) attach cheer to member
                    // TODO buzz lose 2 lives
                    self.lose_lives(self.player_for_card(target), 1)?;
                    // self.check_loss_conditions();
                    // if self.active_step == Step::GameOver {
                    //     return;
                    // }
                }
            }
        }

        // TODO send (event) end step

        Ok(GameContinue)
    }

    pub fn end_step(&mut self) -> GameResult {
        // TODO send (event) start step
        // TODO (trigger) step change effects

        // - any end step effect

        // - if no center, back stage to center
        if self.active_board().center_stage.is_none() && !self.active_board().back_stage.is_empty()
        {
            println!("prompt new center member");
            // TODO send (event) put main stage
            // TODO request (intent)
            let back = self.prompt_for_back_stage_to_center(self.active_player);
            // self.active_board_mut()
            //     .send_to_zone(back, Zone::MainStageCenter);
            self.send_from_back_stage_to_center_stage(self.active_player, back)?;
        }

        // TODO send (event) end step

        Ok(GameContinue)
    }

    pub fn win_game(&mut self) -> GameResult {
        match self.active_player {
            Player::One => println!("player 1 wins"),
            Player::Two => println!("player 2 wins"),
            _ => unreachable!("both players cannot be active at the same time"),
        };
        // stop the game
        self.active_step = Step::GameOver;
        // TODO send (event) win game
        // TODO send (event) game over
        self.report_game_over(self.active_player)?;

        Err(GameOutcome {
            winning_player: Some(self.active_player),
        })
    }
    pub fn lose_game(&mut self) -> GameResult {
        self.active_player = match self.active_player {
            Player::One => Player::Two,
            Player::Two => Player::One,
            _ => unreachable!("both players cannot be active at the same time"),
        };
        self.win_game()
    }

    pub fn check_loss_conditions(&mut self) -> GameResult {
        // cannot lose in setup, except from mulligan
        if self.active_step == Step::Setup {
            return Ok(GameContinue);
        }

        // already game over
        // if self.active_step == Step::GameOver {
        //     return;
        // }

        let mut lose_player = None;
        for (player, board) in [(Player::One, &self.player_1), (Player::Two, &self.player_2)] {
            // - life is 0
            if board.life.count() == 0 {
                lose_player = Some(player);
                println!("player {player:?} has no life remaining");
                // TODO send (event) game over reason
            }

            // - 0 member in play
            if board.main_stage().count() + board.back_stage().count() == 0 {
                lose_player = Some(player);
                println!("player {player:?} has no more members on stage");
                // TODO send (event) game over reason
            }

            if lose_player.is_some() {
                break;
            }
        }

        if let Some(lose_player) = lose_player {
            self.active_player = lose_player;
            return self.lose_game();
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

    // pub fn bloom_member(&mut self, bloom: CardRef, card: CardRef) {
    //     self.active_board_mut().attach_to_card(bloom, card);
    //     self.active_board_mut().promote_attachment(bloom, card);
    //     self.promote_modifiers(bloom, card);
    // }

    // pub fn pay_baton_pass_cost(&mut self, card: CardRef) {
    //     let mem = self
    //         .lookup_holo_member(card)
    //         .expect("cannot pay baton pass cost for non member");
    //     // TODO cost should automatic when there is a single cheers color
    //     let cheers = self.prompt_for_baton_pass(card, mem.baton_pass_cost);
    //     self.active_board_mut().remove_many_attachments(cheers);
    // }

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

    // pub fn send_to_archive(&mut self, card: CardRef) {
    //     self.board_for_card_mut(card)
    //         .send_to_zone(card, Zone::Archive);
    //     self.board_for_card_mut(card)
    //         .send_all_attachments_to_zone(card, Zone::Archive);
    //     self.clear_all_modifiers(card);
    // }

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

    pub fn prompt_for_back_stage_to_center(&mut self, player: Player) -> CardRef {
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

    #[allow(clippy::unnecessary_filter_map)]
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
                        if count < MAX_MEMBERS_ON_STAGE {
                            Some(MainStepAction::BackStageMember(c))
                        } else {
                            None
                        }
                    }
                    HoloMemberLevel::First | HoloMemberLevel::Second => {
                        // TODO this is duplicated in the bloom prompt
                        // check condition for bloom
                        let bloom_lookup = (
                            match m.level {
                                HoloMemberLevel::Debut | HoloMemberLevel::Spot => {
                                    panic!("can only bloom from first or second")
                                }
                                // will match on multiple names, if the card has them
                                HoloMemberLevel::First => {
                                    vec![HoloMemberLevel::Debut, HoloMemberLevel::First]
                                }
                                // TODO verify 2nd -> 2nd
                                HoloMemberLevel::Second => vec![HoloMemberLevel::First],
                            },
                            m.names().collect_vec(),
                        );
                        let can_bloom = self
                            .board(player)
                            .stage()
                            .filter_map(|c| self.lookup_holo_member(c))
                            // TODO cannot bloom to remaining hp < 0
                            // TODO cannot bloom on each player first turn
                            // TODO cannot bloom the member twice  in a turn
                            // TODO not sure if the name needs to be consistent with debut? e.i. Sora -> Sora/AZKi -> AZKi
                            .any(|m| {
                                // match on level
                                bloom_lookup.0.iter().any(|r| *r == m.level)
                                // match on name
                                    && bloom_lookup
                                        .1
                                        .iter()
                                        .any(|n| m.names().any(|m_n| *n == m_n))
                            });
                        if can_bloom {
                            Some(MainStepAction::BloomMember(c))
                        } else {
                            None
                        }
                    }
                },
                // TODO check condition to play support
                Card::Support(_) => Some(MainStepAction::UseSupportCard(c)),
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
                    Card::HoloMember(_) => {
                        if self.board(player).collab_stage.is_none() {
                            Some(MainStepAction::CollabMember(c))
                        } else {
                            None
                        }
                    }
                    Card::Support(_) => unreachable!("support cannot be in the back stage"),
                    Card::Cheer(_) => unreachable!("cheer cannot be in the back stage"),
                })
                .map(|a| MainStepActionDisplay::new(a, self)),
        );
        // baton pass
        actions.extend(
            self.board(player)
                .center_stage()
                .filter_map(|c| {
                    match self.lookup_card(c) {
                        Card::OshiHoloMember(_) => {
                            unreachable!("oshi cannot be in the center stage")
                        }
                        Card::HoloMember(m) => {
                            // check condition for baton pass
                            let cheer_count = self.attached_cheers(c).count();
                            let back_stage_count = self
                                .board(player)
                                .back_stage()
                                .filter_map(|c| self.lookup_holo_member(c))
                                .count();

                            if cheer_count >= m.baton_pass_cost as usize && back_stage_count > 0 {
                                Some(MainStepAction::BatonPass(c))
                            } else {
                                None
                            }
                        }
                        Card::Support(_) => unreachable!("support cannot be in the center stage"),
                        Card::Cheer(_) => unreachable!("cheer cannot be in the center stage"),
                    }
                })
                .map(|a| MainStepActionDisplay::new(a, self)),
        );
        // abilities
        actions.extend(
            self.board(player)
                .oshi
                .iter()
                .flat_map(|c| {
                    match self.lookup_card(*c) {
                        Card::OshiHoloMember(o) => o
                            .skills
                            .iter()
                            .enumerate()
                            .filter(|(_, a)| self.board(player).holo_power.count() >= a.cost.into())
                            // TODO check condition for ability
                            // prevent duplicate ability use
                            .filter(|(i, _)| !self.has_modifier(*c, PreventAbility(*i)))
                            .map(|(i, _a)| MainStepAction::UseAbilities(*c, i))
                            .collect_vec(),
                        Card::HoloMember(_) => todo!("members are not in oshi position"),
                        Card::Support(_) => todo!("supports are not in oshi position"),
                        Card::Cheer(_) => todo!("cheers are not in oshi position"),
                    }
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
        // TODO extract that filtering to a reusable function
        let Card::HoloMember(bloom) = self.lookup_card(card) else {
            panic!("can only bloom from member")
        };

        let bloom_lookup = (
            match bloom.level {
                HoloMemberLevel::Debut | HoloMemberLevel::Spot => {
                    panic!("can only bloom from first or second")
                }
                // will match on multiple names, if the card has them
                HoloMemberLevel::First => vec![HoloMemberLevel::Debut, HoloMemberLevel::First],
                // TODO verify 2nd -> 2nd
                HoloMemberLevel::Second => vec![HoloMemberLevel::First],
            },
            bloom.names().collect_vec(),
        );

        let stage: Vec<_> = self
            .board(player)
            .stage()
            .filter_map(|c| self.lookup_holo_member(c).map(|m| (c, m)))
            // TODO cannot bloom to remaining hp < 0
            // TODO cannot bloom on each player first turn
            // TODO cannot bloom the member twice  in a turn
            // TODO not sure if the name needs to be consistent with debut? e.i. Sora -> Sora/AZKi -> AZKi
            .filter(|(_, m)| {
                // match on level
                bloom_lookup.0.iter().any(|r| *r == m.level)
                // match on name
                    && bloom_lookup
                        .1
                        .iter()
                        .any(|n| m.names().any(|m_n| *n == m_n))
            })
            // TODO check for rest?
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
        // TODO extract that filtering to a reusable function
        if let Some(mem) = self.lookup_holo_member(card) {
            // breaking into separate part to appease the borrow checker
            let arts: Vec<_> = mem
                .arts
                .iter()
                .enumerate()
                .filter(|(_, a)| self.required_attached_cheers(card, &a.cost))
                .map(|(i, a)| (i, a.condition.clone()))
                .collect();
            let arts: Vec<_> = arts
                .into_iter()
                .filter(|(_, cond)| cond.clone().start_evaluate(self, card))
                .collect();
            let arts: Vec<_> = arts
                .into_iter()
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
}

#[derive(Debug)]
pub struct GameBoard {
    main_deck: Vec<CardRef>,
    oshi: Option<CardRef>,
    center_stage: Option<CardRef>,
    collab_stage: Option<CardRef>,
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
            main_deck: loadout
                .main_deck
                .iter()
                .map(|c| register_card(player, c, card_map))
                .collect(),
            oshi: Some(register_card(player, &loadout.oshi, card_map)),
            center_stage: None,
            collab_stage: None,
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
            activate_support: Vec::new(),
            attachments: HashMap::new(),
        }
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
    pub fn collab_stage(&self) -> impl Iterator<Item = CardRef> + '_ {
        self.collab_stage.iter().copied()
    }
    pub fn main_stage(&self) -> impl Iterator<Item = CardRef> + '_ {
        self.center_stage().chain(self.collab_stage())
    }
    pub fn back_stage(&self) -> impl Iterator<Item = CardRef> + '_ {
        self.back_stage.iter().copied()
    }

    // pub fn draw(&mut self, amount: usize) {
    //     self.send_from_zone(Zone::MainDeck, Zone::Hand, amount);
    // }

    // pub fn add_life(&mut self, amount: u8) {
    //     self.send_from_zone(Zone::CheerDeck, Zone::Life, amount.into());
    // }

    pub fn is_attached(&self, attachment: CardRef) -> bool {
        self.attachments.contains_key(&attachment)
    }
    pub fn is_attached_to(&self, attachment: CardRef, card: CardRef) -> bool {
        self.attachments.get(&attachment) == Some(&card)
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
        self.send_to_zone(attachment, Zone::Archive);
    }
    pub fn remove_many_attachments(&mut self, attachments: impl IntoIterator<Item = CardRef>) {
        self.send_many_to_zone(attachments, Zone::Archive);
    }

    pub fn send_all_attachments_to_zone(&mut self, card: CardRef, target_zone: Zone) {
        let attached = self.attachments(card);
        self.send_many_to_zone(attached, target_zone)
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

    pub fn send_to_zone(&mut self, card: CardRef, target_zone: Zone) {
        let current_zone = self.find_card_zone(card);
        if let Some(zone) = current_zone {
            self.get_zone_mut(zone).remove_card(card);
            self.get_zone_mut(target_zone).add_top_card(card);
        } else if self.is_attached(card) {
            self.attachments.remove(&card);
            self.get_zone_mut(target_zone).add_top_card(card);
        }
    }

    pub fn send_many_to_zone(
        &mut self,
        cards: impl IntoIterator<Item = CardRef>,
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

    pub fn get_zone(&self, zone: Zone) -> &dyn ZoneControl {
        match zone {
            Zone::All => unreachable!("a card cannot be in all zones"),
            Zone::Temporary => unimplemented!("no temp zone yet"),
            Zone::MainDeck => &self.main_deck,
            Zone::MainStageOshi => &self.oshi,
            Zone::MainStageCenter => &self.center_stage,
            Zone::MainStageCollab => &self.collab_stage,
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
            Zone::All => unreachable!("a card cannot be in all zones"),
            Zone::Temporary => unimplemented!("no temp zone yet"),
            Zone::MainDeck => &mut self.main_deck,
            Zone::MainStageOshi => &mut self.oshi,
            Zone::MainStageCenter => &mut self.center_stage,
            Zone::MainStageCollab => &mut self.collab_stage,
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
            Some(Zone::MainStageOshi)
        } else if self.center_stage.is_in_zone(card) {
            Some(Zone::MainStageCenter)
        } else if self.collab_stage.is_in_zone(card) {
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
    Temporary, // used for playing support cards
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
    ActivateSupport,
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
    UseAbilities(CardRef, usize),
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
            MainStepAction::UseAbilities(card, idx) => {
                let display = CardDisplay::new(card, game);
                let ability = match game.lookup_card(card) {
                    Card::OshiHoloMember(o) => o
                        .skills
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
            Card::OshiHoloMember(o) => format!("{} ({} life)", o.name, o.life),
            Card::HoloMember(m) => {
                format!(
                    "{} ({:?}) ({}/{}) ({} cheers) (?) ({})",
                    m.name,
                    m.level,
                    game.remaining_hp(card),
                    m.hp,
                    game.attached_cheers(card)
                        .filter_map(|c| game.lookup_cheer(c))
                        .map(|c| format!("{:?}", c.color))
                        .collect_vec()
                        .join(", "),
                    m.card_number
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
                "{} ({:?}) ({})",
                art.name,
                art.damage,
                art.cost
                    .iter()
                    .map(|c| format!("{c:?}"))
                    .collect_vec()
                    .join(", "),
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
