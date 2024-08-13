use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::num::NonZeroU16;
use std::sync::atomic::AtomicU16;

use crate::card_effects::evaluate::EvaluateEffect;
use crate::card_effects::Condition;

use super::cards::*;
use super::gameplay::*;

static NEXT_MODIFIER_REF: AtomicU16 = AtomicU16::new(1);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModifierRef(NonZeroU16);

impl Debug for ModifierRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "m_{:04x}", self.0)
    }
}
impl Display for ModifierRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierKind {
    Conditional(Condition, Box<ModifierKind>),
    // attributes
    // buff
    // debuff
    Resting,
    PreventOshiSkill(usize),
    PreventArt(usize),
    PreventAllArts,
    PreventAbility(usize),
    PreventAbilities,
    PreventCollab,
    PreventBloom,
    PreventLimitedSupport,
    PreventBatonPass,
    SkipStep(Step),
    MoreDamage(usize),
    NextDiceRoll(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifeTime {
    ThisGame,
    ThisTurn,
    /// becomes ThisTurn on player's next start turn
    NextTurn(Player),
    ThisStep,
    ThisArt,
    ThisEffect,
    /// stays until manually removed
    UntilRemoved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modifier {
    pub id: ModifierRef,
    pub kind: ModifierKind,
    life_time: LifeTime,
}

impl Modifier {
    pub fn for_card(_card: CardRef, kind: ModifierKind, life_time: LifeTime) -> Self {
        let next_ref = NEXT_MODIFIER_REF.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let id = ModifierRef(NonZeroU16::new(next_ref).expect("card is non zero"));
        Modifier {
            id,
            kind,
            life_time,
        }
    }
    pub fn for_zone(_player: Player, _zone: Zone, kind: ModifierKind, life_time: LifeTime) -> Self {
        let next_ref = NEXT_MODIFIER_REF.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let id = ModifierRef(NonZeroU16::new(next_ref).expect("zone is non zero"));
        Modifier {
            id,
            kind,
            life_time,
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.life_time, LifeTime::NextTurn(_))
    }

    pub fn start_turn(&mut self, active_player: Player) {
        // becomes ThisTurn on player's next start turn
        match self.life_time {
            LifeTime::NextTurn(p) if p == active_player => self.life_time = LifeTime::ThisTurn,
            _ => {}
        };
    }

    pub fn end_turn(&mut self, _active_player: Player) {}
}

impl GameState {
    pub fn find_modifiers(&self, card: CardRef) -> impl Iterator<Item = &Modifier> + '_ {
        let (player, _) = self.card_map.get(&card).expect("should be in the map");

        let zone = match player {
            Player::One => self
                .player_1
                .find_card_zone(card)
                .expect("the card should be on player 1 side"),
            Player::Two => self
                .player_2
                .find_card_zone(card)
                .expect("the card should be on player 2 side"),
            Player::Both => unreachable!("a card can't be owned by both player"),
        };

        self.card_modifiers.get(&card).into_iter().flatten().chain(
            self.zone_modifiers
                .get(player)
                .into_iter()
                .flatten()
                .filter(move |(z, _)| z.includes(zone))
                .map(|(_, b)| b),
        )
    }

    /// is used with Zone::All
    pub fn player_has_modifier(&self, player: Player, kind: ModifierKind) -> bool {
        // need to look for any card, oshi is always there
        let oshi = self
            .board(player)
            .get_zone(Zone::Oshi)
            .peek_top_card()
            .expect("oshi is always there");
        self.has_modifier(oshi, kind)
    }
    pub fn has_modifier(&self, card: CardRef, kind: ModifierKind) -> bool {
        self.has_modifier_with(card, |m| *m == kind)
    }
    pub fn has_modifier_with(
        &self,
        card: CardRef,
        filter_fn: impl FnMut(&ModifierKind) -> bool,
    ) -> bool {
        self.find_modifiers(card)
            .filter(|m| m.is_active())
            .filter_map(|m| match &m.kind {
                ModifierKind::Conditional(c, k) => {
                    c.evaluate_with_card(self, card).then_some(k.as_ref())
                }
                _ => Some(&m.kind),
            })
            .any(filter_fn)
    }

    // damage markers
    pub fn has_damage(&self, card: CardRef) -> bool {
        self.card_damage_markers
            .get(&card)
            .filter(|dmg| dmg.0 > 0)
            .is_some()
    }

    pub fn remaining_hp(&self, card: CardRef) -> HoloMemberHp {
        let dmg = self.get_damage(card);
        let hp = self
            .lookup_holo_member(card)
            .expect("should be a member")
            .hp;
        hp.saturating_sub(dmg.to_hp())
    }

    pub fn get_damage(&self, card: CardRef) -> DamageMarkers {
        self.card_damage_markers
            .get(&card)
            .copied()
            .unwrap_or_default()
    }
}

impl Game {
    pub fn find_modifiers(&self, card: CardRef) -> impl Iterator<Item = &Modifier> + '_ {
        self.state.find_modifiers(card)
    }

    /// is used with Zone::All
    pub fn player_has_modifier(&self, player: Player, kind: ModifierKind) -> bool {
        self.state.player_has_modifier(player, kind)
    }
    pub fn has_modifier(&self, card: CardRef, kind: ModifierKind) -> bool {
        self.state.has_modifier(card, kind)
    }
    pub fn has_modifier_with(
        &self,
        card: CardRef,
        filter_fn: impl FnMut(&ModifierKind) -> bool,
    ) -> bool {
        self.state.has_modifier_with(card, filter_fn)
    }

    pub fn add_modifier(
        &mut self,
        event_origin: Option<CardRef>,
        card: CardRef,
        kind: ModifierKind,
        life_time: LifeTime,
    ) -> GameResult {
        self.add_many_modifiers(event_origin, card, vec![(kind, life_time)])
    }
    pub fn add_many_modifiers(
        &mut self,
        event_origin: Option<CardRef>,
        card: CardRef,
        modifiers: Vec<(ModifierKind, LifeTime)>,
    ) -> GameResult {
        let player = self.player_for_card(card);
        let zone = self
            .board(player)
            .find_card_zone(card)
            .expect("the card should be in a zone");
        self.add_many_modifiers_to_many_cards(
            event_origin,
            player,
            zone,
            vec![card],
            modifiers
                .into_iter()
                .map(move |(kind, life_time)| Modifier::for_card(card, kind, life_time))
                .collect(),
        )
    }

    pub fn add_zone_modifier(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        zone: Zone,
        kind: ModifierKind,
        life_time: LifeTime,
    ) -> GameResult {
        self.add_many_zone_modifiers(event_origin, player, zone, kind, life_time, 1)
    }
    pub fn add_many_zone_modifiers(
        &mut self,
        event_origin: Option<CardRef>,
        player: Player,
        zone: Zone,
        kind: ModifierKind,
        life_time: LifeTime,
        amount: usize,
    ) -> GameResult {
        self.add_many_modifiers_to_zone(
            event_origin,
            player,
            zone,
            (0..amount)
                .map(|_| Modifier::for_zone(player, zone, kind.clone(), life_time))
                .collect(),
        )
    }

    pub fn remove_all_modifiers(
        &mut self,
        event_origin: Option<CardRef>,
        card: CardRef,
        kind: ModifierKind,
    ) -> GameResult {
        self.remove_all_modifiers_with(event_origin, card, |m| m.kind == kind)
    }
    pub fn remove_all_modifiers_with(
        &mut self,
        event_origin: Option<CardRef>,
        card: CardRef,
        filter_fn: impl FnMut(&&Modifier) -> bool,
    ) -> GameResult {
        let player = self.player_for_card(card);
        let zone = self
            .board(player)
            .find_card_zone(card)
            .expect("the card should be in a zone");

        let modifiers = self
            .state
            .card_modifiers
            .get(&card)
            .into_iter()
            .flatten()
            .filter(filter_fn)
            .map(|m| m.id)
            .collect();

        self.remove_many_modifiers_from_many_cards(
            event_origin,
            player,
            zone,
            vec![card],
            modifiers,
        )
    }

    pub fn clear_all_modifiers(
        &mut self,
        event_origin: Option<CardRef>,
        card: CardRef,
    ) -> GameResult {
        let player = self.player_for_card(card);
        let zone = self
            .board(player)
            .find_card_zone(card)
            .expect("the card should be in a zone");

        self.clear_all_modifiers_from_many_cards(event_origin, player, zone, vec![card])
    }

    pub fn promote_modifiers(&mut self, attachment: CardRef, parent: CardRef) {
        if let Some((_, modifiers)) = self.state.card_modifiers.remove_entry(&parent) {
            self.state
                .card_modifiers
                .entry(attachment)
                .or_default()
                .extend(modifiers);
        }
    }

    pub fn start_turn_modifiers(&mut self, player: Player) {
        // house keeping for the card modifiers
        // split in 2 because can't modify and player_for_card at the same time
        let to_modify: Vec<_> = self
            .state
            .card_modifiers
            .iter()
            .enumerate()
            .filter(|(_, (c, _))| self.player_for_card(**c) == player)
            .map(|(i, _)| i)
            .collect();
        self.state
            .card_modifiers
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| to_modify.contains(i))
            .flat_map(|(_, (_, ms))| ms)
            .for_each(|m| {
                m.start_turn(player);
            });

        // house keeping for the zone modifiers
        self.state
            .zone_modifiers
            .get_mut(&player)
            .into_iter()
            .flatten()
            .for_each(|(_, m)| {
                m.start_turn(player);
            });
    }

    pub fn end_turn_modifiers(&mut self, player: Player) {
        // house keeping for the card modifiers
        // split in 2 because can't modify and player_for_card at the same time
        let to_modify: Vec<_> = self
            .state
            .card_modifiers
            .iter()
            .enumerate()
            .filter(|(_, (c, _))| self.player_for_card(**c) == player)
            .map(|(i, _)| i)
            .collect();
        self.state
            .card_modifiers
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| to_modify.contains(i))
            .flat_map(|(_, (_, ms))| ms)
            .for_each(|m| {
                m.end_turn(player);
            });

        // house keeping for the zone modifiers
        self.state
            .zone_modifiers
            .get_mut(&player)
            .into_iter()
            .flatten()
            .for_each(|(_, m)| {
                m.end_turn(player);
            });
    }

    pub fn remove_expiring_modifiers(
        &mut self,
        event_origin: Option<CardRef>,
        life_time: LifeTime,
    ) -> GameResult {
        // remove expiring card modifiers
        let c_mods: HashMap<_, Vec<_>> = self
            .state
            .card_modifiers
            .iter()
            .flat_map(|(c, ms)| ms.iter().map(move |m| (c, m)))
            .filter(|(_, m)| m.life_time == life_time)
            .fold(HashMap::new(), |mut c_m, (c, m)| {
                let p = self.player_for_card(*c);
                let z = self
                    .board(p)
                    .find_card_zone(*c)
                    .expect("the card should be in a zone");
                c_m.entry((p, z, *c)).or_default().push(m.id);
                c_m
            });
        for ((p, z, c), m) in c_mods {
            self.remove_many_modifiers_from_many_cards(event_origin, p, z, vec![c], m)?;
        }

        // remove expiring zone modifiers
        let z_mods: HashMap<_, Vec<_>> = self
            .state
            .zone_modifiers
            .iter()
            .flat_map(|(p, ms)| ms.iter().map(move |(z, m)| (p, z, m)))
            .filter(|(_, _, m)| m.life_time == life_time)
            .fold(HashMap::new(), |mut z_m, (p, z, m)| {
                z_m.entry((*p, *z)).or_default().push(m.id);
                z_m
            });
        for ((p, z), m) in z_mods {
            self.remove_many_modifiers_from_zone(event_origin, p, z, m)?;
        }

        Ok(GameContinue)
    }

    // damage markers
    pub fn has_damage(&self, card: CardRef) -> bool {
        self.state.has_damage(card)
    }

    pub fn remaining_hp(&self, card: CardRef) -> HoloMemberHp {
        self.state.remaining_hp(card)
    }

    pub fn get_damage(&self, card: CardRef) -> DamageMarkers {
        self.state.get_damage(card)
    }

    pub fn add_damage_markers(
        &mut self,
        event_origin: Option<CardRef>,
        card: CardRef,
        dmg: DamageMarkers,
    ) -> GameResult {
        let player = self.player_for_card(card);
        let zone = self
            .board(player)
            .find_card_zone(card)
            .expect("the card should be in a zone");

        self.add_damage_markers_to_many_cards(event_origin, player, zone, vec![card], dmg)
    }

    pub fn remove_damage_markers(
        &mut self,
        event_origin: Option<CardRef>,
        card: CardRef,
        dmg: DamageMarkers,
    ) -> GameResult {
        let player = self.player_for_card(card);
        let zone = self
            .board(player)
            .find_card_zone(card)
            .expect("the card should be in a zone");

        self.remove_damage_markers_from_many_cards(event_origin, player, zone, vec![card], dmg)
    }

    pub fn promote_damage_markers(&mut self, attachment: CardRef, parent: CardRef) {
        if let Some((_, dmg)) = self.state.card_damage_markers.remove_entry(&parent) {
            *self
                .state
                .card_damage_markers
                .entry(attachment)
                .or_default() += dmg;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DamageMarkers(pub usize);

impl DamageMarkers {
    pub fn from_hp(hp: HoloMemberHp) -> Self {
        DamageMarkers(hp as usize / 10)
    }
    pub fn to_hp(self) -> HoloMemberHp {
        self.0 as HoloMemberHp * 10
    }
}

impl std::ops::Add for DamageMarkers {
    type Output = DamageMarkers;

    fn add(self, rhs: Self) -> Self::Output {
        DamageMarkers(self.0.saturating_add(rhs.0))
    }
}
impl std::ops::AddAssign for DamageMarkers {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl std::ops::Sub for DamageMarkers {
    type Output = DamageMarkers;

    fn sub(self, rhs: Self) -> Self::Output {
        DamageMarkers(self.0.saturating_sub(rhs.0))
    }
}
impl std::ops::SubAssign for DamageMarkers {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
