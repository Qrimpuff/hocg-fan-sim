use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::num::NonZeroU16;

use bincode::{Decode, Encode};
use get_size::GetSize;

use crate::card_effects::evaluate::EvaluateEffect;
use crate::card_effects::Condition;

use super::cards::*;
use super::gameplay::*;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, GetSize, Encode, Decode)]
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
impl From<&str> for ModifierRef {
    fn from(value: &str) -> Self {
        let hex = u16::from_str_radix(value.trim_start_matches("m_"), 16).unwrap();
        let num = NonZeroU16::new(hex).unwrap();
        ModifierRef(num)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, GetSize, Encode, Decode)]
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
    NoLifeLoss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, GetSize, Encode, Decode)]
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

#[derive(Debug, Clone, PartialEq, Eq, GetSize, Encode, Decode)]
pub struct Modifier {
    pub id: ModifierRef,
    pub kind: ModifierKind,
    pub life_time: LifeTime,
}

impl Modifier {
    pub fn for_card(
        _card: CardRef,
        kind: ModifierKind,
        life_time: LifeTime,
        next_modifier_ref: &mut u16,
    ) -> Self {
        let next_ref = *next_modifier_ref;
        *next_modifier_ref += 1;
        let id = ModifierRef(NonZeroU16::new(next_ref).expect("card is non zero"));
        Modifier {
            id,
            kind,
            life_time,
        }
    }
    pub fn for_zone(
        _player: Player,
        _zone: Zone,
        kind: ModifierKind,
        life_time: LifeTime,
        next_modifier_ref: &mut u16,
    ) -> Self {
        let next_ref = *next_modifier_ref;
        *next_modifier_ref += 1;
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
    pub fn start_turn_modifiers(&mut self, player: Player) {
        // house keeping for the card modifiers
        // split in 2 because can't modify and player_for_card at the same time
        let to_modify: Vec<_> = self
            .card_modifiers
            .iter()
            .enumerate()
            .filter(|(_, (c, _))| self.player_for_card(**c) == player)
            .map(|(i, _)| i)
            .collect();
        self.card_modifiers
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| to_modify.contains(i))
            .flat_map(|(_, (_, ms))| ms)
            .for_each(|m| {
                m.start_turn(player);
            });

        // house keeping for the zone modifiers
        self.zone_modifiers
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
            .card_modifiers
            .iter()
            .enumerate()
            .filter(|(_, (c, _))| self.player_for_card(**c) == player)
            .map(|(i, _)| i)
            .collect();
        self.card_modifiers
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| to_modify.contains(i))
            .flat_map(|(_, (_, ms))| ms)
            .for_each(|m| {
                m.end_turn(player);
            });

        // house keeping for the zone modifiers
        self.zone_modifiers
            .get_mut(&player)
            .into_iter()
            .flatten()
            .for_each(|(_, m)| {
                m.end_turn(player);
            });
    }

    pub fn promote_modifiers(&mut self, attachment: CardRef, parent: CardRef) {
        if let Some((_, modifiers)) = self.card_modifiers.remove_entry(&parent) {
            self.card_modifiers
                .entry(attachment)
                .or_default()
                .extend(modifiers);
        }
    }

    pub fn promote_damage_markers(&mut self, attachment: CardRef, parent: CardRef) {
        if let Some((_, dmg)) = self.card_damage_markers.remove_entry(&parent) {
            *self.card_damage_markers.entry(attachment).or_default() += dmg;
        }
    }
}

impl Game {
    pub fn find_modifiers(&self, card: CardRef) -> impl Iterator<Item = &Modifier> + '_ {
        let player = self.player_for_card(card);
        let zone = self.board(player).find_card_zone(card).unwrap_or(Zone::All);

        self.state
            .card_modifiers
            .get(&card)
            .into_iter()
            .flatten()
            .chain(
                self.state
                    .zone_modifiers
                    .get(&player)
                    .into_iter()
                    .flatten()
                    .filter(move |(z, _)| z.includes(zone))
                    .map(|(_, b)| b),
            )
    }
    pub fn find_player_modifiers(&self, player: Player) -> impl Iterator<Item = &Modifier> + '_ {
        // need to look for any card, oshi is always there
        let oshi = self
            .board(player)
            .get_zone(Zone::Oshi)
            .peek_top_card()
            .expect("oshi is always there");
        self.find_modifiers(oshi)
    }

    /// is used with Zone::All
    pub fn player_has_modifier(&self, player: Player, kind: ModifierKind) -> bool {
        self.player_has_modifier_with(player, |m| *m == kind)
    }
    pub fn player_has_modifier_with(
        &self,
        player: Player,
        filter_fn: impl FnMut(&ModifierKind) -> bool,
    ) -> bool {
        // need to look for any card, oshi is always there
        let oshi = self
            .board(player)
            .get_zone(Zone::Oshi)
            .peek_top_card()
            .expect("oshi is always there");
        self.has_modifier_with(oshi, filter_fn)
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
                ModifierKind::Conditional(c, k) => c
                    .ctx()
                    .with_card(card, self)
                    .evaluate(self)
                    .then_some(k.as_ref()),
                _ => Some(&m.kind),
            })
            .any(filter_fn)
    }

    // damage markers
    pub fn has_damage(&self, card: CardRef) -> bool {
        self.state
            .card_damage_markers
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
        self.state
            .card_damage_markers
            .get(&card)
            .copied()
            .unwrap_or_default()
    }
}

impl GameDirector {
    pub fn find_modifiers(&self, card: CardRef) -> impl Iterator<Item = &Modifier> + '_ {
        self.game.find_modifiers(card)
    }
    pub fn find_player_modifiers(&self, player: Player) -> impl Iterator<Item = &Modifier> + '_ {
        self.game.find_player_modifiers(player)
    }

    /// is used with Zone::All
    pub fn player_has_modifier(&self, player: Player, kind: ModifierKind) -> bool {
        self.game.player_has_modifier(player, kind)
    }
    pub fn player_has_modifier_with(
        &self,
        player: Player,
        filter_fn: impl FnMut(&ModifierKind) -> bool,
    ) -> bool {
        self.game.player_has_modifier_with(player, filter_fn)
    }
    pub fn has_modifier(&self, card: CardRef, kind: ModifierKind) -> bool {
        self.game.has_modifier(card, kind)
    }
    pub fn has_modifier_with(
        &self,
        card: CardRef,
        filter_fn: impl FnMut(&ModifierKind) -> bool,
    ) -> bool {
        self.game.has_modifier_with(card, filter_fn)
    }

    pub async fn add_modifier(
        &mut self,
        card: CardRef,
        kind: ModifierKind,
        life_time: LifeTime,
    ) -> GameResult {
        self.add_many_modifiers(card, vec![(kind, life_time)]).await
    }
    pub async fn add_many_modifiers(
        &mut self,
        card: CardRef,
        modifiers: Vec<(ModifierKind, LifeTime)>,
    ) -> GameResult {
        let modifiers = modifiers
            .into_iter()
            .map(|(kind, life_time)| {
                Modifier::for_card(card, kind, life_time, &mut self.next_modifier_ref)
            })
            .collect();
        self.add_many_modifiers_to_many_cards(vec![card], modifiers)
            .await
    }

    pub async fn add_zone_modifier(
        &mut self,
        player: Player,
        zone: Zone,
        kind: ModifierKind,
        life_time: LifeTime,
    ) -> GameResult {
        self.add_many_zone_modifiers(player, zone, kind, life_time, 1)
            .await
    }
    pub async fn add_many_zone_modifiers(
        &mut self,
        player: Player,
        zone: Zone,
        kind: ModifierKind,
        life_time: LifeTime,
        amount: usize,
    ) -> GameResult {
        let modifiers = (0..amount)
            .map(|_| {
                Modifier::for_zone(
                    player,
                    zone,
                    kind.clone(),
                    life_time,
                    &mut self.next_modifier_ref,
                )
            })
            .collect();
        self.add_many_modifiers_to_zone(player, zone, modifiers)
            .await
    }

    pub async fn remove_all_modifiers(&mut self, card: CardRef, kind: ModifierKind) -> GameResult {
        self.remove_all_modifiers_with(card, |m| m.kind == kind)
            .await
    }
    pub async fn remove_all_modifiers_with(
        &mut self,
        card: CardRef,
        filter_fn: impl FnMut(&&Modifier) -> bool,
    ) -> GameResult {
        let modifiers = self
            .game
            .state
            .card_modifiers
            .get(&card)
            .into_iter()
            .flatten()
            .filter(filter_fn)
            .map(|m| m.id)
            .collect();

        self.remove_many_modifiers_from_many_cards(vec![card], modifiers)
            .await
    }

    pub async fn remove_all_zone_modifiers(
        &mut self,
        player: Player,
        zone: Zone,
        kind: ModifierKind,
    ) -> GameResult {
        self.remove_all_zone_modifiers_with(player, zone, |(_, m)| m.kind == kind)
            .await
    }
    pub async fn remove_all_zone_modifiers_with(
        &mut self,
        player: Player,
        zone: Zone,
        filter_fn: impl FnMut(&&(Zone, Modifier)) -> bool,
    ) -> GameResult {
        let modifiers = self
            .game
            .state
            .zone_modifiers
            .get(&player)
            .into_iter()
            .flatten()
            .filter(move |(z, _)| z.includes(zone))
            .filter(filter_fn)
            .map(|(_, m)| m.id)
            .collect();

        self.remove_many_modifiers_from_zone(player, zone, modifiers)
            .await
    }

    pub async fn clear_all_modifiers(&mut self, card: CardRef) -> GameResult {
        self.clear_all_modifiers_from_many_cards(vec![card]).await
    }

    pub async fn remove_expiring_modifiers(&mut self, life_time: LifeTime) -> GameResult {
        // remove expiring card modifiers
        let c_mods: HashMap<_, Vec<_>> = self
            .game
            .state
            .card_modifiers
            .iter()
            .flat_map(|(c, ms)| ms.iter().map(move |m| (c, m)))
            .filter(|(_, m)| m.life_time == life_time)
            .fold(HashMap::new(), |mut c_m, (c, m)| {
                c_m.entry(*c).or_default().push(m.id);
                c_m
            });
        for (c, m) in c_mods {
            self.remove_many_modifiers_from_many_cards(vec![c], m)
                .await?;
        }

        // remove expiring zone modifiers
        let z_mods: HashMap<_, Vec<_>> = self
            .game
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
            self.remove_many_modifiers_from_zone(p, z, m).await?;
        }

        Ok(GameContinue)
    }

    // damage markers
    pub fn has_damage(&self, card: CardRef) -> bool {
        self.game.has_damage(card)
    }

    pub fn remaining_hp(&self, card: CardRef) -> HoloMemberHp {
        self.game.remaining_hp(card)
    }

    pub fn get_damage(&self, card: CardRef) -> DamageMarkers {
        self.game.get_damage(card)
    }

    pub async fn add_damage_markers(&mut self, card: CardRef, dmg: DamageMarkers) -> GameResult {
        self.add_damage_markers_to_many_cards(vec![card], dmg).await
    }

    pub async fn remove_damage_markers(&mut self, card: CardRef, dmg: DamageMarkers) -> GameResult {
        self.remove_damage_markers_from_many_cards(vec![card], dmg)
            .await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, GetSize, Encode, Decode)]
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
