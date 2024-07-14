use std::collections::HashMap;

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

pub type CardRef = CardId;

#[derive(Debug)]
pub struct Loadout {
    pub oshi: CardRef,
    pub main_deck: Vec<CardRef>,
    pub cheer_deck: Vec<CardRef>,
    // cosmetic...
}

#[derive(Debug)]
pub struct GlobalLibrary {
    pub cards: HashMap<CardRef, Card>,
}

impl GlobalLibrary {
    pub fn lookup_card(&self, card_ref: &CardRef) -> Option<&Card> {
        self.cards.get(card_ref)
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Color {
    White,
    Green,
    Blue,
    Red,
    Purple,
    Yellow,
}

pub type CardId = String;
pub type IllustrationPath = String;
pub type OshiLife = u8;
pub type OshiAbilityCost = u8;
pub type CardEffect = ();
pub type HoloMemberHp = u16;
pub type HoloMemberTag = String;
pub type HoloMemberBatonPassCost = u8;
pub type HoloMemberAttackCost = String;
pub type HoloMemberAttackDamage = HoloMemberHp;

#[derive(Debug)]
pub struct OshiHoloMemberCard {
    pub id: CardId,
    pub name: String,
    pub rarity: Rarity,
    pub illustration: IllustrationPath,
    pub artist: String,
    pub color: Color,
    pub life: OshiLife,
    pub abilities: Vec<OshiAbility>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OshiAbilityKind {
    Unknown,
}

#[derive(Debug)]
pub struct OshiAbility {
    pub kind: OshiAbilityKind,
    pub name: String,
    pub cost: OshiAbilityCost,
    pub effect: CardEffect,
    pub text: String,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HoloMemberRank {
    Debut,
    First,
    Second,
}

#[derive(Debug)]
pub struct HoloMemberCard {
    pub id: CardId,
    pub name: String,
    pub rarity: Rarity,
    pub illustration: IllustrationPath,
    pub artist: String,
    pub color: Color,
    pub hp: HoloMemberHp,
    pub rank: HoloMemberRank,
    pub tags: Vec<HoloMemberTag>,
    pub baton_pass: HoloMemberBatonPassCost,
    pub abilities: Vec<MemberAbility>,
    pub attacks: Vec<MemberAttack>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MemberAbilityKind {
    Unknown,
}

#[derive(Debug)]
pub struct MemberAbility {
    pub kind: MemberAbilityKind,
    pub name: String,
    pub effect: CardEffect,
    pub text: String,
}

#[derive(Debug)]
pub struct MemberAttack {
    pub name: String,
    pub cost: HoloMemberAttackCost,
    pub damage: HoloMemberAttackDamage,
    pub effect: CardEffect,
    pub text: String,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SupportKind {
    Unknown,
}

#[derive(Debug)]
pub struct SupportCard {
    pub id: CardId,
    pub name: String,
    pub rarity: Rarity,
    pub illustration: IllustrationPath,
    pub artist: String,
    pub kind: SupportKind,
    pub effect: CardEffect,
    pub text: String,
    // support_mascot: String,
    // limited_use: String,
}

#[derive(Debug)]
pub struct CheerCard {
    pub id: CardId,
    pub name: String,
    pub rarity: Rarity,
    pub illustration: IllustrationPath,
    pub artist: String,
    pub color: Color,
    pub effect: Option<CardEffect>,
    pub text: String,
}
