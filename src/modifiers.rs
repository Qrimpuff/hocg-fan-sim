use std::collections::HashMap;

use super::cards::*;
use super::gameplay::*;

use ModifierKind::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierKind {
    // attributes
    /// an instance of damage received, is added multiple times
    DamageMarker,
    // buff
    // debuff
    Rested,
    PreventAbility(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modifier {
    pub kind: ModifierKind,
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

impl Game {
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
        self.add_many_modifiers(card, kind, life_time, 1);
    }

    pub fn add_many_modifiers(
        &mut self,
        card: CardRef,
        kind: ModifierKind,
        life_time: LifeTime,
        amount: usize,
    ) {
        let player = self.player_for_card(card);
        let zone = self
            .board(player)
            .find_card_zone(card)
            .expect("the card should be in a zone");
        self.add_many_modifiers_to_many_cards(
            player,
            zone,
            vec![card],
            (0..amount)
                .map(move |_| Modifier::new(kind, life_time))
                .collect(),
        )
    }

    pub fn remove_modifier(&mut self, card: CardRef, kind: ModifierKind) {
        self.remove_many_modifiers(card, kind, 1);
    }

    pub fn remove_many_modifiers(&mut self, card: CardRef, kind: ModifierKind, amount: usize) {
        let player = self.player_for_card(card);
        let zone = self
            .board(player)
            .find_card_zone(card)
            .expect("the card should be in a zone");

        let modifiers = self
            .card_modifiers
            .iter()
            .filter(|(c, m)| *c == card && m.kind == kind)
            .map(|(_, m)| m.kind)
            .take(amount)
            .collect();

        self.remove_many_modifiers_from_many_cards(player, zone, vec![card], modifiers);
    }

    pub fn remove_all_modifiers(&mut self, card: CardRef, kind: ModifierKind) {
        self.remove_many_modifiers(card, kind, usize::MAX);
    }

    pub fn clear_all_modifiers(&mut self, card: CardRef) {
        let player = self.player_for_card(card);
        let zone = self
            .board(player)
            .find_card_zone(card)
            .expect("the card should be in a zone");

        let modifiers = self
            .card_modifiers
            .iter()
            .filter(|(c, _)| *c == card)
            .map(|(_, m)| m.kind)
            .collect();

        self.remove_many_modifiers_from_many_cards(player, zone, vec![card], modifiers);
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
        // increase the life counter of the card modifiers
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

        // increase the life counter of the zone modifiers
        self.zone_modifiers
            .iter_mut()
            .filter(|(p, _, _)| *p == player)
            .for_each(|(_, _, m)| {
                m.end_turn();
            });

        // remove expiring card modifiers
        let c_mods: HashMap<_, Vec<_>> = self
            .card_modifiers
            .iter()
            .filter(|(_, m)| !m.survive_end_turn())
            .fold(HashMap::new(), |mut c_m, (c, m)| {
                let p = self.player_for_card(*c);
                let z = self
                    .board(p)
                    .find_card_zone(*c)
                    .expect("the card should be in a zone");
                c_m.entry((p, z, *c)).or_default().push(m.kind);
                c_m
            });
        for ((p, z, c), m) in c_mods {
            self.remove_many_modifiers_from_many_cards(p, z, vec![c], m);
        }

        // remove expiring zone modifiers
        let z_mods: HashMap<_, Vec<_>> = self
            .zone_modifiers
            .iter()
            .filter(|(_, _, m)| !m.survive_end_turn())
            .fold(HashMap::new(), |mut z_m, (p, z, m)| {
                z_m.entry((*p, *z)).or_default().push(m.kind);
                z_m
            });
        for ((p, z), m) in z_mods {
            self.remove_many_modifiers_from_zone(p, z, m);
        }
    }

    // common modifiers
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageMarkers(pub usize);

impl DamageMarkers {
    pub fn from_hp(hp: HoloMemberHp) -> Self {
        DamageMarkers(hp as usize / 10)
    }
    pub fn to_hp(self) -> HoloMemberHp {
        self.0 as HoloMemberHp * 10
    }
}
