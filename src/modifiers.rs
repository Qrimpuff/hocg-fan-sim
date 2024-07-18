use super::cards::*;
use super::gameplay::*;

use ModifierKind::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModifierKind {
    // attributes
    /// an instance of damage received, is added multiple times
    DamageMarker,
    // buff
    // debuff
    Rested,
    PreventAbility(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LifeTime {
    /// stays until their player's next end step
    Turns(u8, u8),
    /// stays until manually removed
    Unlimited,
}

impl LifeTime {
    pub fn this_turn() -> Self {
        LifeTime::Turns(0, 0)
    }
    pub fn next_turn() -> Self {
        LifeTime::next_turns(1)
    }
    pub fn next_turns(count: u8) -> Self {
        LifeTime::Turns(1, count)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Modifier {
    kind: ModifierKind,
    life_time: LifeTime,
    turn_count: u8,
    // TODO active on specific player's turn?
}

impl Modifier {
    pub fn new(kind: ModifierKind, life_time: LifeTime) -> Self {
        Modifier {
            kind,
            life_time,
            turn_count: 0,
        }
    }

    pub fn is_active(&self) -> bool {
        match self.life_time {
            LifeTime::Turns(min, max) => (min..=max).contains(&self.turn_count),
            LifeTime::Unlimited => true,
        }
    }

    pub fn start_turn(&mut self) {}

    pub fn end_turn(&mut self) {
        self.turn_count += 1;
    }

    pub fn survive_end_turn(&self) -> bool {
        match self.life_time {
            LifeTime::Turns(_, max) => self.turn_count <= max,
            LifeTime::Unlimited => true,
        }
    }
}

impl<P: Prompter> Game<P> {
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

        self.card_modifiers
            .iter()
            .filter(move |(c, _)| *c == card)
            .map(|(_, b)| b)
            .chain(
                self.zone_modifiers
                    .iter()
                    .filter(move |(p, z, _)| p == player && *z == zone || *z == Zone::All)
                    .map(|(_, _, b)| b),
            )
    }

    pub fn has_modifier(&self, card: CardRef, kind: ModifierKind) -> bool {
        self.find_modifiers(card)
            .filter(|m| m.is_active())
            .any(|m| m.kind == kind)
    }

    pub fn add_modifier(&mut self, card: CardRef, kind: ModifierKind, life_time: LifeTime) {
        self.card_modifiers
            .push((card, Modifier::new(kind, life_time)));
    }

    pub fn add_many_modifiers(
        &mut self,
        card: CardRef,
        kind: ModifierKind,
        life_time: LifeTime,
        amount: usize,
    ) {
        self.card_modifiers
            .extend((0..amount).map(move |_| (card, Modifier::new(kind, life_time))));
    }

    pub fn remove_modifier(&mut self, card: CardRef, kind: ModifierKind) {
        self.remove_many_modifiers(card, kind, 1);
    }

    pub fn remove_many_modifiers(&mut self, card: CardRef, kind: ModifierKind, amount: usize) {
        let mut count = 0;
        self.card_modifiers.retain(|(c, m)| {
            if *c != card || m.kind != kind || count >= amount {
                true
            } else {
                count += 1;
                false
            }
        });
    }

    pub fn remove_all_modifiers(&mut self, card: CardRef, kind: ModifierKind) {
        self.card_modifiers
            .retain(|(c, m)| *c != card || m.kind != kind);
    }

    pub fn clear_all_modifiers(&mut self, card: CardRef) {
        self.card_modifiers.retain(|(c, _)| *c != card);
    }

    pub fn promote_modifiers(&mut self, attachment: CardRef, parent: CardRef) {
        self.card_modifiers
            .iter_mut()
            .filter(|(c, _)| *c == parent)
            .for_each(|(c, _)| *c = attachment);
    }

    pub fn start_turn_modifiers(&mut self, player: Player) {
        // split in 2 because can't modify and player_for_card at the same time
        let to_modify: Vec<_> = self
            .card_modifiers
            .iter()
            .enumerate()
            .filter(|(_, (c, _))| self.player_for_card(*c) == player)
            .map(|(i, _)| i)
            .collect();
        self.card_modifiers
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| to_modify.contains(i))
            .for_each(|(_, (_, m))| {
                m.start_turn();
            });

        self.zone_modifiers
            .iter_mut()
            .filter(|(p, _, _)| *p == player)
            .for_each(|(_, _, m)| {
                m.start_turn();
            });
    }

    pub fn end_turn_modifiers(&mut self, player: Player) {
        // split in 2 because can't modify and player_for_card at the same time
        let to_modify: Vec<_> = self
            .card_modifiers
            .iter()
            .enumerate()
            .filter(|(_, (c, _))| self.player_for_card(*c) == player)
            .map(|(i, _)| i)
            .collect();
        self.card_modifiers
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| to_modify.contains(i))
            .for_each(|(_, (_, m))| {
                m.end_turn();
            });

        self.zone_modifiers
            .iter_mut()
            .filter(|(p, _, _)| *p == player)
            .for_each(|(_, _, m)| {
                m.end_turn();
            });

        self.card_modifiers.retain(|(_, m)| m.survive_end_turn());
        self.zone_modifiers.retain(|(_, _, m)| m.survive_end_turn());
    }

    pub fn is_rested(&self, card: CardRef) -> bool {
        self.has_modifier(card, Rested)
    }
    pub fn rest_card(&mut self, card: CardRef) {
        self.add_modifier(card, Rested, LifeTime::Unlimited)
    }
    pub fn awake_card(&mut self, card: CardRef) {
        self.remove_modifier(card, Rested)
    }

    pub fn remaining_hp(&self, card: CardRef) -> HoloMemberHp {
        let dmg = self
            .find_modifiers(card)
            .filter(|m| m.is_active())
            .filter(|m| m.kind == DamageMarker)
            .count();
        let hp = self
            .lookup_holo_member(card)
            .expect("should be a member")
            .hp;
        hp.saturating_sub(DamageMarkers(dmg).to_hp())
    }
    pub fn add_damage(&mut self, card: CardRef, dmg: DamageMarkers) {
        self.add_many_modifiers(card, DamageMarker, LifeTime::Unlimited, dmg.0);
    }
    pub fn remove_damage(&mut self, card: CardRef, dmg: DamageMarkers) {
        self.remove_many_modifiers(card, DamageMarker, dmg.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageMarkers(pub usize);

impl DamageMarkers {
    pub fn from_hp(hp: HoloMemberHp) -> Self {
        DamageMarkers(hp as usize / 10)
    }
    pub fn to_hp(self) -> HoloMemberHp {
        self.0 as HoloMemberHp * 10
    }
}
