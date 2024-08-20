use std::collections::HashMap;

use crate::{
    card_effects::evaluate::{EvaluateContext, EvaluateEffect, EvaluateEffectMut},
    cards::*,
    gameplay::{
        CardRef, Game, GameContinue, GameOutcome, GameOverReason, GameResult, MainStepAction,
        PerformanceStepAction, Player, Rps, Step, Zone, ZoneAddLocation, MAX_MEMBERS_ON_STAGE,
        PRIVATE_CARD,
    },
    modifiers::{DamageMarkers, LifeTime, Modifier, ModifierKind, ModifierRef},
};
use enum_dispatch::enum_dispatch;
use iter_tools::Itertools;
use rand::Rng;
use tracing::{debug, error};
use ModifierKind::*;

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
pub enum TriggeredEvent<'a> {
    // maybe individual variants, or a container?, maybe only used for triggers, not for network?
    Before(&'a Event),
    After(&'a Event),
}

impl<'a> TriggeredEvent<'a> {
    pub fn event(&self) -> &Event {
        match self {
            TriggeredEvent::Before(e) => e,
            TriggeredEvent::After(e) => e,
        }
    }
}

pub type AdjustEventResult = Result<AdjustEventOutcome, GameOutcome>;
pub enum AdjustEventOutcome {
    ContinueEvent,
    PreventEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub origin: Option<CardRef>,
    pub kind: EventKind,
}

#[enum_dispatch(EvaluateEvent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    // Basic events
    Setup,
    Shuffle,
    RpsOutcome,
    PlayerGoingFirst,
    Reveal,
    CardMapping,

    GameStart,
    GameOver,
    StartTurn,
    EndTurn,
    EnterStep,
    ExitStep,

    AddCardModifiers,
    RemoveCardModifiers,
    ClearCardModifiers,
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
    ActivateSupportCard,
    ActivateSupportAbility,
    /// used by Lui oshi skill
    ActivateOshiSkill,
    ActivateHoloMemberAbility,
    ActivateHoloMemberArtEffect,

    PerformArt,
    WaitingForPlayerIntent,

    // Card effect events
    //...
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
    ActivateEffect {
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
        min_amount: usize,
        max_amount: usize,
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
        min_amount: usize,
        max_amount: usize,
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
    SelectNumber {
        player: Player,
        select_numbers: Vec<usize>,
    },
}

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum MainStepAction {
//     BackStageMember {
//         card: (Zone, CardRef),
//     },
//     BloomMember {
//         card: (Zone, CardRef),
//         target: (Zone, CardRef),
//     },
//     UseSupportCard {
//         card: (Zone, CardRef),
//     },
//     CollabMember {
//         card: (Zone, CardRef),
//     },
//     BatonPass {
//         card: (Zone, CardRef),
//     },
//     UseSkill {
//         card: (Zone, CardRef),
//         skill_idx: usize,
//     },
//     Skip,
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum PerformanceStepAction {
//     UseArt {
//         card: (Zone, CardRef),
//         art_idx: usize,
//         target: (Zone, CardRef),
//     },
//     Skip,
// }

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
    ActivateEffect {
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
    SelectNumber {
        player: Player,
        select_number: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EventSpan {
    pub origin_stack: Vec<Option<CardRef>>,
    pub event_stack: Vec<Event>,
}

impl EventSpan {
    pub fn new() -> Self {
        Self {
            origin_stack: Vec::new(),
            event_stack: Vec::new(),
        }
    }

    pub fn open_card_span(&mut self, card: CardRef) {
        self.origin_stack.push(Some(card));
    }
    pub fn open_event_span(&mut self, event: Event) {
        self.event_stack.push(event);
    }
    pub fn open_card_event_span(&mut self, card: CardRef, event: Event) {
        self.open_card_span(card);
        self.open_event_span(event);
    }
    pub fn open_untracked_span(&mut self) {
        self.origin_stack.push(None);
    }

    pub fn close_card_span(&mut self, card: CardRef) {
        assert_eq!(self.current_card(), Some(card));
        self.origin_stack.pop();
    }
    pub fn close_event_span(&mut self, event: &Event) {
        assert_eq!(self.current_event(), Some(event));
        self.event_stack.pop();
    }
    pub fn close_event_span_unchecked(&mut self) {
        self.event_stack.pop();
    }
    pub fn close_card_event_span(&mut self, card: CardRef, event: &Event) {
        self.close_card_span(card);
        self.close_event_span(event);
    }
    pub fn close_untracked_span(&mut self) {
        assert_eq!(self.current_card(), None);
        self.origin_stack.pop();
    }

    pub fn current_card(&self) -> Option<CardRef> {
        self.origin_stack.last().copied().flatten()
    }
    pub fn current_event(&self) -> Option<&Event> {
        self.event_stack.last()
    }

    pub fn trigger_card(&self) -> Option<CardRef> {
        self.origin_stack
            .get(self.origin_stack.len() - 2)
            .copied()
            .flatten()
    }
    pub fn trigger_event(&self) -> Option<&Event> {
        self.event_stack.get(self.event_stack.len() - 2)
    }

    pub fn event_origin_for_evaluate(&self, ctx: &EvaluateContext) -> Option<CardRef> {
        if ctx.is_triggered {
            // effect was activated because of a trigger
            self.trigger_card()
        } else {
            // no trigger are active
            self.current_card()
        }
    }
}

impl Game {
    pub fn send_event(
        &mut self,
        event_origin: Option<CardRef>,
        event_type: EventKind,
    ) -> Result<Event, GameOutcome> {
        let mut event = Event {
            origin: event_origin,
            kind: event_type,
        };

        // keep track of current event
        self.state.event_span.open_event_span(event.clone());

        // trigger before effects
        let before = TriggeredEvent::Before(&event);
        self.evaluate_triggers(before)?;

        // change the event before it happens, with modifiers from triggers
        if let AdjustEventOutcome::PreventEvent = event.adjust_event(self)? {
            // done with the current event
            // unchecked should be fine, can't check because the event changed in adjust
            self.state.event_span.close_event_span_unchecked();
            return Ok(event);
        }

        debug!(
            "EVENT = [{:?}] {event:?}",
            self.state.event_span.current_card()
        );
        // perform the modification to the game state
        event.evaluate_event(self)?;

        // TODO sanitize the event before sending it to each player
        self.client(self.state.active_player)
            .0
            .send(ClientReceive::Event(event.clone()))
            .unwrap();
        self.client(self.state.active_player.opponent())
            .0
            .send(ClientReceive::Event(event.clone()))
            .unwrap();

        // trigger after effects
        let after = TriggeredEvent::After(&event);
        self.evaluate_triggers(after)?;

        // done with the current event
        // unchecked should be fine, can't check because the event could have been changed in adjust
        self.state.event_span.close_event_span_unchecked();

        Ok(event)
    }

    fn evaluate_triggers(&mut self, trigger: TriggeredEvent) -> GameResult {
        debug!(
            "TRIGGER = [{:?}] {trigger:?}",
            self.state.event_span.current_card()
        );

        let current_player = self.state.active_player;
        let current_player_cards_on_stage = self
            .board(current_player)
            .oshi()
            .into_iter()
            .chain(self.board(current_player).stage())
            .map(|c| (current_player, c));
        let opponent = self.state.active_player.opponent();
        let opponent_cards_on_stage = self
            .board(opponent)
            .oshi()
            .into_iter()
            .chain(self.board(opponent).stage())
            .map(|c| (opponent, c));

        // current player activate first
        let all_cards_on_stage = current_player_cards_on_stage
            .chain(opponent_cards_on_stage)
            .flat_map(|(p, c)| Some(c).into_iter().chain(self.board(p).attachments(c)))
            .collect_vec();

        for card in all_cards_on_stage {
            let mut oshi_skill = None;
            let mut member_ability = None;
            let mut support_ability = false;
            match self.lookup_card(card) {
                Card::OshiHoloMember(o) => {
                    for (idx, skill) in o.skills.iter().enumerate() {
                        // FIXME need to use the usual check, but with event?
                        if skill.triggers.iter().any(|t| t.should_activate(&trigger))
                            && o.can_use_skill(card, idx, self, false)
                        {
                            debug!("ACTIVATE SKILL? = {skill:?}");
                            oshi_skill = Some(idx);
                        }
                    }
                }
                Card::HoloMember(m) => {
                    for (idx, ability) in m.abilities.iter().enumerate() {
                        // FIXME need to use the usual check, but with event?
                        if ability.should_activate(card, &trigger)
                            && m.can_use_ability(card, idx, self, false)
                        {
                            debug!("ACTIVATE ABILITY = {ability:?}");
                            member_ability = Some(idx);
                        }
                    }
                }
                Card::Support(s) => {
                    // FIXME need to use the usual check, but with event?
                    if s.triggers.iter().any(|t| t.should_activate(&trigger))
                        && s.can_use_ability(card, self, false)
                    {
                        debug!("ACTIVATE SUPPORT = {s:?}");
                        support_ability = true;
                    }
                }
                Card::Cheer(_) => {} // cheers do not have triggers yet
            }

            // activate skill or ability
            self.state.event_span.open_untracked_span();
            if let Some(idx) = oshi_skill {
                // prompt for yes / no, optional activation
                let activate = self.prompt_for_optional_activate(self.player_for_card(card));
                if activate {
                    self.activate_oshi_skill(Some(card), card, idx, true)?;
                }
            }
            if let Some(idx) = member_ability {
                self.activate_holo_member_ability(Some(card), card, idx, true)?;
            }
            if support_ability {
                self.activate_support_ability(Some(card), card, true)?;
            }
            self.state.event_span.close_untracked_span();
        }

        Ok(GameContinue)
    }

    pub fn setup_game(&mut self, event_origin: Option<CardRef>) -> GameResult {
        self.send_event(event_origin, Setup {}.into())?;

        Ok(GameContinue)
    }

    pub fn report_rps_draw(&mut self, event_origin: Option<CardRef>) -> GameResult {
        self.send_event(
            event_origin,
            RpsOutcome {
                winning_player: None,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }
    pub fn report_rps_win(&mut self, event_origin: Option<CardRef>, player: Player) -> GameResult {
        self.send_event(
            event_origin,
            RpsOutcome {
                winning_player: Some(player),
            }
            .into(),
        )?;

        Ok(GameContinue)
    }
    pub fn report_player_going_first(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
    ) -> GameResult {
        self.send_event(
            event_origin,
            PlayerGoingFirst {
                first_player: player,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn report_start_game(
        &mut self,
        event_origin: Option<CardRef>,
        active_player: Player,
    ) -> GameResult {
        self.send_event(event_origin, GameStart { active_player }.into())?;

        Ok(GameContinue)
    }
    pub fn report_game_over(
        &mut self,
        event_origin: Option<CardRef>,
        game_outcome: GameOutcome,
    ) -> GameResult {
        self.send_event(
            event_origin,
            GameOver {
                game_outcome,
                turn_number: self.state.turn_number,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }
    pub fn report_game_over_draw(&mut self, event_origin: Option<CardRef>) -> GameResult {
        self.send_event(
            event_origin,
            GameOver {
                game_outcome: GameOutcome {
                    winning_player: None,
                    reason: GameOverReason::Draw,
                },
                turn_number: self.state.turn_number,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }
    pub fn report_start_turn(
        &mut self,
        event_origin: Option<CardRef>,
        active_player: Player,
    ) -> GameResult {
        self.send_event(
            event_origin,
            StartTurn {
                active_player,
                turn_number: self.state.turn_number,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }
    pub fn report_end_turn(
        &mut self,
        event_origin: Option<CardRef>,
        active_player: Player,
    ) -> GameResult {
        self.send_event(
            event_origin,
            EndTurn {
                active_player,
                turn_number: self.state.turn_number,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }
    pub fn report_enter_step(
        &mut self,
        event_origin: Option<CardRef>,
        active_player: Player,
        active_step: Step,
    ) -> GameResult {
        self.send_event(
            event_origin,
            EnterStep {
                active_player,
                active_step,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }
    pub fn report_exit_step(
        &mut self,
        event_origin: Option<CardRef>,
        active_player: Player,
        active_step: Step,
    ) -> GameResult {
        self.send_event(
            event_origin,
            ExitStep {
                active_player,
                active_step,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn send_full_hand_to_main_deck(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
    ) -> GameResult {
        let hand = self.board(player).get_zone(Zone::Hand);
        self.send_event(
            event_origin,
            ZoneToZone {
                player,
                from_zone: Zone::Hand,
                cards: hand.peek_top_cards(hand.count()),
                to_zone: Zone::MainDeck,
                to_zone_location: Zone::MainDeck.default_add_location(),
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn shuffle_main_deck(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
    ) -> GameResult {
        self.send_event(
            event_origin,
            Shuffle {
                player,
                zone: Zone::MainDeck,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn shuffle_cheer_deck(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
    ) -> GameResult {
        self.send_event(
            event_origin,
            Shuffle {
                player,
                zone: Zone::CheerDeck,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn reveal_cards(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        zone: Zone,
        cards: &[CardRef],
    ) -> GameResult {
        if cards.is_empty() {
            return Ok(GameContinue);
        }

        self.send_event(
            event_origin,
            Reveal {
                player,
                zone,
                cards: cards
                    .iter()
                    .map(|c| (*c, self.lookup_card_number(*c).clone()))
                    .collect(),
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn reveal_all_cards_in_zone(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        zone: Zone,
    ) -> GameResult {
        let cards = self.board(player).get_zone(zone).all_cards();
        self.reveal_cards(event_origin, player, zone, &cards)
    }

    pub fn send_from_hand_to_center_stage(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        card: CardRef,
    ) -> GameResult {
        self.send_event(
            event_origin,
            ZoneToZone {
                player,
                from_zone: Zone::Hand,
                cards: vec![card],
                to_zone: Zone::CenterStage,
                to_zone_location: Zone::CenterStage.default_add_location(),
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn send_from_back_stage_to_center_stage(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        card: CardRef,
    ) -> GameResult {
        self.send_event(
            event_origin,
            ZoneToZone {
                player,
                from_zone: Zone::BackStage,
                cards: vec![card],
                to_zone: Zone::CenterStage,
                to_zone_location: Zone::CenterStage.default_add_location(),
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn send_from_hand_to_back_stage(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        cards: Vec<CardRef>,
    ) -> GameResult {
        if cards.is_empty() {
            return Ok(GameContinue);
        }

        self.send_event(
            event_origin,
            ZoneToZone {
                player,
                from_zone: Zone::Hand,
                cards,
                to_zone: Zone::BackStage,
                to_zone_location: Zone::BackStage.default_add_location(),
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn send_cards_to_holo_power(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        amount: usize,
    ) -> GameResult {
        if amount < 1 {
            return Ok(GameContinue);
        }

        let deck = self.board(player).get_zone(Zone::MainDeck);
        if deck.count() > 0 {
            self.send_event(
                event_origin,
                ZoneToZone {
                    player,
                    from_zone: Zone::MainDeck,
                    cards: deck.peek_top_cards(amount),
                    to_zone: Zone::HoloPower,
                    to_zone_location: Zone::HoloPower.default_add_location(),
                }
                .into(),
            )?;
        }

        Ok(GameContinue)
    }

    pub fn send_holo_power_to_archive(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        amount: usize,
    ) -> GameResult {
        if amount < 1 {
            return Ok(GameContinue);
        }

        let power = self.board(player).get_zone(Zone::HoloPower);
        if power.count() < amount {
            panic!("not enough holo power");
        }

        self.send_event(
            event_origin,
            ZoneToZone {
                player,
                from_zone: Zone::HoloPower,
                cards: power.peek_top_cards(amount),
                to_zone: Zone::Archive,
                to_zone_location: Zone::Archive.default_add_location(),
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn send_from_back_stage_to_collab(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        card: CardRef,
    ) -> GameResult {
        self.send_event(
            event_origin,
            Collab {
                player,
                card: (Zone::BackStage, card),
                holo_power_amount: 1, // TODO some cards could maybe power for more
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn send_from_collab_to_back_stage(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        card: CardRef,
    ) -> GameResult {
        self.send_event(
            event_origin,
            ZoneToZone {
                player,
                from_zone: Zone::Collab,
                cards: vec![card],
                to_zone: Zone::BackStage,
                to_zone_location: Zone::BackStage.default_add_location(),
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn send_from_center_stage_to_back_stage(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        card: CardRef,
    ) -> GameResult {
        self.send_event(
            event_origin,
            ZoneToZone {
                player,
                from_zone: Zone::CenterStage,
                cards: vec![card],
                to_zone: Zone::BackStage,
                to_zone_location: Zone::BackStage.default_add_location(),
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn baton_pass_center_stage_to_back_stage(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        from_card: CardRef,
        to_card: CardRef,
    ) -> GameResult {
        self.send_event(
            event_origin,
            BatonPass {
                player,
                from_card: (Zone::CenterStage, from_card),
                to_card: (Zone::BackStage, to_card),
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn send_cheers_to_life(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        amount: usize,
    ) -> GameResult {
        if amount < 1 {
            return Ok(GameContinue);
        }

        let cheers = self.board(player).get_zone(Zone::CheerDeck);
        self.send_event(
            event_origin,
            ZoneToZone {
                player,
                from_zone: Zone::CheerDeck,
                cards: cheers.peek_top_cards(amount),
                to_zone: Zone::Life,
                to_zone_location: Zone::Life.default_add_location(),
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn send_cards_to_archive(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        cards: Vec<CardRef>,
    ) -> GameResult {
        if cards.is_empty() {
            return Ok(GameContinue);
        }

        for (zone, cards) in self.board(player).group_cards_by_zone(&cards) {
            self.send_event(
                event_origin,
                ZoneToZone {
                    player,
                    from_zone: zone,
                    cards,
                    to_zone: Zone::Archive,
                    to_zone_location: Zone::Archive.default_add_location(),
                }
                .into(),
            )?;
        }

        Ok(GameContinue)
    }

    pub fn send_cards_to_zone(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        cards: Vec<CardRef>,
        to_zone: Zone,
        location: ZoneAddLocation,
    ) -> GameResult {
        if cards.is_empty() {
            return Ok(GameContinue);
        }

        let mut from_zone: HashMap<_, Vec<_>> = HashMap::new();
        let mut from_cards: HashMap<_, Vec<_>> = HashMap::new();

        // group by card or zone
        for card in cards {
            if let Some(attached_to) = self.board(player).attached_to(card) {
                from_cards.entry(attached_to).or_default().push(card);
            } else {
                let zone = self
                    .board(player)
                    .find_card_zone(card)
                    .expect("card should be in a zone");
                from_zone.entry(zone).or_default().push(card);
            }
        }

        for (zone, cards) in from_zone {
            self.send_event(
                event_origin,
                ZoneToZone {
                    player,
                    from_zone: zone,
                    cards,
                    to_zone,
                    to_zone_location: location,
                }
                .into(),
            )?;
        }

        for (from_card, attachments) in from_cards {
            let from_zone = self
                .board(player)
                .find_card_zone(from_card)
                .expect("card should be in a zone");
            self.send_event(
                event_origin,
                AttachToZone {
                    player,
                    from_card: (from_zone, from_card),
                    attachments,
                    to_zone,
                    to_zone_location: location,
                }
                .into(),
            )?;
        }

        Ok(GameContinue)
    }

    pub fn attach_cheers_from_zone(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        zone: Zone,
        amount: usize,
    ) -> GameResult {
        if amount < 1 {
            return Ok(GameContinue);
        }

        // - draw cards from zone (cheer deck, life), then attach it
        let cheers = self.board(player).get_zone(zone).peek_top_cards(amount);
        self.reveal_cards(event_origin, player, zone, &cheers)?;
        for cheer in cheers {
            // TODO package with prompt
            // println!("lost a life: {}", CardDisplay::new(cheer, self));

            if let Some(mem) = self.prompt_for_cheer(player) {
                let to_zone = self
                    .board(player)
                    .find_card_zone(mem)
                    .expect("the card should be in a zone");

                self.send_event(
                    event_origin,
                    ZoneToAttach {
                        player,
                        from_zone: zone,
                        attachments: vec![cheer],
                        to_card: (to_zone, mem),
                    }
                    .into(),
                )?;
            } else {
                self.send_event(
                    event_origin,
                    ZoneToZone {
                        player,
                        from_zone: zone,
                        cards: vec![cheer],
                        to_zone: Zone::Archive,
                        to_zone_location: Zone::Archive.default_add_location(),
                    }
                    .into(),
                )?;
            }
        }

        Ok(GameContinue)
    }

    pub fn attach_cards_to_card(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        attachments: Vec<CardRef>,
        card: CardRef,
    ) -> GameResult {
        if attachments.is_empty() {
            return Ok(GameContinue);
        }

        let to_zone = self
            .board(player)
            .find_card_zone(card)
            .expect("card should be in a zone");
        let mut from_zone: HashMap<_, Vec<_>> = HashMap::new();
        let mut from_cards: HashMap<_, Vec<_>> = HashMap::new();

        // group by card or zone
        for card in attachments {
            if let Some(attached_to) = self.board(player).attached_to(card) {
                from_cards.entry(attached_to).or_default().push(card);
            } else {
                let zone = self
                    .board(player)
                    .find_card_zone(card)
                    .expect("card should be in a zone");
                from_zone.entry(zone).or_default().push(card);
            }
        }

        for (zone, attachments) in from_zone {
            self.send_event(
                event_origin,
                ZoneToAttach {
                    player,
                    from_zone: zone,
                    attachments,
                    to_card: (to_zone, card),
                }
                .into(),
            )?;
        }

        for (from_card, attachments) in from_cards {
            let from_zone = self
                .board(player)
                .find_card_zone(from_card)
                .expect("card should be in a zone");
            self.send_event(
                event_origin,
                AttachToAttach {
                    player,
                    from_card: (from_zone, from_card),
                    attachments,
                    to_card: (to_zone, card),
                }
                .into(),
            )?;
        }

        Ok(GameContinue)
    }

    pub fn send_attachments_to_archive(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        card: CardRef,
        attachments: Vec<CardRef>,
    ) -> GameResult {
        if attachments.is_empty() {
            return Ok(GameContinue);
        }

        let zone = self
            .board(player)
            .find_card_zone(card)
            .expect("the card should be in a zone");

        self.send_event(
            event_origin,
            AttachToZone {
                player,
                from_card: (zone, card),
                attachments,
                to_zone: Zone::Archive,
                to_zone_location: Zone::Archive.default_add_location(),
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn add_many_modifiers_to_many_cards(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        zone: Zone,
        cards: Vec<CardRef>,
        modifiers: Vec<Modifier>,
    ) -> GameResult {
        if cards.is_empty() || modifiers.is_empty() {
            return Ok(GameContinue);
        }

        self.send_event(
            event_origin,
            AddCardModifiers {
                player,
                zone,
                cards,
                modifiers,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn remove_many_modifiers_from_many_cards(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        zone: Zone,
        cards: Vec<CardRef>,
        modifiers: Vec<ModifierRef>,
    ) -> GameResult {
        if cards.is_empty() || modifiers.is_empty() {
            return Ok(GameContinue);
        }

        self.send_event(
            event_origin,
            RemoveCardModifiers {
                player,
                zone,
                cards,
                modifiers,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn clear_all_modifiers_from_many_cards(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        zone: Zone,
        cards: Vec<CardRef>,
    ) -> GameResult {
        let cards = cards
            .into_iter()
            .filter(|c| self.state.card_modifiers.contains_key(c))
            .collect_vec();

        if cards.is_empty() {
            return Ok(GameContinue);
        }

        self.send_event(
            event_origin,
            ClearCardModifiers {
                player,
                zone,
                cards,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn add_many_modifiers_to_zone(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        zone: Zone,
        modifiers: Vec<Modifier>,
    ) -> GameResult {
        if modifiers.is_empty() {
            return Ok(GameContinue);
        }

        self.send_event(
            event_origin,
            AddZoneModifiers {
                player,
                zone,
                modifiers,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn remove_many_modifiers_from_zone(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        zone: Zone,
        modifiers: Vec<ModifierRef>,
    ) -> GameResult {
        if modifiers.is_empty() {
            return Ok(GameContinue);
        }

        self.send_event(
            event_origin,
            RemoveZoneModifiers {
                player,
                zone,
                modifiers,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn deal_damage(
        &mut self,
        event_origin: Option<CardRef>,
        card: CardRef,
        target: CardRef,
        dmg: DamageMarkers,
    ) -> GameResult {
        let player = self.player_for_card(card);
        let card_zone = self
            .board(player)
            .find_card_zone(card)
            .expect("the card should be in a zone");

        let target_player = self.player_for_card(target);
        let target_zone = self
            .board(target_player)
            .find_card_zone(target)
            .expect("the target should be in a zone");

        self.send_event(
            event_origin,
            DealDamage {
                player,
                card: (card_zone, card),
                target_player,
                target: (target_zone, target),
                dmg,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn add_damage_markers_to_many_cards(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        zone: Zone,
        cards: Vec<CardRef>,
        dmg: DamageMarkers,
    ) -> GameResult {
        if cards.is_empty() || dmg.0 < 1 {
            return Ok(GameContinue);
        }

        self.send_event(
            event_origin,
            AddDamageMarkers {
                player,
                zone,
                cards,
                dmg,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn remove_damage_markers_from_many_cards(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        zone: Zone,
        cards: Vec<CardRef>,
        dmg: DamageMarkers,
    ) -> GameResult {
        let cards = cards
            .into_iter()
            .filter(|c| self.has_damage(*c))
            .collect_vec();

        if cards.is_empty() || dmg.0 < 1 {
            return Ok(GameContinue);
        }

        self.send_event(
            event_origin,
            RemoveDamageMarkers {
                player,
                zone,
                cards,
                dmg,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn clear_all_damage_markers_from_many_cards(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        zone: Zone,
        cards: Vec<CardRef>,
    ) -> GameResult {
        let cards = cards
            .into_iter()
            .filter(|c| self.has_damage(*c))
            .collect_vec();

        if cards.is_empty() {
            return Ok(GameContinue);
        }

        self.send_event(
            event_origin,
            ClearDamageMarkers {
                player,
                zone,
                cards,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn draw_from_main_deck(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        amount: usize,
    ) -> GameResult {
        if amount < 1 {
            return Ok(GameContinue);
        }

        self.send_event(event_origin, Draw { player, amount }.into())?;

        Ok(GameContinue)
    }

    pub fn lose_lives(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        amount: usize,
    ) -> GameResult {
        if amount < 1 {
            return Ok(GameContinue);
        }

        self.send_event(event_origin, LoseLives { player, amount }.into())?;

        Ok(GameContinue)
    }

    pub fn bloom_holo_member(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        bloom: CardRef,
        target: CardRef,
    ) -> GameResult {
        let stage = self
            .board(player)
            .find_card_zone(target)
            .expect("the card should be on stage");
        self.send_event(
            event_origin,
            Bloom {
                player,
                // can only bloom from hand, for now
                from_card: (Zone::Hand, bloom),
                to_card: (stage, target),
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn use_support_card(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        card: CardRef,
    ) -> GameResult {
        self.send_event(
            event_origin,
            ActivateSupportCard {
                player,
                // can only use card from hand, for now
                card: (Zone::Hand, card),
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn use_oshi_skill(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        card: CardRef,
        skill_idx: usize,
    ) -> GameResult {
        self.send_event(
            event_origin,
            ActivateOshiSkill {
                player,
                // can only use skill from oshi
                card: (Zone::Oshi, card),
                skill_idx,
                is_triggered: false,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn activate_oshi_skill(
        &mut self,
        event_origin: Option<CardRef>,
        card: CardRef,
        skill_idx: usize,
        is_triggered: bool,
    ) -> GameResult {
        let player = self.player_for_card(card);
        self.send_event(
            event_origin,
            ActivateOshiSkill {
                player,
                // can only use skill from oshi
                card: (Zone::Oshi, card),
                skill_idx,
                is_triggered,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn activate_holo_member_ability(
        &mut self,
        event_origin: Option<CardRef>,
        card: CardRef,
        ability_idx: usize,
        is_triggered: bool,
    ) -> GameResult {
        let player = self.player_for_card(card);
        let zone = self
            .board(player)
            .find_card_zone(card)
            .expect("member should be in a zone");
        self.send_event(
            event_origin,
            ActivateHoloMemberAbility {
                player,
                card: (zone, card),
                ability_idx,
                is_triggered,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn activate_support_ability(
        &mut self,
        event_origin: Option<CardRef>,
        card: CardRef,
        is_triggered: bool,
    ) -> GameResult {
        let player = self.player_for_card(card);
        let zone = self
            .board(player)
            .find_card_zone(card)
            .expect("member should be in a zone");
        self.send_event(
            event_origin,
            ActivateSupportAbility {
                player,
                card: (zone, card),
                is_triggered,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn perform_art(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        card: CardRef,
        art_idx: usize,
        target: Option<CardRef>,
    ) -> GameResult {
        let card_zone = self
            .board(player)
            .find_card_zone(card)
            .expect("the card should be in a zone");

        let mut target_player = None;
        let mut target_zone_card = None;
        if let Some(target) = target {
            let t_player = self.player_for_card(target);
            let t_zone = self
                .board(t_player)
                .find_card_zone(target)
                .expect("the target should be in a zone");
            target_player = Some(t_player);
            target_zone_card = Some((t_zone, target));
        }

        self.send_event(
            event_origin,
            PerformArt {
                player,
                card: (card_zone, card),
                art_idx,
                target_player,
                target: target_zone_card,
            }
            .into(),
        )?;

        Ok(GameContinue)
    }

    pub fn roll_dice(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
    ) -> Result<u8, GameOutcome> {
        let event = self.send_event(event_origin, RollDice { player, number: 0 }.into())?;

        let Event {
            kind: EventKind::RollDice(roll_dice),
            ..
        } = event
        else {
            unreachable!("the event type cannot change")
        };

        Ok(roll_dice.number)
    }
}

#[enum_dispatch]
trait EvaluateEvent {
    fn adjust_event(
        &mut self,
        _event_origin: Option<CardRef>,
        _game: &mut Game,
    ) -> AdjustEventResult {
        Ok(AdjustEventOutcome::ContinueEvent)
    }

    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult;
}

impl Event {
    fn adjust_event(&mut self, game: &mut Game) -> AdjustEventResult {
        self.kind.adjust_event(self.origin, game)
    }

    fn evaluate_event(&self, game: &mut Game) -> GameResult {
        self.kind.evaluate_event(self.origin, game)
    }
}

fn verify_cards_in_zone(game: &Game, player: Player, zone: Zone, cards: &[CardRef]) {
    // from_zone is only there for client knowledge, the game knows where the card is
    let from_zone = game.board(player).get_zone(zone);
    let all_card_in_zone = cards.iter().all(|c| from_zone.is_in_zone(*c));
    if !all_card_in_zone {
        error!("not all cards are in zone - game: {game:#?} - player: {player:#?} - zone: {zone:#?} - cards: {cards:#?}");
        panic!("not all cards are in zone")
    }
}

fn verify_cards_attached(game: &Game, player: Player, card: CardRef, attachments: &[CardRef]) {
    // from_zone is only there for client knowledge, the game knows where the card is
    let board = game.board(player);
    let all_card_attached = attachments.iter().all(|a| board.is_attached_to(*a, card));
    if !all_card_attached {
        error!("not all cards are attached - game: {game:#?} - player: {player:#?} - card: {card:#?} - attachments: {attachments:#?}");
        panic!("not all cards are attached")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setup {
    // send both decks loadout, private, asymmetric
    // TODO not sure what to do with these
    // pub you: Player,
    // pub player_1: Loadout,
    // pub player_2: Loadout,
}
impl EvaluateEvent for Setup {
    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        game.state.active_step = Step::Setup;

        // - shuffle main deck
        game.shuffle_main_deck(event_origin, Player::One)?;
        game.shuffle_main_deck(event_origin, Player::Two)?;

        // - shuffle cheer deck
        game.shuffle_cheer_deck(event_origin, Player::One)?;
        game.shuffle_cheer_deck(event_origin, Player::Two)?;

        // - oshi face down
        // TODO oshi hide
        // TODO send (event) put oshi, not needed ? starts hidden

        // - rock/paper/scissor to choose first
        // TODO request (intent)
        let first_player;
        loop {
            println!("prompt rps");
            let rps_1 = game.prompt_for_rps(Player::One);
            let rps_2 = game.prompt_for_rps(Player::Two);
            use super::gameplay::RpsOutcome;
            match rps_1.vs(rps_2) {
                RpsOutcome::Win => {
                    println!("player 1 win rps");
                    game.report_rps_win(event_origin, Player::One)?;
                    // TODO choose first or second
                    first_player = Player::One;
                    break;
                }
                RpsOutcome::Lose => {
                    println!("player 2 win rps");
                    game.report_rps_win(event_origin, Player::Two)?;
                    // TODO choose first or second
                    first_player = Player::Two;
                    break;
                }
                RpsOutcome::Draw => {
                    println!("draw rps");
                    game.report_rps_draw(event_origin)?;
                    continue;
                }
            }
        }
        // TODO choose first or second
        game.report_player_going_first(event_origin, first_player)?;
        let second_player = first_player.opponent();

        // - draw 7 cards from main deck
        //   - can mulligan once, forced for -1 card. at 0 lose the game
        // TODO request (intent)
        game.handle_mulligan(first_player)?;

        // TODO request (intent)
        game.handle_mulligan(second_player)?;

        // - place debut member center face down
        // TODO member hide
        // TODO request (intent)
        println!("prompt debut 1");
        let debut_1 = game.prompt_for_first_debut(first_player);
        game.send_from_hand_to_center_stage(event_origin, first_player, debut_1)?;

        // TODO member hide
        // TODO request (intent)
        println!("prompt debut 2");
        let debut_2 = game.prompt_for_first_debut(second_player);
        game.send_from_hand_to_center_stage(event_origin, second_player, debut_2)?;

        // - place other debut / spot members back stage
        // TODO member hide
        // TODO request (intent)
        println!("prompt other debut 1");
        let other_debut_1: Vec<_> = game.prompt_for_first_back_stage(first_player);
        game.send_from_hand_to_back_stage(event_origin, first_player, other_debut_1)?;

        // TODO member hide
        // TODO request (intent)
        println!("prompt other debut 2");
        let other_debut_2: Vec<_> = game.prompt_for_first_back_stage(second_player);
        game.send_from_hand_to_back_stage(event_origin, second_player, other_debut_2)?;

        // - reveal face down oshi and members
        // oshi and members reveal
        game.reveal_all_cards_in_zone(event_origin, first_player, Zone::Oshi)?;
        game.reveal_all_cards_in_zone(event_origin, second_player, Zone::Oshi)?;
        game.reveal_all_cards_in_zone(event_origin, first_player, Zone::CenterStage)?;
        game.reveal_all_cards_in_zone(event_origin, second_player, Zone::CenterStage)?;
        game.reveal_all_cards_in_zone(event_origin, first_player, Zone::BackStage)?;
        game.reveal_all_cards_in_zone(event_origin, second_player, Zone::BackStage)?;

        // - draw life cards face down from cheer
        let oshi_1 = game
            .board(first_player)
            .oshi()
            .and_then(|c| game.lookup_oshi(c))
            .expect("oshi should always be there");
        game.send_cheers_to_life(event_origin, first_player, oshi_1.life as usize)?;

        let oshi_2 = game
            .board(second_player)
            .oshi()
            .and_then(|c| game.lookup_oshi(c))
            .expect("oshi should always be there");
        game.send_cheers_to_life(event_origin, second_player, oshi_2.life as usize)?;

        // skip the first reset step of each player
        game.add_zone_modifier(
            event_origin,
            first_player,
            Zone::All,
            SkipStep(Step::Reset),
            LifeTime::NextTurn(first_player),
        )?;
        game.add_zone_modifier(
            event_origin,
            second_player,
            Zone::All,
            SkipStep(Step::Reset),
            LifeTime::NextTurn(second_player),
        )?;

        // cannot use limited support on the first turn of the first player
        game.add_zone_modifier(
            event_origin,
            first_player,
            Zone::All,
            PreventLimitedSupport,
            LifeTime::NextTurn(first_player),
        )?;

        // cannot bloom on each player's first turn
        game.add_zone_modifier(
            event_origin,
            first_player,
            Zone::All,
            PreventBloom,
            LifeTime::NextTurn(first_player),
        )?;
        game.add_zone_modifier(
            event_origin,
            second_player,
            Zone::All,
            PreventBloom,
            LifeTime::NextTurn(second_player),
        )?;

        // skip the first performance step of the game
        game.add_zone_modifier(
            event_origin,
            first_player,
            Zone::All,
            SkipStep(Step::Performance),
            LifeTime::NextTurn(first_player),
        )?;

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shuffle {
    pub player: Player,
    pub zone: Zone,
}
impl EvaluateEvent for Shuffle {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        // need that split, because the borrow checker needs to know we are accessing different fields
        let zone = match self.player {
            Player::One => game.state.player_1.get_zone_mut(self.zone),
            Player::Two => game.state.player_2.get_zone_mut(self.zone),
            _ => unreachable!("both players cannot be active at the same time"),
        };
        zone.shuffle(&mut game.rng);

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpsOutcome {
    pub winning_player: Option<Player>,
}
impl EvaluateEvent for RpsOutcome {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, _game: &mut Game) -> GameResult {
        // the winning player doesn't change the state of the game

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerGoingFirst {
    pub first_player: Player,
}
impl EvaluateEvent for PlayerGoingFirst {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        game.state.active_player = self.first_player;

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reveal {
    pub player: Player,
    pub zone: Zone,
    pub cards: Vec<(CardRef, CardNumber)>,
}
impl EvaluateEvent for Reveal {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.cards.is_empty() {
            return Ok(GameContinue);
        }

        verify_cards_in_zone(
            game,
            self.player,
            self.zone,
            &self.cards.iter().map(|(c, _)| c).copied().collect_vec(),
        );

        // TODO implement for network

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardMapping {
    pub card_map: HashMap<CardRef, (Player, CardNumber)>,
}
impl EvaluateEvent for CardMapping {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, _game: &mut Game) -> GameResult {
        unimplemented!("it's a temporary patch")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameStart {
    pub active_player: Player,
}
impl EvaluateEvent for GameStart {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        // the state changes on start turn
        game.state.active_player = self.active_player;

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameOver {
    pub game_outcome: GameOutcome,
    pub turn_number: u8,
}
impl EvaluateEvent for GameOver {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        game.state.active_step = Step::GameOver;
        game.state.game_outcome = Some(self.game_outcome);

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartTurn {
    pub active_player: Player,
    pub turn_number: u8,
}
impl EvaluateEvent for StartTurn {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        game.state.active_player = self.active_player;

        game.start_turn_modifiers(self.active_player);

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndTurn {
    pub active_player: Player,
    pub turn_number: u8,
}
impl EvaluateEvent for EndTurn {
    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        assert_eq!(self.active_player, game.state.active_player);

        game.end_turn_modifiers(self.active_player);

        game.state.event_span.open_untracked_span();
        game.remove_expiring_modifiers(event_origin, LifeTime::ThisTurn)?;
        game.state.event_span.close_untracked_span();

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnterStep {
    pub active_player: Player,
    pub active_step: Step,
}
impl EvaluateEvent for EnterStep {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        assert_eq!(self.active_player, game.state.active_player);
        game.state.active_step = self.active_step;

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitStep {
    pub active_player: Player,
    pub active_step: Step,
}
impl EvaluateEvent for ExitStep {
    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        assert_eq!(self.active_player, game.state.active_player);
        assert_eq!(self.active_step, game.state.active_step);

        game.state.event_span.open_untracked_span();
        game.remove_expiring_modifiers(event_origin, LifeTime::ThisStep)?;
        game.state.event_span.close_untracked_span();

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddCardModifiers {
    pub player: Player,
    pub zone: Zone,
    pub cards: Vec<CardRef>,
    pub modifiers: Vec<Modifier>,
}
impl EvaluateEvent for AddCardModifiers {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.cards.is_empty() || self.modifiers.is_empty() {
            return Ok(GameContinue);
        }

        verify_cards_in_zone(game, self.player, self.zone, &self.cards);

        for card in &self.cards {
            game.state
                .card_modifiers
                .entry(*card)
                .or_default()
                .extend(self.modifiers.iter().cloned());
        }

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveCardModifiers {
    pub player: Player,
    pub zone: Zone,
    pub cards: Vec<CardRef>,
    pub modifiers: Vec<ModifierRef>,
}
impl EvaluateEvent for RemoveCardModifiers {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.cards.is_empty() || self.modifiers.is_empty() {
            return Ok(GameContinue);
        }

        verify_cards_in_zone(game, self.player, self.zone, &self.cards);

        let mut to_remove = self
            .cards
            .iter()
            .copied()
            .cartesian_product(self.modifiers.iter().cloned())
            .collect_vec();

        for card in &self.cards {
            game.state
                .card_modifiers
                .entry(*card)
                .or_default()
                .retain(|m| {
                    let idx = to_remove
                        .iter()
                        .enumerate()
                        .find(|(_, r)| r.0 == *card && r.1 == m.id)
                        .map(|(i, _)| i);
                    if let Some(idx) = idx {
                        to_remove.swap_remove(idx);
                        false
                    } else {
                        true
                    }
                });
        }

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearCardModifiers {
    pub player: Player,
    pub zone: Zone,
    pub cards: Vec<CardRef>,
}
impl EvaluateEvent for ClearCardModifiers {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.cards.is_empty() {
            return Ok(GameContinue);
        }

        verify_cards_in_zone(game, self.player, self.zone, &self.cards);

        for card in &self.cards {
            game.state.card_modifiers.remove_entry(card);
        }

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddZoneModifiers {
    pub player: Player,
    pub zone: Zone,
    pub modifiers: Vec<Modifier>,
}
impl EvaluateEvent for AddZoneModifiers {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.modifiers.is_empty() {
            return Ok(GameContinue);
        }

        game.state
            .zone_modifiers
            .entry(self.player)
            .or_default()
            .extend(
                self.modifiers
                    .iter()
                    .cloned()
                    .map(|m: Modifier| (self.zone, m)),
            );

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveZoneModifiers {
    pub player: Player,
    pub zone: Zone,
    pub modifiers: Vec<ModifierRef>,
}
impl EvaluateEvent for RemoveZoneModifiers {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.modifiers.is_empty() {
            return Ok(GameContinue);
        }

        let mut to_remove = self
            .modifiers
            .iter()
            .cloned()
            .map(|m| (self.player, self.zone, m))
            .collect_vec();

        game.state
            .zone_modifiers
            .entry(self.player)
            .or_default()
            .retain(|(z, m)| {
                let idx = to_remove
                    .iter()
                    .enumerate()
                    .find(|(_, r)| r.0 == self.player && r.1 == *z && r.2 == m.id)
                    .map(|(i, _)| i);
                if let Some(idx) = idx {
                    to_remove.swap_remove(idx);
                    false
                } else {
                    true
                }
            });

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddDamageMarkers {
    pub player: Player,
    pub zone: Zone,
    pub cards: Vec<CardRef>,
    pub dmg: DamageMarkers,
}
impl EvaluateEvent for AddDamageMarkers {
    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.cards.is_empty() || self.dmg.0 < 1 {
            return Ok(GameContinue);
        }

        verify_cards_in_zone(game, self.player, self.zone, &self.cards);

        for card in &self.cards {
            *game.state.card_damage_markers.entry(*card).or_default() += self.dmg;
        }

        // verify that they are still alive
        let defeated = self
            .cards
            .iter()
            .copied()
            .filter(|card| game.remaining_hp(*card) == 0)
            .collect_vec();

        // calculate life loss
        let life_loss = defeated
            .iter()
            .filter_map(|c| game.lookup_holo_member(*c))
            .map(|m| {
                // buzz members loses 2 lives
                if m.attributes.contains(&HoloMemberExtraAttribute::Buzz) {
                    2
                } else {
                    1
                }
            })
            .sum();

        // send member to archive, from attack
        for card in defeated {
            game.send_event(
                event_origin,
                HoloMemberDefeated {
                    player: self.player,
                    card: (self.zone, card),
                }
                .into(),
            )?;
        }

        // TODO do we need a untracked span here?
        game.lose_lives(event_origin, self.player, life_loss)?;

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveDamageMarkers {
    pub player: Player,
    pub zone: Zone,
    pub cards: Vec<CardRef>,
    pub dmg: DamageMarkers,
}
impl EvaluateEvent for RemoveDamageMarkers {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.cards.is_empty() || self.dmg.0 < 1 {
            return Ok(GameContinue);
        }

        verify_cards_in_zone(game, self.player, self.zone, &self.cards);

        for card in &self.cards {
            *game.state.card_damage_markers.entry(*card).or_default() -= self.dmg;
        }

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearDamageMarkers {
    pub player: Player,
    pub zone: Zone,
    pub cards: Vec<CardRef>,
}
impl EvaluateEvent for ClearDamageMarkers {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.cards.is_empty() {
            return Ok(GameContinue);
        }

        verify_cards_in_zone(game, self.player, self.zone, &self.cards);

        for card in &self.cards {
            game.state.card_damage_markers.remove_entry(card);
        }

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookAndSelect {
    pub player: Player,
    pub zone: Zone,
    pub cards: Vec<CardRef>, // could just be a count, it's just for the opponent
}
impl EvaluateEvent for LookAndSelect {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.cards.is_empty() {
            return Ok(GameContinue);
        }

        verify_cards_in_zone(game, self.player, self.zone, &self.cards);

        // TODO implement
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneToZone {
    pub player: Player,
    pub from_zone: Zone,
    pub cards: Vec<CardRef>,
    pub to_zone: Zone,
    pub to_zone_location: ZoneAddLocation,
}
impl EvaluateEvent for ZoneToZone {
    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.cards.is_empty() {
            return Ok(GameContinue);
        }

        verify_cards_in_zone(game, self.player, self.from_zone, &self.cards);

        // cannot send to stage if stage is full
        if !Zone::Stage.includes(self.from_zone) && Zone::Stage.includes(self.to_zone) {
            let count = game
                .board(self.player)
                .stage()
                .filter_map(|c| game.lookup_holo_member(c))
                .count();
            if count >= MAX_MEMBERS_ON_STAGE {
                panic!("cannot send to stage. stage is full");
            }
        }

        game.board_mut(self.player).send_many_to_zone(
            self.cards.clone(),
            self.to_zone,
            self.to_zone_location,
        );

        game.state.event_span.open_untracked_span();
        // lose attachments and buffs when leaving stage
        if !Zone::Stage.includes(self.to_zone) {
            game.clear_all_damage_markers_from_many_cards(
                event_origin,
                self.player,
                self.to_zone,
                self.cards.clone(),
            )?;

            for card in &self.cards {
                let attachments = game.board(self.player).attachments(*card);
                game.send_attachments_to_archive(event_origin, self.player, *card, attachments)?;

                game.clear_all_modifiers(event_origin, *card)?;
            }
        }

        // check if a player lost when cards are moving
        game.check_loss_conditions()?;
        game.state.event_span.close_untracked_span();

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneToAttach {
    pub player: Player,
    pub from_zone: Zone,
    pub attachments: Vec<CardRef>,
    pub to_card: (Zone, CardRef),
}
impl EvaluateEvent for ZoneToAttach {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.attachments.is_empty() {
            return Ok(GameContinue);
        }

        verify_cards_in_zone(game, self.player, self.from_zone, &self.attachments);
        verify_cards_in_zone(game, self.player, self.to_card.0, &[self.to_card.1]);

        for attachment in &self.attachments {
            game.board_mut(self.player)
                .attach_to_card(*attachment, self.to_card.1);
        }

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachToAttach {
    pub player: Player,
    pub from_card: (Zone, CardRef),
    pub attachments: Vec<CardRef>,
    pub to_card: (Zone, CardRef),
}
impl EvaluateEvent for AttachToAttach {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.attachments.is_empty() {
            return Ok(GameContinue);
        }

        verify_cards_in_zone(game, self.player, self.from_card.0, &[self.from_card.1]);
        verify_cards_attached(game, self.player, self.from_card.1, &self.attachments);
        verify_cards_in_zone(game, self.player, self.to_card.0, &[self.to_card.1]);

        // zone to attach
        for attachment in &self.attachments {
            game.board_mut(self.player)
                .attach_to_card(*attachment, self.to_card.1);
        }

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachToZone {
    pub player: Player,
    pub from_card: (Zone, CardRef),
    pub attachments: Vec<CardRef>,
    pub to_zone: Zone,
    pub to_zone_location: ZoneAddLocation,
}
impl EvaluateEvent for AttachToZone {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.attachments.is_empty() {
            return Ok(GameContinue);
        }

        verify_cards_in_zone(game, self.player, self.from_card.0, &[self.from_card.1]);
        verify_cards_attached(game, self.player, self.from_card.1, &self.attachments);

        game.board_mut(self.player).send_many_to_zone(
            self.attachments.clone(),
            self.to_zone,
            self.to_zone_location,
        );

        Ok(GameContinue)
    }
}

/// marker event after zone to zone (deck -> hand)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draw {
    pub player: Player,
    pub amount: usize,
}
impl EvaluateEvent for Draw {
    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.amount < 1 {
            return Ok(GameContinue);
        }

        let deck = game.board(self.player).get_zone(Zone::MainDeck);
        game.send_event(
            event_origin,
            ZoneToZone {
                player: self.player,
                from_zone: Zone::MainDeck,
                cards: deck.peek_top_cards(self.amount),
                to_zone: Zone::Hand,
                to_zone_location: Zone::Hand.default_add_location(),
            }
            .into(),
        )?;

        Ok(GameContinue)
    }
}

/// marker event after zone to zone (back stage -> collab stage)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collab {
    pub player: Player,
    pub card: (Zone, CardRef),
    pub holo_power_amount: usize,
}
impl EvaluateEvent for Collab {
    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        verify_cards_in_zone(game, self.player, self.card.0, &[self.card.1]);

        // check condition for collab
        if game.board(self.player).get_zone(Zone::Collab).count() > 0 {
            panic!("collab is already occupied");
        }
        if game.has_modifier(self.card.1, Resting) {
            panic!("cannot collab a resting member");
        }
        if game.has_modifier(self.card.1, PreventCollab) {
            panic!("cannot collab this member");
        }

        game.send_event(
            event_origin,
            ZoneToZone {
                player: self.player,
                from_zone: self.card.0,
                cards: vec![self.card.1],
                to_zone: Zone::Collab,
                to_zone_location: Zone::Collab.default_add_location(),
            }
            .into(),
        )?;

        game.state.event_span.open_untracked_span();
        //   - draw down card from deck into power zone
        game.send_cards_to_holo_power(event_origin, self.player, self.holo_power_amount)?;

        // can only collab once per turn
        game.add_zone_modifier(
            event_origin,
            self.player,
            Zone::All,
            PreventCollab,
            LifeTime::ThisTurn,
        )?;
        game.state.event_span.close_untracked_span();

        Ok(GameContinue)
    }
}

/// marker event before zone to attach (life -> temp zone -> attach)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoseLives {
    pub player: Player,
    pub amount: usize,
}
impl EvaluateEvent for LoseLives {
    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        if self.amount < 1 {
            return Ok(GameContinue);
        }

        // if the remaining lives are too few, send them to archive
        if game.board(self.player).get_zone(Zone::Life).count() <= self.amount {
            let cheers = game.board(self.player).get_zone(Zone::Life).all_cards();
            game.reveal_cards(event_origin, self.player, Zone::Life, &cheers)?;
            game.send_event(
                event_origin,
                ZoneToZone {
                    player: self.player,
                    from_zone: Zone::Life,
                    cards: cheers,
                    to_zone: Zone::Archive,
                    to_zone_location: Zone::Archive.default_add_location(),
                }
                .into(),
            )?;
        } else {
            game.attach_cheers_from_zone(event_origin, self.player, Zone::Life, self.amount)?;
        }

        Ok(GameContinue)
    }
}

/// marker event before zone to zone (deck -> hand)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bloom {
    pub player: Player,
    pub from_card: (Zone, CardRef),
    pub to_card: (Zone, CardRef),
}
impl EvaluateEvent for Bloom {
    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        verify_cards_in_zone(game, self.player, self.from_card.0, &[self.from_card.1]);
        verify_cards_in_zone(game, self.player, self.to_card.0, &[self.to_card.1]);

        let bloom = game
            .lookup_holo_member(self.from_card.1)
            .expect("should be a valid member");
        let target = game
            .lookup_holo_member(self.to_card.1)
            .expect("should be a valid member");
        if !bloom.can_bloom_target(self.from_card.1, game, (self.to_card.1, target)) {
            unreachable!("bloom should not be an option, if it's not allowed")
        }

        game.state.event_span.open_untracked_span();

        // attach the bloom card to the bloom target
        game.board_mut(self.player)
            .attach_to_card(self.from_card.1, self.to_card.1);

        // move the attachments and damage to the new card
        game.board_mut(self.player)
            .promote_attachment(self.from_card.1, self.to_card.1);
        game.promote_modifiers(self.from_card.1, self.to_card.1);
        game.promote_damage_markers(self.from_card.1, self.to_card.1);

        // prevent it from blooming again this turn
        game.add_modifier(
            event_origin,
            self.from_card.1,
            PreventBloom,
            LifeTime::ThisTurn,
        )?;

        game.state.event_span.close_untracked_span();

        Ok(GameContinue)
    }
}

/// marker event after zone to zone (back stage -> collab stage)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatonPass {
    pub player: Player,
    pub from_card: (Zone, CardRef),
    pub to_card: (Zone, CardRef),
}
impl EvaluateEvent for BatonPass {
    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        verify_cards_in_zone(game, self.player, self.from_card.0, &[self.from_card.1]);
        verify_cards_in_zone(game, self.player, self.to_card.0, &[self.to_card.1]);

        // only center stage can baton pass to back stage
        assert_eq!(self.from_card.0, Zone::CenterStage);
        assert_eq!(self.to_card.0, Zone::BackStage);

        let mem = game
            .lookup_holo_member(self.from_card.1)
            .expect("cannot pay baton pass cost for non member");

        if !mem.can_baton_pass(self.from_card.1, game) {
            unreachable!("baton should not be an option, if it's not allowed")
        }

        // pay the baton pass cost
        // TODO cost should automatic when there is a single cheers color
        // TODO request (intent) select attached cheers
        let cheers = game.prompt_for_baton_pass(self.player, self.from_card.1, mem.baton_pass_cost);
        game.send_attachments_to_archive(event_origin, self.player, self.from_card.1, cheers)?;

        // send the center member to the back
        game.send_from_center_stage_to_back_stage(event_origin, self.player, self.from_card.1)?;

        // send back stage member to center
        game.send_from_back_stage_to_center_stage(event_origin, self.player, self.to_card.1)?;

        // can only baton pass once per turn
        game.state.event_span.open_untracked_span();
        game.add_zone_modifier(
            event_origin,
            self.player,
            Zone::All,
            PreventBatonPass,
            LifeTime::ThisTurn,
        )?;
        game.state.event_span.close_untracked_span();

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivateSupportCard {
    pub player: Player,
    pub card: (Zone, CardRef),
}
impl EvaluateEvent for ActivateSupportCard {
    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        verify_cards_in_zone(game, self.player, self.card.0, &[self.card.1]);

        // - use support card
        //   - only one limited per turn
        //   - otherwise unlimited
        let sup = game
            .lookup_support(self.card.1)
            .expect("only support should be allowed here");

        let limited_use = sup.limited;
        let effect = sup.effect.clone();

        if !sup.can_use_support(self.card.1, game) {
            unreachable!("support should not be an option, if it's not allowed")
        }

        // send the support card out of the game, so it doesn't affect itself
        game.state.event_span.open_untracked_span();
        game.send_event(
            event_origin,
            ZoneToZone {
                player: self.player,
                from_zone: self.card.0,
                cards: vec![self.card.1],
                to_zone: Zone::ActivateSupport,
                to_zone_location: Zone::ActivateSupport.default_add_location(),
            }
            .into(),
        )?;
        game.state.event_span.close_untracked_span();

        // activate the support card
        game.state.event_span.open_untracked_span();
        effect.evaluate_with_card_mut(game, self.card.1, false)?;

        // limited support can only be used once per turn
        if limited_use {
            game.add_zone_modifier(
                event_origin,
                self.player,
                Zone::All,
                PreventLimitedSupport,
                LifeTime::ThisTurn,
            )?;
        }

        // send the used card to the archive
        game.send_cards_to_archive(event_origin, game.state.active_player, vec![self.card.1])?;
        game.state.event_span.close_untracked_span();

        Ok(GameContinue)
    }
}

/// used by Lui oshi skill
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivateSupportAbility {
    pub player: Player,
    pub card: (Zone, CardRef),
    pub is_triggered: bool,
}
impl EvaluateEvent for ActivateSupportAbility {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        verify_cards_in_zone(game, self.player, self.card.0, &[self.card.1]);

        let support = game
            .lookup_support(self.card.1)
            .expect("only support should be using skills");

        //  check condition for skill
        if !support.can_use_ability(self.card.1, game, self.is_triggered) {
            panic!("cannot use this skill");
        }

        let effect = support.effect.clone();

        effect.evaluate_with_card_mut(game, self.card.1, self.is_triggered)?;

        Ok(GameContinue)
    }
}

/// used by Lui oshi skill
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivateOshiSkill {
    pub player: Player,
    pub card: (Zone, CardRef),
    pub skill_idx: usize,
    pub is_triggered: bool,
}
impl EvaluateEvent for ActivateOshiSkill {
    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        verify_cards_in_zone(game, self.player, self.card.0, &[self.card.1]);

        let oshi = game
            .lookup_oshi(self.card.1)
            .expect("only oshi should be using skills");

        //  check condition for skill
        if !oshi.can_use_skill(self.card.1, self.skill_idx, game, self.is_triggered) {
            panic!("cannot use this skill");
        }

        // - use oshi skill
        //   - oshi power uses card in power zone
        //   - once per turn / once per game
        let skill = &oshi.skills[self.skill_idx];
        let cost = skill.cost as usize;
        let effect = skill.effect.clone();
        let prevent_life_time = match skill.kind {
            OshiSkillKind::Normal => LifeTime::ThisTurn,
            OshiSkillKind::Special => LifeTime::ThisGame,
        };

        // pay the cost of the oshi skill
        // TODO could have a buff that could pay for the skill
        game.state.event_span.open_untracked_span();
        game.send_holo_power_to_archive(event_origin, self.player, cost)?;
        game.state.event_span.close_untracked_span();

        effect.evaluate_with_card_mut(game, self.card.1, self.is_triggered)?;

        //   - once per turn / once per game
        game.state.event_span.open_untracked_span();
        game.add_modifier(
            event_origin,
            self.card.1,
            PreventOshiSkill(self.skill_idx),
            prevent_life_time,
        )?;
        game.state.event_span.close_untracked_span();

        Ok(GameContinue)
    }
}

/// used by Lui oshi skill
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivateHoloMemberAbility {
    pub player: Player,
    pub card: (Zone, CardRef),
    pub ability_idx: usize,
    pub is_triggered: bool,
}
impl EvaluateEvent for ActivateHoloMemberAbility {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        verify_cards_in_zone(game, self.player, self.card.0, &[self.card.1]);

        let mem = game
            .lookup_holo_member(self.card.1)
            .expect("only member should be using skills");

        //  check condition for skill
        if !mem.can_use_ability(self.card.1, self.ability_idx, game, self.is_triggered) {
            panic!("cannot use this skill");
        }

        let ability = &mem.abilities[self.ability_idx];
        let effect = ability.effect.clone();

        effect.evaluate_with_card_mut(game, self.card.1, self.is_triggered)?;

        Ok(GameContinue)
    }
}

/// used by Lui oshi skill
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivateHoloMemberArtEffect {
    pub player: Player,
    pub card: (Zone, CardRef),
    pub skill_idx: usize,
}
impl EvaluateEvent for ActivateHoloMemberArtEffect {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        verify_cards_in_zone(game, self.player, self.card.0, &[self.card.1]);

        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerformArt {
    pub player: Player,
    pub card: (Zone, CardRef),
    pub art_idx: usize,
    pub target_player: Option<Player>,
    pub target: Option<(Zone, CardRef)>,
}
impl EvaluateEvent for PerformArt {
    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        verify_cards_in_zone(game, self.player, self.card.0, &[self.card.1]);

        if let (Some(target), Some(target_player)) = (self.target, self.target_player) {
            verify_cards_in_zone(game, target_player, target.0, &[target.1]);
        }

        let mem = game
            .lookup_holo_member(self.card.1)
            .expect("this should be a valid member");

        //  check condition for art
        if !mem.can_use_art(self.card.1, self.art_idx, game) {
            panic!("cannot use this art");
        }

        // - can use 2 attacks (center, collab)
        // - can choose target (center, collab)
        // - need required attached cheers to attack
        // - apply damage and effects
        // - remove member if defeated
        //   - lose 1 life
        //   - attach lost life (cheer)
        let art = &mem.arts[self.art_idx];
        let dmg = art.damage;
        let effect = art.effect.clone();

        // evaluate the effect of art, could change damage calculation
        effect.evaluate_with_card_mut(game, self.card.1, false)?;

        // FIXME evaluate damage number
        let mut dmg = match dmg {
            HoloMemberArtDamage::Basic(dmg) => DamageMarkers::from_hp(dmg),
            HoloMemberArtDamage::Plus(dmg) => DamageMarkers::from_hp(dmg), // TODO
            HoloMemberArtDamage::Minus(dmg) => DamageMarkers::from_hp(dmg), // TODO
            HoloMemberArtDamage::Uncertain => unimplemented!(),
        };
        // apply damage modifiers
        for m in game.find_modifiers(self.card.1) {
            if let ModifierKind::MoreDamage(more_dmg_hp) = m.kind {
                dmg += DamageMarkers::from_hp(more_dmg_hp as u16);
            }
        }

        // deal damage if there is a target. if any other damage is done, it will be in the effect
        if let Some(target) = self.target {
            game.deal_damage(event_origin, self.card.1, target.1, dmg)?;
        }

        game.state.event_span.open_untracked_span();
        game.remove_expiring_modifiers(event_origin, LifeTime::ThisArt)?;

        // can only perform art once per turn
        game.add_modifier(
            event_origin,
            self.card.1,
            PreventAllArts,
            LifeTime::ThisTurn,
        )?;
        game.state.event_span.close_untracked_span();

        Ok(GameContinue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitingForPlayerIntent {
    pub player: Player,
    // reason?
}
impl EvaluateEvent for WaitingForPlayerIntent {
    fn evaluate_event(&self, _event_origin: Option<CardRef>, _game: &mut Game) -> GameResult {
        // TODO implement
        unimplemented!()
    }
}
// Card effect events
//...
/// used by Pekora oshi skill, marker event before zone to zone
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoloMemberDefeated {
    pub player: Player,
    pub card: (Zone, CardRef),
}
impl EvaluateEvent for HoloMemberDefeated {
    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        verify_cards_in_zone(game, self.player, self.card.0, &[self.card.1]);

        game.send_cards_to_archive(event_origin, self.player, vec![self.card.1])?;

        Ok(GameContinue)
    }
}
/// used by Suisei oshi skill
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DealDamage {
    pub player: Player,
    pub card: (Zone, CardRef),
    pub target_player: Player,
    pub target: (Zone, CardRef),
    pub dmg: DamageMarkers,
}
impl EvaluateEvent for DealDamage {
    fn evaluate_event(&self, event_origin: Option<CardRef>, game: &mut Game) -> GameResult {
        verify_cards_in_zone(game, self.player, self.card.0, &[self.card.1]);
        verify_cards_in_zone(game, self.target_player, self.target.0, &[self.target.1]);

        game.add_damage_markers(event_origin, self.target.1, self.dmg)?;

        Ok(GameContinue)
    }
}

/// used by AZKi oshi skill
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollDice {
    pub player: Player,
    pub number: u8,
}
impl EvaluateEvent for RollDice {
    fn adjust_event(
        &mut self,
        event_origin: Option<CardRef>,
        game: &mut Game,
    ) -> AdjustEventResult {
        // look for modifiers, and consume them, if not permanent
        let mut to_remove = None;
        for m in game.find_player_modifiers(self.player) {
            if let ModifierKind::NextDiceRoll(number) = m.kind {
                self.number = number as u8;
                if m.life_time == LifeTime::UntilRemoved {
                    to_remove = Some(m.id);
                }
                break;
            }
        }
        if let Some(id) = to_remove {
            game.state.event_span.open_untracked_span();
            game.remove_many_modifiers_from_zone(event_origin, self.player, Zone::All, vec![id])?;
            game.state.event_span.close_untracked_span();
            return Ok(AdjustEventOutcome::PreventEvent);
        }

        // not modifiers
        self.number = game.rng.gen_range(1..=6);

        Ok(AdjustEventOutcome::ContinueEvent)
    }

    fn evaluate_event(&self, _event_origin: Option<CardRef>, _game: &mut Game) -> GameResult {
        // nothing to do here
        Ok(GameContinue)
    }
}
