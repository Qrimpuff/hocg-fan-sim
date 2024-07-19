use std::{collections::HashMap, path::Display};

use crate::{
    modifiers::DamageMarkers, Action, Condition, DamageModifier, Error, ParseEffect, ParseTokens,
    SerializeEffect, Trigger,
};

/**
 * Cards:
- oshi:
  - set id
  - name
  - life
  - color
  - rarity
  - illustration
  - artist
  - abilities
    - cost
    - name
    - pub type
    - effect

- member:
  - set id
  - name
  - hp
  - color
  - rarity
  - illustration
  - artist
  - debut / 1st / 2nd
  - tags
  - retreat cost (baton touch / pass?)
  - abilities
    - name
    - pub type
    - effect
    - text
    - condition? extracted from effect?
  - attack
    - name
    - cost
    - damage
    - effect
    - text
    - condition?

- item / support:
  - set id
  - name
  - pub type
  - rarity
  - illustration
  - artist
  - effect
  - text
  - support? mascot?
  - limited use?

- cheer:
  - set id
  - name
  - rarity
  - illustration
  - artist
  - effect
  - color
  - text
 */

#[derive(Debug)]
pub struct Loadout {
    pub oshi: CardNumber,
    pub main_deck: Vec<CardNumber>,
    pub cheer_deck: Vec<CardNumber>,
    // cosmetic...
}

#[derive(Debug)]
pub struct GlobalLibrary {
    pub cards: HashMap<CardNumber, Card>,
}

impl GlobalLibrary {
    /// Any pre-processing of cards that could make my life easier later
    pub fn pre_process(&mut self) {
        // TODO oshi skill once turn

        // TODO special oshi skill once per game

        // TODO enough holo power to pay the cost for oshi skill

        // TODO enough cheers to perform art for members

        // TODO limited support

        // TODO if you can't select something, it should check that it's there first in condition

        // default condition to always
        let default_condition = Condition::Always;
        let default_action = Action::Noop;
        let default_damage_mod = DamageModifier::None;
        for card in self.cards.values_mut() {
            match card {
                Card::OshiHoloMember(o) => o.skills.iter_mut().for_each(|s| {
                    if s.condition.is_empty() {
                        s.condition.push(default_condition.clone())
                    }
                    if s.effect.is_empty() {
                        s.effect.push(default_action.clone())
                    }
                }),
                Card::HoloMember(m) => {
                    m.abilities.iter_mut().for_each(|a| {
                        if a.condition.is_empty() {
                            a.condition.push(default_condition.clone())
                        }
                        if a.effect.is_empty() {
                            a.effect.push(default_action.clone())
                        }
                    });
                    m.arts.iter_mut().for_each(|a| {
                        if a.condition.is_empty() {
                            a.condition.push(default_condition.clone())
                        }
                        if a.damage_modifier.is_empty() {
                            a.damage_modifier.push(default_damage_mod)
                        }
                        if a.effect.is_empty() {
                            a.effect.push(default_action.clone())
                        }
                    })
                }
                Card::Support(s) => {
                    if s.condition.is_empty() {
                        s.condition.push(default_condition.clone())
                    }
                    if s.effect.is_empty() {
                        s.effect.push(default_action.clone())
                    }
                }
                Card::Cheer(_) => {} // cheers do not have conditions
            }
        }

        // verify effect serialization consistency (de -> ser -> de)
        fn serialization_round_trip<T>(effect: T) -> crate::Result<()>
        where
            T: SerializeEffect + ParseTokens + PartialEq + Clone,
        {
            let string = effect.clone().serialize_effect();
            let de_effect = string.parse_effect::<T>()?;

            if effect != de_effect {
                Err(Error::Message(
                    "effect could not do serialization round trip".into(),
                ))
            } else {
                Ok(())
            }
        }
        let mut has_errors = false;
        for card in self.cards.values_mut() {
            match card {
                Card::OshiHoloMember(o) => o.skills.iter_mut().for_each(|s| {
                    if let Err(e) = serialization_round_trip(s.condition.clone()) {
                        eprintln!("{}: {} - condition - {}", o.card_number, s.name, e);
                        has_errors = true;
                    }
                    if let Err(e) = serialization_round_trip(s.effect.clone()) {
                        eprintln!("{}: {} - effect - {}", o.card_number, s.name, e);
                        has_errors = true;
                    }
                }),
                Card::HoloMember(m) => {
                    m.abilities.iter_mut().for_each(|a| {
                        if let Err(e) = serialization_round_trip(a.condition.clone()) {
                            eprintln!("{}: {} - condition - {}", m.card_number, a.name, e);
                            has_errors = true;
                        }
                        if let Err(e) = serialization_round_trip(a.effect.clone()) {
                            eprintln!("{}: {} - effect - {}", m.card_number, a.name, e);
                            has_errors = true;
                        }
                    });
                    m.arts.iter_mut().for_each(|a| {
                        if let Err(e) = serialization_round_trip(a.condition.clone()) {
                            eprintln!("{}: {} - condition - {}", m.card_number, a.name, e);
                            has_errors = true;
                        }
                        if let Err(e) = serialization_round_trip(a.damage_modifier.clone()) {
                            eprintln!("{}: {} - damage modifier - {}", m.card_number, a.name, e);
                            has_errors = true;
                        }
                        if let Err(e) = serialization_round_trip(a.effect.clone()) {
                            eprintln!("{}: {} - effect - {}", m.card_number, a.name, e);
                            has_errors = true;
                        }
                    })
                }
                Card::Support(s) => {
                    if let Err(e) = serialization_round_trip(s.condition.clone()) {
                        eprintln!("{}: {} - condition - {}", s.card_number, s.name, e);
                        has_errors = true;
                    }
                    if let Err(e) = serialization_round_trip(s.effect.clone()) {
                        eprintln!("{}: {} - effect - {}", s.card_number, s.name, e);
                        has_errors = true;
                    }
                }
                Card::Cheer(_) => {} // cheers do not have effects
            }
        }
        if has_errors {
            panic!("effect serialization is not consistent")
        }
    }

    pub fn lookup_card(&self, card_number: &CardNumber) -> Option<&Card> {
        self.cards.get(card_number)
    }
}

#[derive(Debug)]
pub enum Card {
    OshiHoloMember(OshiHoloMemberCard),
    HoloMember(HoloMemberCard),
    Support(SupportCard),
    Cheer(CheerCard),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Rarity {
    OshiSuperRare, // OSR
    DoubleRare,    // RR
    Rare,          // R
    Uncommon,      // U
    Common,        // C
    Secret,        // SEC
    OshiUltraRare, // OUR
    UltraRare,     // UR
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(clippy::enum_variant_names)]
pub enum Color {
    ColorLess,
    White,
    Green,
    Blue,
    Red,
    Purple,
    Yellow,
}

pub type CardNumber = String;
pub type IllustrationPath = String;
pub type OshiLife = u8;
pub type HoloMemberHp = u16;
pub type OshiSkillCost = u8;
pub type HoloMemberArtCost = Vec<Color>;
pub type CardEffectTrigger = Vec<Trigger>;
pub type CardEffectCondition = Vec<Condition>;
pub type CardEffectDamageModifier = Vec<DamageModifier>;
pub type CardEffect = Vec<Action>;
pub type HoloMemberBatonPassCost = u8;

#[derive(Debug)]
pub struct OshiHoloMemberCard {
    pub card_number: CardNumber,
    pub name: String,
    pub color: Color,
    pub life: OshiLife,
    pub skills: Vec<OshiSkill>,
    pub rarity: Rarity,
    pub illustration: IllustrationPath,
    pub artist: String,
}

#[derive(Debug)]
pub struct OshiSkill {
    pub kind: OshiSkillKind,
    pub name: String,
    pub cost: OshiSkillCost,
    pub text: String,
    pub trigger: CardEffectTrigger,
    pub condition: CardEffectCondition,
    pub effect: CardEffect,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OshiSkillKind {
    Normal,
    Special,
}

#[derive(Debug)]
pub struct HoloMemberCard {
    pub card_number: CardNumber,
    pub name: String,
    pub color: Color,
    pub hp: HoloMemberHp,
    pub level: HoloMemberLevel,
    pub tags: Vec<HoloMemberTag>,
    pub baton_pass_cost: HoloMemberBatonPassCost,
    pub abilities: Vec<HoloMemberAbility>,
    pub arts: Vec<HoloMemberArt>,
    pub extra: Option<String>, // TODO will probably be an enum
    pub rarity: Rarity,
    pub illustration: IllustrationPath,
    pub artist: String,
}

impl HoloMemberCard {
    pub fn names(&self) -> impl Iterator<Item = &String> + '_ {
        Some(&self.name)
            .into_iter()
            .chain(self.tags.iter().filter_map(|t| match t {
                HoloMemberTag::Name(n) => Some(n),
                _ => None,
            }))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HoloMemberLevel {
    Debut,
    First,
    Second,
    Spot,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HoloMemberTag {
    JP,
    ID,
    EN,
    Generation0,
    Generation1,
    Generation2,
    Generation3,
    Generation4,
    Generation5,
    Name(String),
}

#[derive(Debug)]
pub struct HoloMemberAbility {
    pub kind: MemberAbilityKind,
    pub name: String,
    pub text: String,
    pub condition: CardEffectCondition,
    pub effect: CardEffect,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemberAbilityKind {
    CollabEffect,
    BloomEffect,
    Gift(CardEffectTrigger), // TODO verify if gift is correct. what kind of effect they have?
}

#[derive(Debug)]
pub struct HoloMemberArt {
    pub name: String,
    pub cost: HoloMemberArtCost,
    pub damage: HoloMemberArtDamage,
    pub text: String,
    pub condition: CardEffectCondition,
    pub damage_modifier: CardEffectDamageModifier,
    pub effect: CardEffect,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HoloMemberArtDamage {
    Basic(HoloMemberHp),
    Plus(HoloMemberHp),
    Minus(HoloMemberHp),
    Uncertain,
}

#[derive(Debug)]
pub struct SupportCard {
    pub card_number: CardNumber,
    pub name: String,
    pub kind: SupportKind,
    // limited_use: bool, // TODO limited is needed, but not sure how
    pub text: String,
    pub condition: CardEffectCondition,
    pub effect: CardEffect,
    pub rarity: Rarity,
    pub illustration: IllustrationPath,
    pub artist: String,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SupportKind {
    Item,
    Staff, // TODO verify if this is true
}

#[derive(Debug)]
pub struct CheerCard {
    pub card_number: CardNumber,
    pub name: String,
    pub color: Color,
    pub text: String,
    pub rarity: Rarity,
    pub illustration: IllustrationPath,
    pub artist: String,
}
