use std::{collections::HashMap, path::Display};

use crate::{modifiers::DamageMarkers, Action, Condition, DamageModifier, Trigger};

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
pub type CardEffectTrigger = Trigger;
pub type CardEffectCondition = Condition;
pub type CardEffectDamageModifier = DamageModifier;
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
    pub trigger: Option<CardEffectTrigger>,
    pub condition: Option<CardEffectCondition>,
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
    pub rank: HoloMemberRank,
    pub tags: Vec<HoloMemberTag>,
    pub baton_pass_cost: HoloMemberBatonPassCost,
    pub abilities: Vec<HoloMemberAbility>,
    pub arts: Vec<HoloMemberArt>,
    pub extra: Option<String>, // TODO will probably be an enum
    pub rarity: Rarity,
    pub illustration: IllustrationPath,
    pub artist: String,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HoloMemberRank {
    Debut,
    First,
    Second,
    Spot,
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
}

#[derive(Debug)]
pub struct HoloMemberAbility {
    pub kind: MemberAbilityKind,
    pub name: String,
    pub text: String,
    pub condition: Option<CardEffectCondition>,
    pub effect: CardEffect,
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
    pub condition: Option<CardEffectCondition>,
    pub damage_modifier: Option<CardEffectDamageModifier>,
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
    pub trigger: Option<CardEffectTrigger>,
    pub condition: Option<CardEffectCondition>,
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
    pub trigger: Option<CardEffectTrigger>,
    pub condition: Option<CardEffectCondition>,
    pub effect: Option<CardEffect>,
    pub rarity: Rarity,
    pub illustration: IllustrationPath,
    pub artist: String,
}
