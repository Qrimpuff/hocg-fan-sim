use crate::{
    gameplay::{CardRef, Game, Player, Rps, Step, Zone},
    modifiers::{Modifier, ModifierKind},
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
    StartStep,
    EndStep,

    AddCardModifiers,
    RemoveCardModifiers,
    AddZoneModifiers,
    RemoveZoneModifiers,

    LookAndSelect,
    ZoneToZone,
    ZoneToAttach,
    AttachToAttach,
    AttachToZone,

    UseSupportCard,
    /// marker event after zone to zone (back stage -> collab stage)
    Collab,

    PerformArt,
    /// marker event before zone to attach (life -> temp zone -> attach)
    LoseLife,
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
        println!("EVENT = {event:?}");

        // TODO trigger before effects
        let _before = Trigger::Before(&event);

        // perform the modification to the game state
        event.evaluate_event(self);

        // TODO sanitize the event before sending it to each player
        // TODO send event to each player

        // TODO trigger after effects
        let _after = Trigger::After(&event);
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
        self.send_event(StartTurn { active_player }.into())
    }
    pub fn report_end_turn(&mut self, active_player: Player) {
        self.send_event(EndTurn { active_player }.into())
    }
    pub fn report_start_step(&mut self, active_player: Player, active_step: Step) {
        self.send_event(
            StartStep {
                active_player,
                active_step,
            }
            .into(),
        )
    }
    pub fn report_end_step(&mut self, active_player: Player, active_step: Step) {
        self.send_event(
            EndStep {
                active_player,
                active_step,
            }
            .into(),
        )
    }

    pub fn draw_from_main_deck(&mut self, player: Player, amount: usize) {
        if amount < 1 {
            return;
        }

        let deck = self.board(player).get_zone(Zone::MainDeck);
        self.send_event(
            ZoneToZone {
                player,
                from_zone: Zone::MainDeck,
                cards: deck.peek_top_cards(amount),
                to_zone: Zone::Hand,
            }
            .into(),
        );
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
        panic!("not all card are in zone")
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
    fn evaluate_event(&self, game: &mut Game) {
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

        // TODO implement
        unimplemented!()
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
    fn evaluate_event(&self, game: &mut Game) {
        // the state doesn't change on game over
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartTurn {
    active_player: Player,
}
impl EvaluateEvent for StartTurn {
    fn evaluate_event(&self, game: &mut Game) {
        game.active_player = self.active_player;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndTurn {
    active_player: Player,
}
impl EvaluateEvent for EndTurn {
    fn evaluate_event(&self, game: &mut Game) {
        assert_eq!(self.active_player, game.active_player);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartStep {
    active_player: Player,
    active_step: Step,
}
impl EvaluateEvent for StartStep {
    fn evaluate_event(&self, game: &mut Game) {
        assert_eq!(self.active_player, game.active_player);
        game.active_step = self.active_step;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndStep {
    active_player: Player,
    active_step: Step,
}
impl EvaluateEvent for EndStep {
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

        // TODO rethink this, this is inefficient for damage marker, maybe an amount instead?
        game.card_modifiers.extend(
            self.cards
                .iter()
                .copied()
                .cartesian_product(self.modifiers.iter().cloned()),
        );
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

        // TODO rethink this, this is inefficient for damage marker, maybe an amount instead?
        game.card_modifiers.retain(|(c, m)| {
            let idx = to_remove
                .iter()
                .enumerate()
                .find(|(_, r)| r.0 == *c && r.1 == m.kind)
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

        game.zone_modifiers.extend(
            self.modifiers
                .iter()
                .cloned()
                .map(|m| (self.player, self.zone, m)),
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

        game.zone_modifiers.retain(|(p, z, m)| {
            let idx = to_remove
                .iter()
                .enumerate()
                .find(|(_, r)| r.0 == *p && r.1 == *z && r.2 == m.kind)
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
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneToAttach {
    player: Player,
    from_zone: Zone,
    cards: Vec<CardRef>,
    to_card: (Zone, CardRef),
    bloom: bool,
}
impl EvaluateEvent for ZoneToAttach {
    fn evaluate_event(&self, game: &mut Game) {
        if self.cards.is_empty() {
            return;
        }

        verify_cards_in_zone(game, self.player, self.from_zone, &self.cards);
        verify_cards_in_zone(game, self.player, self.to_card.0, &[self.to_card.1]);

        // TODO implement
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachToAttach {
    player: Player,
    from_card: (Zone, CardRef),
    cards: Vec<CardRef>,
    to_card: (Zone, CardRef),
    bloom: bool,
}
impl EvaluateEvent for AttachToAttach {
    fn evaluate_event(&self, game: &mut Game) {
        if self.cards.is_empty() {
            return;
        }

        verify_cards_in_zone(game, self.player, self.from_card.0, &[self.from_card.1]);
        verify_cards_in_zone(game, self.player, self.to_card.0, &[self.to_card.1]);

        // TODO implement
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachToZone {
    player: Player,
    from_card: (Zone, CardRef),
    cards: Vec<CardRef>,
    to_zone: Zone,
}
impl EvaluateEvent for AttachToZone {
    fn evaluate_event(&self, game: &mut Game) {
        if self.cards.is_empty() {
            return;
        }

        verify_cards_in_zone(game, self.player, self.from_card.0, &[self.from_card.1]);

        // TODO implement
        unimplemented!()
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
/// marker event after zone to zone (back stage -> collab stage)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collab {
    player: Player,
    card: (Zone, CardRef),
}
impl EvaluateEvent for Collab {
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
/// marker event before zone to attach (life -> temp zone -> attach)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoseLife {
    player: Player,
    amount: usize,
}
impl EvaluateEvent for LoseLife {
    fn evaluate_event(&self, game: &mut Game) {
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
    fn evaluate_event(&self, game: &mut Game) {
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
    fn evaluate_event(&self, game: &mut Game) {
        // TODO implement
        unimplemented!()
    }
}
