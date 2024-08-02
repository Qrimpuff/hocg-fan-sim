use crate::{
    gameplay::{CardRef, Game, Player, Rps, Step, Zone},
    modifiers::{DamageMarkers, Modifier, ModifierKind},
    CardNumber, Loadout,
};
use enum_dispatch::enum_dispatch;
use iter_tools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientReceive {
    Event(Event),
    IntentRequest(IntentRequest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientSend {
    IntentResponse(IntentResponse),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trigger<'a> {
    // maybe individual variants, or a container?, maybe only used for triggers, not for network?
    Before(&'a Event),
    After(&'a Event),
}

#[enum_dispatch(EvaluateEvent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    // Basic events
    Setup,
    Shuffle,
    RpsOutcome,
    PlayerGoingFirst,
    Reveal,

    GameStart,
    GameOver,
    StartTurn,
    EndTurn,
    EnterStep,
    ExitStep,

    AddCardModifiers,
    RemoveCardModifiers,
    AddZoneModifiers,
    RemoveZoneModifiers,
    AddDamageMarkers,
    RemoveDamageMarkers,
    ClearDamageMarkers,

    LookAndSelect,
    ZoneToZone,
    ZoneToAttach,
    AttachToAttach,
    AttachToZone,

    /// marker event before zone to attach (deck -> hand)
    Draw,
    /// marker event after zone to zone (back stage -> collab stage)
    Collab,
    /// marker event before zone to attach (life -> temp zone -> attach)
    LoseLives,
    /// marker event before zone to attach (hand -> attach)
    Bloom,

    BatonPass,
    UseSupportCard,

    PerformArt,
    WaitingForPlayerIntent,

    // Card effect events
    //...
    /// used by Lui oshi skill
    UseAbilitySkill,
    /// used by Pekora oshi skill, marker event before zone to zone
    HoloMemberDefeated,
    /// used by Suisei oshi skill
    DealDamage,
    /// used by AZKi oshi skill
    RollDice,
}

// Basic events

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentRequest {
    // basic intents
    Rps {
        player: Player,
        select_rps: Vec<Rps>,
    },
    Mulligan {
        player: Player,
        select_yes_no: Vec<bool>,
    },
    // FirstDebut,      // zone to zone
    // FirstBackStage,  // zone to zone
    // BackStageCenter, // zone to zone
    LookSelectZoneToZone {
        player: Player,
        from_zone: Zone,
        to_zone: Zone,
        look_cards: Vec<CardRef>,
        select_cards: Vec<CardRef>,
        amount: usize,
    },
    // MemberForCheer, // attach target
    SelectToAttach {
        player: Player,
        zones: Vec<Zone>,
        select_cards: Vec<CardRef>,
    },
    MainStepAction {
        player: Player,
        select_actions: Vec<MainStepAction>,
    },
    // HandToBackStage,
    // BloomMember
    // UseSupport
    // Collab
    // BatonPass
    // CheerForCost, // remove attach
    SelectAttachments {
        player: Player,
        card: (Zone, CardRef),
        select_attachments: Vec<CardRef>,
        amount: usize,
    },
    // UseOshiSkill,
    PerformanceStepAction {
        player: Player,
        select_actions: Vec<PerformanceStepAction>,
    },
    // PerformArt
    // ArtTarget

    // Card effect intents
    //...
    /// used by AZKi oshi skill
    RollDiceNumber {
        player: Player,
        select_numbers: Vec<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MainStepAction {
    BackStageMember {
        card: (Zone, CardRef),
    },
    BloomMember {
        card: (Zone, CardRef),
        target: (Zone, CardRef),
    },
    UseSupportCard {
        card: (Zone, CardRef),
    },
    CollabMember {
        card: (Zone, CardRef),
    },
    BatonPass {
        card: (Zone, CardRef),
    },
    UseSkill {
        card: (Zone, CardRef),
        skill_idx: usize,
    },
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerformanceStepAction {
    UseArt {
        card: (Zone, CardRef),
        art_idx: usize,
        target: (Zone, CardRef),
    },
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentResponse {
    // basic intents
    Rps {
        player: Player,
        select_rps: Rps,
    },
    Mulligan {
        player: Player,
        select_yes_no: bool,
    },
    LookSelectZoneToZone {
        player: Player,
        select_cards: Vec<CardRef>,
    },
    SelectToAttach {
        player: Player,
        select_card: CardRef,
    },
    MainStepAction {
        player: Player,
        select_action: MainStepAction,
    },
    SelectAttachments {
        player: Player,
        select_attachments: Vec<CardRef>,
    },
    PerformanceStepAction {
        player: Player,
        select_action: PerformanceStepAction,
    },

    // Card effect intents
    //...
    /// used by AZKi oshi skill
    RollDiceNumber {
        player: Player,
        select_number: usize,
    },
}

impl Game {
    pub fn send_event(&mut self, event: Event) {
        // TODO trigger before effects
        let before = Trigger::Before(&event);
        println!("  trigger = {before:?}");

        println!("EVENT = {event:?}");
        // perform the modification to the game state
        event.evaluate_event(self);

        // TODO sanitize the event before sending it to each player
        // TODO send event to each player

        // TODO trigger after effects
        let after = Trigger::After(&event);
        println!("  trigger = {after:?}");
    }

    pub fn report_rps_draw(&mut self) {
        self.send_event(
            RpsOutcome {
                winning_player: None,
            }
            .into(),
        )
    }
    pub fn report_rps_win(&mut self, player: Player) {
        self.send_event(
            RpsOutcome {
                winning_player: Some(player),
            }
            .into(),
        )
    }
    pub fn report_player_going_first(&mut self, player: Player) {
        self.send_event(
            PlayerGoingFirst {
                first_player: player,
            }
            .into(),
        )
    }

    pub fn report_start_game(&mut self, active_player: Player) {
        self.send_event(GameStart { active_player }.into())
    }
    pub fn report_game_over(&mut self, winning_player: Player) {
        self.send_event(
            GameOver {
                winning_player: Some(winning_player),
            }
            .into(),
        )
    }
    pub fn report_game_over_draw(&mut self) {
        self.send_event(
            GameOver {
                winning_player: None,
            }
            .into(),
        )
    }
    pub fn report_start_turn(&mut self, active_player: Player) {
        self.send_event(
            StartTurn {
                active_player,
                turn_number: self.turn_number,
            }
            .into(),
        )
    }
    pub fn report_end_turn(&mut self, active_player: Player) {
        self.send_event(
            EndTurn {
                active_player,
                turn_number: self.turn_number,
            }
            .into(),
        )
    }
    pub fn report_start_step(&mut self, active_player: Player, active_step: Step) {
        self.send_event(
            EnterStep {
                active_player,
                active_step,
            }
            .into(),
        )
    }
    pub fn report_end_step(&mut self, active_player: Player, active_step: Step) {
        self.send_event(
            ExitStep {
                active_player,
                active_step,
            }
            .into(),
        )
    }

    pub fn send_full_hand_to_main_deck(&mut self, player: Player) {
        let hand = self.board(player).get_zone(Zone::Hand);
        self.send_event(
            ZoneToZone {
                player,
                from_zone: Zone::Hand,
                cards: hand.peek_top_cards(hand.count()),
                to_zone: Zone::MainDeck,
            }
            .into(),
        );
    }

    pub fn shuffle_main_deck(&mut self, player: Player) {
        self.send_event(
            Shuffle {
                player,
                zone: Zone::MainDeck,
            }
            .into(),
        );
    }

    pub fn shuffle_cheer_deck(&mut self, player: Player) {
        self.send_event(
            Shuffle {
                player,
                zone: Zone::CheerDeck,
            }
            .into(),
        );
    }

    pub fn reveal_cards(&mut self, player: Player, zone: Zone, cards: &[CardRef]) {
        if cards.is_empty() {
            return;
        }

        self.send_event(
            Reveal {
                player,
                zone,
                cards: cards
                    .iter()
                    .map(|c| (*c, self.lookup_card_number(*c).clone()))
                    .collect(),
            }
            .into(),
        );
    }

    pub fn reveal_all_cards_in_zone(&mut self, player: Player, zone: Zone) {
        let cards = self.board(player).get_zone(zone).all_cards();
        self.reveal_cards(player, zone, &cards);
    }

    pub fn send_from_hand_to_center_stage(&mut self, player: Player, card: CardRef) {
        self.send_event(
            ZoneToZone {
                player,
                from_zone: Zone::Hand,
                cards: vec![card],
                to_zone: Zone::MainStageCenter,
            }
            .into(),
        );
    }

    pub fn send_from_back_stage_to_center_stage(&mut self, player: Player, card: CardRef) {
        self.send_event(
            ZoneToZone {
                player,
                from_zone: Zone::BackStage,
                cards: vec![card],
                to_zone: Zone::MainStageCenter,
            }
            .into(),
        );
    }

    pub fn send_from_hand_to_back_stage(&mut self, player: Player, cards: Vec<CardRef>) {
        if cards.is_empty() {
            return;
        }

        self.send_event(
            ZoneToZone {
                player,
                from_zone: Zone::Hand,
                cards,
                to_zone: Zone::BackStage,
            }
            .into(),
        );
    }

    pub fn send_cards_to_holo_power(&mut self, player: Player, amount: usize) {
        if amount < 1 {
            return;
        }

        let deck = self.board(player).get_zone(Zone::MainDeck);
        if deck.count() > 0 {
            self.send_event(
                ZoneToZone {
                    player,
                    from_zone: Zone::MainDeck,
                    cards: deck.peek_top_cards(amount),
                    to_zone: Zone::HoloPower,
                }
                .into(),
            );
        }
    }

    pub fn send_from_back_stage_to_collab(&mut self, player: Player, card: CardRef) {
        self.send_event(
            Collab {
                player,
                card: (Zone::BackStage, card),
                holo_power_amount: 1, // TODO some cards could maybe power for more
            }
            .into(),
        );
    }

    pub fn send_from_collab_to_back_stage(&mut self, player: Player, card: CardRef) {
        self.send_event(
            ZoneToZone {
                player,
                from_zone: Zone::MainStageCollab,
                cards: vec![card],
                to_zone: Zone::BackStage,
            }
            .into(),
        );
    }

    pub fn send_from_center_stage_to_back_stage(&mut self, player: Player, card: CardRef) {
        self.send_event(
            ZoneToZone {
                player,
                from_zone: Zone::MainStageCenter,
                cards: vec![card],
                to_zone: Zone::BackStage,
            }
            .into(),
        );
    }

    pub fn baton_pass_center_stage_to_back_stage(
        &mut self,
        player: Player,
        from_card: CardRef,
        to_card: CardRef,
    ) {
        self.send_event(
            BatonPass {
                player,
                from_card: (Zone::MainStageCenter, from_card),
                to_card: (Zone::BackStage, to_card),
            }
            .into(),
        );
    }

    pub fn send_cheers_to_life(&mut self, player: Player, amount: usize) {
        if amount < 1 {
            return;
        }

        let cheers = self.board(player).get_zone(Zone::CheerDeck);
        self.send_event(
            ZoneToZone {
                player,
                from_zone: Zone::CheerDeck,
                cards: cheers.peek_top_cards(amount),
                to_zone: Zone::Life,
            }
            .into(),
        );
    }

    pub fn send_cards_to_archive(&mut self, player: Player, cards: Vec<CardRef>) {
        if cards.is_empty() {
            return;
        }

        for (zone, cards) in self.board(player).group_cards_by_zone(&cards) {
            self.send_event(
                ZoneToZone {
                    player,
                    from_zone: zone,
                    cards,
                    to_zone: Zone::Archive,
                }
                .into(),
            );
        }
    }

    pub fn attach_cheers_from_zone(&mut self, player: Player, zone: Zone, amount: usize) {
        if amount < 1 {
            return;
        }

        // - draw cards from zone (cheer deck, life), then attach it
        let cheers = self.board(player).get_zone(zone).peek_top_cards(amount);
        self.reveal_cards(player, zone, &cheers);
        for cheer in cheers {
            // TODO package with prompt
            // println!("lost a life: {}", CardDisplay::new(cheer, self));

            if let Some(mem) = self.prompt_for_cheer(player) {
                let to_zone = self
                    .board(player)
                    .find_card_zone(mem)
                    .expect("the card should be in a zone");

                self.send_event(
                    ZoneToAttach {
                        player,
                        from_zone: zone,
                        attachments: vec![cheer],
                        to_card: (to_zone, mem),
                    }
                    .into(),
                );
            } else {
                self.send_event(
                    ZoneToZone {
                        player,
                        from_zone: zone,
                        cards: vec![cheer],
                        to_zone: Zone::Archive,
                    }
                    .into(),
                );
            }
        }
    }

    pub fn send_attachments_to_archive(
        &mut self,
        player: Player,
        card: CardRef,
        attachments: Vec<CardRef>,
    ) {
        if attachments.is_empty() {
            return;
        }

        let zone = self
            .board(player)
            .find_card_zone(card)
            .expect("the card should be in a zone");

        self.send_event(
            AttachToZone {
                player,
                from_card: (zone, card),
                attachments,
                to_zone: Zone::Archive,
            }
            .into(),
        );
    }

    pub fn add_many_modifiers_to_many_cards(
        &mut self,
        player: Player,
        zone: Zone,
        cards: Vec<CardRef>,
        modifiers: Vec<Modifier>,
    ) {
        if cards.is_empty() || modifiers.is_empty() {
            return;
        }

        self.send_event(
            AddCardModifiers {
                player,
                zone,
                cards,
                modifiers,
            }
            .into(),
        )
    }

    pub fn remove_many_modifiers_from_many_cards(
        &mut self,
        player: Player,
        zone: Zone,
        cards: Vec<CardRef>,
        modifiers: Vec<ModifierKind>,
    ) {
        if cards.is_empty() || modifiers.is_empty() {
            return;
        }

        self.send_event(
            RemoveCardModifiers {
                player,
                zone,
                cards,
                modifiers,
            }
            .into(),
        )
    }

    pub fn add_many_modifiers_to_zone(
        &mut self,
        player: Player,
        zone: Zone,
        modifiers: Vec<Modifier>,
    ) {
        if modifiers.is_empty() {
            return;
        }

        self.send_event(
            AddZoneModifiers {
                player,
                zone,
                modifiers,
            }
            .into(),
        )
    }

    pub fn remove_many_modifiers_from_zone(
        &mut self,
        player: Player,
        zone: Zone,
        modifiers: Vec<ModifierKind>,
    ) {
        if modifiers.is_empty() {
            return;
        }

        self.send_event(
            RemoveZoneModifiers {
                player,
                zone,
                modifiers,
            }
            .into(),
        )
    }

    pub fn add_damage_markers_to_many_cards(
        &mut self,
        player: Player,
        zone: Zone,
        cards: Vec<CardRef>,
        dmg: DamageMarkers,
    ) {
        if cards.is_empty() || dmg.0 < 1 {
            return;
        }

        self.send_event(
            AddDamageMarkers {
                player,
                zone,
                cards,
                dmg,
            }
            .into(),
        )
    }

    pub fn remove_damage_markers_from_many_cards(
        &mut self,
        player: Player,
        zone: Zone,
        cards: Vec<CardRef>,
        dmg: DamageMarkers,
    ) {
        let cards = cards
            .into_iter()
            .filter(|c| self.has_damage(*c))
            .collect_vec();

        if cards.is_empty() || dmg.0 < 1 {
            return;
        }

        self.send_event(
            RemoveDamageMarkers {
                player,
                zone,
                cards,
                dmg,
            }
            .into(),
        )
    }

    pub fn clear_all_damage_markers_from_many_cards(
        &mut self,
        player: Player,
        zone: Zone,
        cards: Vec<CardRef>,
    ) {
        let cards = cards
            .into_iter()
            .filter(|c| self.has_damage(*c))
            .collect_vec();

        if cards.is_empty() {
            return;
        }

        self.send_event(
            ClearDamageMarkers {
                player,
                zone,
                cards,
            }
            .into(),
        )
    }

    pub fn draw_from_main_deck(&mut self, player: Player, amount: usize) {
        if amount < 1 {
            return;
        }

        self.send_event(Draw { player, amount }.into());
    }

    pub fn lose_lives(&mut self, player: Player, amount: usize) {
        if amount < 1 {
            return;
        }

        self.send_event(LoseLives { player, amount }.into());
    }

    pub fn bloom_holo_member(&mut self, player: Player, bloom: CardRef, target: CardRef) {
        let stage = self
            .board(player)
            .find_card_zone(target)
            .expect("the card should be on stage");
        self.send_event(
            Bloom {
                player,
                // can only bloom from hand, for now
                from_card: (Zone::Hand, bloom),
                to_card: (stage, target),
            }
            .into(),
        );
    }
}

#[enum_dispatch]
trait EvaluateEvent {
    fn evaluate_event(&self, game: &mut Game);
}

fn verify_cards_in_zone(game: &Game, player: Player, zone: Zone, cards: &[CardRef]) {
    // from_zone is only there for client knowledge, the game knows where the card is
    let from_zone = game.board(player).get_zone(zone);
    let all_card_in_zone = cards.iter().all(|c| from_zone.is_in_zone(*c));
    if !all_card_in_zone {
        // println!("{game:#?}");
        // println!("{player:#?}");
        // println!("{zone:#?}");
        // println!("{cards:#?}");
        panic!("not all cards are in zone")
    }
}

fn verify_cards_attached(game: &Game, player: Player, card: CardRef, attachments: &[CardRef]) {
    // from_zone is only there for client knowledge, the game knows where the card is
    let board = game.board(player);
    let all_card_attached = attachments.iter().all(|a| board.is_attached_to(*a, card));
    if !all_card_attached {
        // println!("{game:#?}");
        // println!("{player:#?}");
        // println!("{card:#?}");
        // println!("{attachments:#?}");
        panic!("not all cards are attached")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setup {
    // send both decks loadout, private, asymmetric
    you: Player,
    player_1: Loadout,
    player_2: Loadout,
}
impl EvaluateEvent for Setup {
    fn evaluate_event(&self, _game: &mut Game) {
        // TODO implement
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shuffle {
    player: Player,
    zone: Zone,
}
impl EvaluateEvent for Shuffle {
    fn evaluate_event(&self, game: &mut Game) {
        let zone = game.board_mut(self.player).get_zone_mut(self.zone);
        zone.shuffle();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpsOutcome {
    winning_player: Option<Player>,
}
impl EvaluateEvent for RpsOutcome {
    fn evaluate_event(&self, _game: &mut Game) {
        // the winning player doesn't change the state of the game
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerGoingFirst {
    first_player: Player,
}
impl EvaluateEvent for PlayerGoingFirst {
    fn evaluate_event(&self, game: &mut Game) {
        game.active_player = self.first_player;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reveal {
    player: Player,
    zone: Zone,
    cards: Vec<(CardRef, CardNumber)>,
}
impl EvaluateEvent for Reveal {
    fn evaluate_event(&self, game: &mut Game) {
        if self.cards.is_empty() {
            return;
        }

        verify_cards_in_zone(
            game,
            self.player,
            self.zone,
            &self.cards.iter().map(|(c, _)| c).copied().collect_vec(),
        );

        // TODO implement for network
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameStart {
    active_player: Player,
}
impl EvaluateEvent for GameStart {
    fn evaluate_event(&self, game: &mut Game) {
        // the state changes on start turn
        game.active_player = self.active_player;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameOver {
    winning_player: Option<Player>,
}
impl EvaluateEvent for GameOver {
    fn evaluate_event(&self, _game: &mut Game) {
        // the state doesn't change on game over
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartTurn {
    active_player: Player,
    turn_number: usize,
}
impl EvaluateEvent for StartTurn {
    fn evaluate_event(&self, game: &mut Game) {
        game.active_player = self.active_player;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndTurn {
    active_player: Player,
    turn_number: usize,
}
impl EvaluateEvent for EndTurn {
    fn evaluate_event(&self, game: &mut Game) {
        assert_eq!(self.active_player, game.active_player);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnterStep {
    active_player: Player,
    active_step: Step,
}
impl EvaluateEvent for EnterStep {
    fn evaluate_event(&self, game: &mut Game) {
        assert_eq!(self.active_player, game.active_player);
        game.active_step = self.active_step;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitStep {
    active_player: Player,
    active_step: Step,
}
impl EvaluateEvent for ExitStep {
    fn evaluate_event(&self, game: &mut Game) {
        assert_eq!(self.active_player, game.active_player);
        assert_eq!(self.active_step, game.active_step);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddCardModifiers {
    player: Player,
    zone: Zone,
    cards: Vec<CardRef>,
    modifiers: Vec<Modifier>,
}
impl EvaluateEvent for AddCardModifiers {
    fn evaluate_event(&self, game: &mut Game) {
        if self.cards.is_empty() || self.modifiers.is_empty() {
            return;
        }

        verify_cards_in_zone(game, self.player, self.zone, &self.cards);

        for card in &self.cards {
            game.card_modifiers
                .entry(*card)
                .or_default()
                .extend(self.modifiers.iter().cloned());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveCardModifiers {
    player: Player,
    zone: Zone,
    cards: Vec<CardRef>,
    modifiers: Vec<ModifierKind>,
}
impl EvaluateEvent for RemoveCardModifiers {
    fn evaluate_event(&self, game: &mut Game) {
        if self.cards.is_empty() || self.modifiers.is_empty() {
            return;
        }

        verify_cards_in_zone(game, self.player, self.zone, &self.cards);

        let mut to_remove = self
            .cards
            .iter()
            .copied()
            .cartesian_product(self.modifiers.iter().cloned())
            .collect_vec();

        for card in &self.cards {
            game.card_modifiers.entry(*card).or_default().retain(|m| {
                let idx = to_remove
                    .iter()
                    .enumerate()
                    .find(|(_, r)| r.0 == *card && r.1 == m.kind)
                    .map(|(i, _)| i);
                if let Some(idx) = idx {
                    to_remove.swap_remove(idx);
                    false
                } else {
                    true
                }
            });
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddZoneModifiers {
    player: Player,
    zone: Zone,
    modifiers: Vec<Modifier>,
}
impl EvaluateEvent for AddZoneModifiers {
    fn evaluate_event(&self, game: &mut Game) {
        if self.modifiers.is_empty() {
            return;
        }

        game.zone_modifiers.entry(self.player).or_default().extend(
            self.modifiers
                .iter()
                .cloned()
                .map(|m: Modifier| (self.zone, m)),
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveZoneModifiers {
    player: Player,
    zone: Zone,
    modifiers: Vec<ModifierKind>,
}
impl EvaluateEvent for RemoveZoneModifiers {
    fn evaluate_event(&self, game: &mut Game) {
        if self.modifiers.is_empty() {
            return;
        }

        let mut to_remove = self
            .modifiers
            .iter()
            .cloned()
            .map(|m| (self.player, self.zone, m))
            .collect_vec();

        game.zone_modifiers
            .entry(self.player)
            .or_default()
            .retain(|(z, m)| {
                let idx = to_remove
                    .iter()
                    .enumerate()
                    .find(|(_, r)| r.0 == self.player && r.1 == *z && r.2 == m.kind)
                    .map(|(i, _)| i);
                if let Some(idx) = idx {
                    to_remove.swap_remove(idx);
                    false
                } else {
                    true
                }
            });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddDamageMarkers {
    player: Player,
    zone: Zone,
    cards: Vec<CardRef>,
    dmg: DamageMarkers,
}
impl EvaluateEvent for AddDamageMarkers {
    fn evaluate_event(&self, game: &mut Game) {
        if self.cards.is_empty() || self.dmg.0 < 1 {
            return;
        }

        verify_cards_in_zone(game, self.player, self.zone, &self.cards);

        for card in &self.cards {
            *game.card_damage_markers.entry(*card).or_default() += self.dmg;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveDamageMarkers {
    player: Player,
    zone: Zone,
    cards: Vec<CardRef>,
    dmg: DamageMarkers,
}
impl EvaluateEvent for RemoveDamageMarkers {
    fn evaluate_event(&self, game: &mut Game) {
        if self.cards.is_empty() || self.dmg.0 < 1 {
            return;
        }

        verify_cards_in_zone(game, self.player, self.zone, &self.cards);

        for card in &self.cards {
            *game.card_damage_markers.entry(*card).or_default() -= self.dmg;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearDamageMarkers {
    player: Player,
    zone: Zone,
    cards: Vec<CardRef>,
}
impl EvaluateEvent for ClearDamageMarkers {
    fn evaluate_event(&self, game: &mut Game) {
        if self.cards.is_empty() {
            return;
        }

        verify_cards_in_zone(game, self.player, self.zone, &self.cards);

        for card in &self.cards {
            game.card_damage_markers.remove_entry(card);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookAndSelect {
    player: Player,
    zone: Zone,
    cards: Vec<CardRef>, // could just be a count, it's just for the opponent
}
impl EvaluateEvent for LookAndSelect {
    fn evaluate_event(&self, game: &mut Game) {
        if self.cards.is_empty() {
            return;
        }

        verify_cards_in_zone(game, self.player, self.zone, &self.cards);

        // TODO implement
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneToZone {
    player: Player,
    from_zone: Zone,
    cards: Vec<CardRef>,
    to_zone: Zone,
}
impl EvaluateEvent for ZoneToZone {
    fn evaluate_event(&self, game: &mut Game) {
        if self.cards.is_empty() {
            return;
        }

        verify_cards_in_zone(game, self.player, self.from_zone, &self.cards);

        game.board_mut(self.player)
            .send_many_to_zone(self.cards.clone(), self.to_zone);

        // lose attachments and buffs when leaving stage
        if self.to_zone != Zone::MainStageCenter
            && self.to_zone != Zone::MainStageCollab
            && self.to_zone != Zone::BackStage
        {
            game.clear_all_damage_markers_from_many_cards(
                self.player,
                self.to_zone,
                self.cards.clone(),
            );

            for card in &self.cards {
                let attachments = game.board(self.player).attachments(*card);
                game.send_attachments_to_archive(self.player, *card, attachments);

                game.clear_all_modifiers(*card);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneToAttach {
    player: Player,
    from_zone: Zone,
    attachments: Vec<CardRef>,
    to_card: (Zone, CardRef),
}
impl EvaluateEvent for ZoneToAttach {
    fn evaluate_event(&self, game: &mut Game) {
        if self.attachments.is_empty() {
            return;
        }

        verify_cards_in_zone(game, self.player, self.from_zone, &self.attachments);
        verify_cards_in_zone(game, self.player, self.to_card.0, &[self.to_card.1]);

        for attachment in &self.attachments {
            game.board_mut(self.player)
                .attach_to_card(*attachment, self.to_card.1);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachToAttach {
    player: Player,
    from_card: (Zone, CardRef),
    attachments: Vec<CardRef>,
    to_card: (Zone, CardRef),
}
impl EvaluateEvent for AttachToAttach {
    fn evaluate_event(&self, game: &mut Game) {
        if self.attachments.is_empty() {
            return;
        }

        verify_cards_in_zone(game, self.player, self.from_card.0, &[self.from_card.1]);
        verify_cards_attached(game, self.player, self.from_card.1, &self.attachments);
        verify_cards_in_zone(game, self.player, self.to_card.0, &[self.to_card.1]);

        // TODO implement
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachToZone {
    player: Player,
    from_card: (Zone, CardRef),
    attachments: Vec<CardRef>,
    to_zone: Zone,
}
impl EvaluateEvent for AttachToZone {
    fn evaluate_event(&self, game: &mut Game) {
        if self.attachments.is_empty() {
            return;
        }

        verify_cards_in_zone(game, self.player, self.from_card.0, &[self.from_card.1]);
        verify_cards_attached(game, self.player, self.from_card.1, &self.attachments);

        game.board_mut(self.player)
            .send_many_to_zone(self.attachments.clone(), self.to_zone);
    }
}

/// marker event after zone to zone (deck -> hand)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draw {
    player: Player,
    amount: usize,
}
impl EvaluateEvent for Draw {
    fn evaluate_event(&self, game: &mut Game) {
        if self.amount < 1 {
            return;
        }

        let deck = game.board(self.player).get_zone(Zone::MainDeck);
        game.send_event(
            ZoneToZone {
                player: self.player,
                from_zone: Zone::MainDeck,
                cards: deck.peek_top_cards(self.amount),
                to_zone: Zone::Hand,
            }
            .into(),
        );
    }
}

/// marker event after zone to zone (back stage -> collab stage)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collab {
    player: Player,
    card: (Zone, CardRef),
    holo_power_amount: usize,
}
impl EvaluateEvent for Collab {
    fn evaluate_event(&self, game: &mut Game) {
        verify_cards_in_zone(game, self.player, self.card.0, &[self.card.1]);

        game.send_event(
            ZoneToZone {
                player: self.player,
                from_zone: self.card.0,
                cards: vec![self.card.1],
                to_zone: Zone::MainStageCollab,
            }
            .into(),
        );

        //   - draw down card from deck into power zone
        game.send_cards_to_holo_power(self.player, self.holo_power_amount);
    }
}

/// marker event before zone to attach (life -> temp zone -> attach)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoseLives {
    player: Player,
    amount: usize,
}
impl EvaluateEvent for LoseLives {
    fn evaluate_event(&self, game: &mut Game) {
        if self.amount < 1 {
            return;
        }

        game.attach_cheers_from_zone(self.player, Zone::Life, self.amount);
    }
}

/// marker event before zone to zone (deck -> hand)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bloom {
    player: Player,
    from_card: (Zone, CardRef),
    to_card: (Zone, CardRef),
}
impl EvaluateEvent for Bloom {
    fn evaluate_event(&self, game: &mut Game) {
        verify_cards_in_zone(game, self.player, self.from_card.0, &[self.from_card.1]);
        verify_cards_in_zone(game, self.player, self.to_card.0, &[self.to_card.1]);

        // game.send_event(
        //     ZoneToAttach {
        //         player: self.player,
        //         from_zone: self.from_zone,
        //         cards: vec![self.card],
        //         to_card: self.to_card,
        //     }
        //     .into(),
        // );
        game.board_mut(self.player)
            .attach_to_card(self.from_card.1, self.to_card.1);
        game.board_mut(self.player)
            .promote_attachment(self.from_card.1, self.to_card.1);
        game.promote_modifiers(self.from_card.1, self.to_card.1);
        game.promote_damage_markers(self.from_card.1, self.to_card.1);
    }
}

/// marker event after zone to zone (back stage -> collab stage)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatonPass {
    player: Player,
    from_card: (Zone, CardRef),
    to_card: (Zone, CardRef),
}
impl EvaluateEvent for BatonPass {
    fn evaluate_event(&self, game: &mut Game) {
        verify_cards_in_zone(game, self.player, self.from_card.0, &[self.from_card.1]);
        verify_cards_in_zone(game, self.player, self.to_card.0, &[self.to_card.1]);

        // only center stage can baton pass to back stage
        assert_eq!(self.from_card.0, Zone::MainStageCenter);
        assert_eq!(self.to_card.0, Zone::BackStage);

        // pay the baton pass cost
        let mem = game
            .lookup_holo_member(self.from_card.1)
            .expect("cannot pay baton pass cost for non member");
        // TODO cost should automatic when there is a single cheers color
        let cheers = game.prompt_for_baton_pass(self.from_card.1, mem.baton_pass_cost);
        game.send_attachments_to_archive(self.player, self.from_card.1, cheers);

        // send the center member to the back
        game.send_from_center_stage_to_back_stage(self.player, self.from_card.1);

        // send back stage member to center
        game.send_from_back_stage_to_center_stage(self.player, self.to_card.1);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseSupportCard {
    player: Player,
    card: (Zone, CardRef),
}
impl EvaluateEvent for UseSupportCard {
    fn evaluate_event(&self, game: &mut Game) {
        verify_cards_in_zone(game, self.player, self.card.0, &[self.card.1]);

        // TODO implement
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerformArt {
    player: Player,
    card: (Zone, CardRef),
    art_idx: usize,
    target: Option<(Zone, CardRef)>,
}
impl EvaluateEvent for PerformArt {
    fn evaluate_event(&self, game: &mut Game) {
        verify_cards_in_zone(game, self.player, self.card.0, &[self.card.1]);

        if let Some(target) = self.target {
            verify_cards_in_zone(game, self.player, target.0, &[target.1]);
        }
        // TODO implement
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitingForPlayerIntent {
    player: Player,
    // reason?
}
impl EvaluateEvent for WaitingForPlayerIntent {
    fn evaluate_event(&self, _game: &mut Game) {
        // TODO implement
        unimplemented!()
    }
}
// Card effect events
//...
/// used by Lui oshi skill
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseAbilitySkill {
    player: Player,
    card: (Zone, CardRef),
    skill_idx: usize,
}
impl EvaluateEvent for UseAbilitySkill {
    fn evaluate_event(&self, game: &mut Game) {
        verify_cards_in_zone(game, self.player, self.card.0, &[self.card.1]);

        // TODO implement
        unimplemented!()
    }
}
/// used by Pekora oshi skill, marker event before zone to zone
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoloMemberDefeated {
    player: Player,
    card: (Zone, CardRef),
}
impl EvaluateEvent for HoloMemberDefeated {
    fn evaluate_event(&self, game: &mut Game) {
        verify_cards_in_zone(game, self.player, self.card.0, &[self.card.1]);

        // TODO implement
        unimplemented!()
    }
}
/// used by Suisei oshi skill
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DealDamage {
    player: Player,
    card: (Zone, CardRef),
    target: (Zone, CardRef),
    amount: usize,
}
impl EvaluateEvent for DealDamage {
    fn evaluate_event(&self, game: &mut Game) {
        verify_cards_in_zone(game, self.player, self.card.0, &[self.card.1]);
        verify_cards_in_zone(game, self.player, self.target.0, &[self.target.1]);

        // TODO implement
        unimplemented!()
    }
}

/// used by AZKi oshi skill
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollDice {
    player: Player,
}
impl EvaluateEvent for RollDice {
    fn evaluate_event(&self, _game: &mut Game) {
        // TODO implement
        unimplemented!()
    }
}
