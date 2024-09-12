use std::num::ParseIntError;

use bincode::{Decode, Encode};
use evaluate::EvaluateEffect;
use get_size::GetSize;
use iter_tools::Itertools;
use serde::{Deserialize, Serialize};

use crate::card_effects::{
    effects::{
        deserialize_actions, deserialize_conditions, serialize_actions, serialize_conditions,
        skip_default_actions, skip_default_conditions,
    },
    *,
};
use crate::events::{Bloom, Collab, Event, TriggeredEvent};
use crate::gameplay::Zone;
use crate::gameplay::{CardRef, GameDirector};
use crate::modifiers::ModifierKind::*;

/*
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

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "card_type")]
pub enum Card {
    OshiHoloMember(OshiHoloMemberCard),
    HoloMember(HoloMemberCard),
    Support(SupportCard),
    Cheer(CheerCard),
}

impl Card {
    pub fn card_number(&self) -> &str {
        match self {
            Card::OshiHoloMember(c) => &c.card_number,
            Card::HoloMember(c) => &c.card_number,
            Card::Support(c) => &c.card_number,
            Card::Cheer(c) => &c.card_number,
        }
    }
    pub fn illustration_url(&self) -> &str {
        match self {
            Card::OshiHoloMember(c) => &c.illustration_url,
            Card::HoloMember(c) => &c.illustration_url,
            Card::Support(c) => &c.illustration_url,
            Card::Cheer(c) => &c.illustration_url,
        }
    }

    pub fn is_attribute(&self, attribute: HoloMemberExtraAttribute) -> bool {
        match self {
            Card::HoloMember(m) => m.attributes.contains(&attribute),
            _ => false,
        }
    }

    pub fn is_color(&self, color: Color) -> bool {
        match self {
            Card::OshiHoloMember(o) => o.color == color,
            Card::HoloMember(m) => m.colors.contains(&color),
            Card::Support(_s) => false,
            Card::Cheer(c) => c.color == color,
        }
    }

    pub fn is_cheer(&self) -> bool {
        matches!(self, Card::Cheer(_))
    }

    pub fn is_level(&self, level: HoloMemberLevel) -> bool {
        match self {
            Card::HoloMember(m) => m.level == level,
            _ => false,
        }
    }

    pub fn is_member(&self) -> bool {
        matches!(self, Card::HoloMember(_))
    }

    pub fn is_named(&self, name: &str) -> bool {
        match self {
            Card::HoloMember(m) => m.names().contains(&name.to_string()),
            _ => false,
        }
    }

    pub fn is_support_limited(&self) -> bool {
        match self {
            Card::Support(s) => s.limited,
            _ => false,
        }
    }
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, GetSize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    #[serde(rename = "osr")]
    OshiSuperRare, // OSR
    #[serde(rename = "rr")]
    DoubleRare, // RR
    #[serde(rename = "r")]
    Rare, // R
    #[serde(rename = "u")]
    Uncommon, // U
    #[serde(rename = "c")]
    Common, // C
    #[serde(rename = "sec")]
    Secret, // SEC
    #[serde(rename = "our")]
    OshiUltraRare, // OUR
    #[serde(rename = "ur")]
    UltraRare, // UR
}

#[derive(
    Encode,
    Decode,
    Serialize,
    Deserialize,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    GetSize,
)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum Color {
    White,
    Green,
    Red,
    Blue,
    Purple,
    Yellow,
    Colorless,
}

pub type CardNumber = String;
pub type IllustrationUrl = String;
pub type OshiLife = u8;
pub type HoloMemberHp = u16;
pub type OshiSkillCost = u8;
pub type HoloMemberArtCost = Vec<Color>;
pub type CardEffectTrigger = Vec<Trigger>;
pub type CardEffectCondition = Vec<Condition>;
pub type CardEffect = Vec<Action>;
pub type HoloMemberBatonPassCost = u8;

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
pub struct OshiHoloMemberCard {
    pub card_number: CardNumber,
    pub name: String,
    pub color: Color,
    pub life: OshiLife,
    pub skills: Vec<OshiSkill>,
    pub rarity: Rarity,
    pub illustration_url: IllustrationUrl,
    pub artist: String,
}

impl OshiHoloMemberCard {
    pub fn can_use_skill(
        &self,
        card: CardRef,
        skill_idx: usize,
        game: &GameDirector,
        is_triggered: bool,
    ) -> bool {
        let player = game.player_for_card(card);

        //  need the required holo power cost
        // TODO could have a buff that could pay for the skill
        let holo_power_count = game.board(player).get_zone(Zone::HoloPower).count();
        if holo_power_count < self.skills[skill_idx].cost.into() {
            return false;
        }

        //  cannot use the same skill twice in a turn
        if game.has_modifier(card, PreventOshiSkill(skill_idx)) {
            return false;
        }

        self.skills[skill_idx]
            .condition
            .ctx()
            .with_card(card, &game.game)
            .with_triggered(is_triggered)
            .evaluate(&game.game)
    }
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
pub struct OshiSkill {
    pub kind: OshiSkillKind,
    pub name: String,
    pub cost: OshiSkillCost,
    pub text: String,
    pub triggers: CardEffectTrigger,
    #[serde(serialize_with = "serialize_conditions")]
    #[serde(deserialize_with = "deserialize_conditions")]
    #[serde(skip_serializing_if = "skip_default_conditions")]
    pub condition: CardEffectCondition,
    #[serde(serialize_with = "serialize_actions")]
    #[serde(deserialize_with = "deserialize_actions")]
    #[serde(skip_serializing_if = "skip_default_actions")]
    pub effect: CardEffect,
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, GetSize)]
#[serde(rename_all = "snake_case")]
pub enum OshiSkillKind {
    Normal,
    Special,
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
pub struct HoloMemberCard {
    pub card_number: CardNumber,
    pub name: String,
    pub colors: Vec<Color>,
    pub hp: HoloMemberHp,
    pub level: HoloMemberLevel,
    pub hash_tags: Vec<HoloMemberHashTag>,
    pub baton_pass_cost: HoloMemberBatonPassCost,
    pub abilities: Vec<HoloMemberAbility>,
    pub arts: Vec<HoloMemberArt>,
    pub attributes: Vec<HoloMemberExtraAttribute>,
    pub rarity: Rarity,
    pub illustration_url: IllustrationUrl,
    pub artist: String,
}

impl HoloMemberCard {
    pub fn names(&self) -> impl Iterator<Item = &String> + Clone + '_ {
        Some(&self.name)
            .into_iter()
            .chain(self.attributes.iter().filter_map(|t| match t {
                HoloMemberExtraAttribute::Name(n) => Some(n),
                _ => None,
            }))
    }

    pub fn can_baton_pass(&self, card: CardRef, game: &GameDirector) -> bool {
        let player = game.player_for_card(card);

        // can only baton pass once per turn
        if game.has_modifier(card, PreventBatonPass) {
            return false;
        }

        // cannot baton pass if resting
        if game.has_modifier(card, Resting) {
            return false;
        }

        // can only baton pass if there are members on back stage, that are not resting
        if game
            .board(player)
            .back_stage()
            .all(|b| game.has_modifier(b, Resting))
        {
            return false;
        }

        // can only baton pass if there is enough cheers attached
        let cost = std::iter::repeat(Color::Colorless)
            .take(self.baton_pass_cost as usize)
            .collect_vec();
        if !game.required_attached_cheers(card, &cost) {
            return false;
        }

        true
    }

    pub fn can_bloom_target(
        &self,
        _card: CardRef,
        game: &GameDirector,
        target: (CardRef, &HoloMemberCard),
    ) -> bool {
        // debut and spot members cannot bloom anything
        if self.level == HoloMemberLevel::Debut || self.level == HoloMemberLevel::Spot {
            return false;
        }

        // cannot bloom to lower level
        if self.level < target.1.level {
            return false;
        }

        // cannot skip a level
        if self.level as usize - target.1.level as usize > 1 {
            return false;
        }

        //  cannot bloom the member twice in a turn
        if game.has_modifier(target.0, PreventBloom) {
            return false;
        }

        //  cannot bloom if the damage is more the bloom hp, it would be defeated instantly
        if game.get_damage(target.0).to_hp() >= self.hp {
            return false;
        }

        // TODO not sure if the name needs to be consistent with debut? e.i. Sora -> Sora/AZKi -> AZKi
        // need to have a common name, with every previous bloom
        let names = Some(self.names())
            .into_iter()
            .chain(Some(target.1.names()))
            .chain(
                game.board_for_card(target.0)
                    .attachments(target.0)
                    .into_iter()
                    .filter_map(|a| game.lookup_holo_member(a))
                    .map(|m| m.names()),
            );

        names
            .multi_cartesian_product()
            .any(|ns| ns.into_iter().all_equal())
    }

    pub fn can_use_ability(
        &self,
        card: CardRef,
        ability_idx: usize,
        game: &GameDirector,
        is_triggered: bool,
    ) -> bool {
        //  could prevent art by effect
        if game.has_modifier(card, PreventAbility(ability_idx)) {
            return false;
        }
        if game.has_modifier(card, PreventAbilities) {
            return false;
        }

        self.abilities[ability_idx]
            .condition
            .ctx()
            .with_card(card, &game.game)
            .with_triggered(is_triggered)
            .evaluate(&game.game)
    }

    pub fn can_use_art(
        &self,
        card: CardRef,
        art_idx: usize,
        target_card: CardRef,
        game: &GameDirector,
    ) -> bool {
        //  could prevent art by effect
        if game.has_modifier(card, PreventArt(art_idx)) {
            return false;
        }
        if game.has_modifier(card, PreventAllArts) {
            return false;
        }
        if game.has_modifier(card, Resting) {
            return false;
        }

        // need required attached cheers to attack
        if !game.required_attached_cheers(card, &self.arts[art_idx].cost) {
            return false;
        }

        self.arts[art_idx]
            .condition
            .ctx()
            .with_card(card, &game.game)
            .with_art_target(target_card)
            .evaluate(&game.game)
        // .evaluate_with_card(&game.game, card, false)
    }
}

#[derive(
    Encode,
    Decode,
    Serialize,
    Deserialize,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    GetSize,
)]
#[serde(rename_all = "snake_case")]
pub enum HoloMemberLevel {
    /// This holomem cannot Bloom.
    Spot,
    Debut,
    First,
    Second,
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, GetSize)]
pub enum HoloMemberHashTag {
    // generations
    JP,
    ID,
    EN,
    Gen0,
    Gen3,
    Gen4,
    Gen5,
    SecretSocietyholoX,
    IDGen1,
    IDGen3,
    Myth,
    Promise,
    // misc
    Alcohol,
    AnimalEars,
    Bird,
    Drawing,
    Song,
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, GetSize)]
#[serde(rename_all = "snake_case")]
pub enum HoloMemberExtraAttribute {
    /// When this holomem is Knocked Out, you lose 2 Life.
    Buzz,
    // This card is treated as [<name>].
    Name(String),
    /// You may include any number of this holomem in your deck.
    Unlimited,
    Unknown,
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
pub struct HoloMemberAbility {
    pub kind: MemberAbilityKind,
    pub name: String,
    pub text: String,
    #[serde(serialize_with = "serialize_conditions")]
    #[serde(deserialize_with = "deserialize_conditions")]
    #[serde(skip_serializing_if = "skip_default_conditions")]
    pub condition: CardEffectCondition,
    #[serde(serialize_with = "serialize_actions")]
    #[serde(deserialize_with = "deserialize_actions")]
    #[serde(skip_serializing_if = "skip_default_actions")]
    pub effect: CardEffect,
}

impl HoloMemberAbility {
    pub fn should_activate(&self, card: CardRef, triggered_event: &TriggeredEvent) -> bool {
        match self.kind {
            MemberAbilityKind::CollabEffect => {
                if let TriggeredEvent::After(Event::Collab(Collab {
                    card: collab_card, ..
                })) = triggered_event
                {
                    *collab_card == card
                } else {
                    false
                }
            }
            MemberAbilityKind::BloomEffect => {
                if let TriggeredEvent::After(Event::Bloom(Bloom {
                    from_card: bloom_card,
                    ..
                })) = triggered_event
                {
                    *bloom_card == card
                } else {
                    false
                }
            }
            MemberAbilityKind::Gift(_) => unimplemented!(), // TODO not sure what gift does
        }
    }
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, GetSize)]
#[serde(rename_all = "snake_case")]
pub enum MemberAbilityKind {
    CollabEffect,
    BloomEffect,
    Gift(CardEffectTrigger), // TODO verify if gift is correct. what kind of effect they have?
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
pub struct HoloMemberArt {
    pub name: String,
    pub cost: HoloMemberArtCost,
    pub damage: HoloMemberArtDamage,
    pub special_damage: Option<(Color, HoloMemberHp)>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub text: String,
    #[serde(serialize_with = "serialize_conditions")]
    #[serde(deserialize_with = "deserialize_conditions")]
    #[serde(skip_serializing_if = "skip_default_conditions")]
    pub condition: CardEffectCondition,
    #[serde(serialize_with = "serialize_actions")]
    #[serde(deserialize_with = "deserialize_actions")]
    #[serde(skip_serializing_if = "skip_default_actions")]
    pub effect: CardEffect,
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, GetSize)]
#[serde(rename_all = "snake_case")]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub enum HoloMemberArtDamage {
    Basic(HoloMemberHp),
    Plus(HoloMemberHp),
    Minus(HoloMemberHp),
    Multiple(HoloMemberHp),
    Uncertain,
}

impl TryFrom<String> for HoloMemberArtDamage {
    type Error = ParseIntError;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        if value == "?" {
            Ok(Self::Uncertain)
        } else if value.ends_with('+') {
            Ok(Self::Plus(value.trim_end_matches('+').parse()?))
        } else if value.ends_with('-') {
            Ok(Self::Minus(value.trim_end_matches('-').parse()?))
        } else if value.ends_with('x') {
            Ok(Self::Multiple(value.trim_end_matches('x').parse()?))
        } else {
            Ok(Self::Basic(value.parse()?))
        }
    }
}

impl From<HoloMemberArtDamage> for String {
    fn from(value: HoloMemberArtDamage) -> Self {
        match value {
            HoloMemberArtDamage::Basic(dmg) => format!("{dmg}"),
            HoloMemberArtDamage::Plus(dmg) => format!("{dmg}+"),
            HoloMemberArtDamage::Minus(dmg) => format!("{dmg}-"),
            HoloMemberArtDamage::Multiple(dmg) => format!("{dmg}x"),
            HoloMemberArtDamage::Uncertain => "?".into(),
        }
    }
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
pub struct SupportCard {
    pub card_number: CardNumber,
    pub name: String,
    pub kind: SupportKind,
    pub limited: bool, // TODO limited is needed, but not sure how
    pub text: String,
    pub effects: Vec<SupportEffect>,
    pub rarity: Rarity,
    pub illustration_url: IllustrationUrl,
    pub artist: String,
}

impl SupportCard {
    pub fn can_use_support(&self, card: CardRef, effect_idx: usize, game: &GameDirector) -> bool {
        if self.limited && game.has_modifier(card, PreventLimitedSupport) {
            return false;
        }

        self.effects[effect_idx]
            .condition
            .ctx()
            .with_card(card, &game.game)
            .evaluate(&game.game)
    }

    pub fn can_attach_target(
        &self,
        card: CardRef,
        effect_idx: usize,
        target: CardRef,
        game: &GameDirector,
    ) -> bool {
        if self.limited && game.has_modifier(card, PreventLimitedSupport) {
            return false;
        }

        self.effects[effect_idx]
            .condition
            .ctx()
            .with_card(card, &game.game)
            .with_attach_target(target)
            .evaluate(&game.game)
    }

    pub fn can_use_effect(
        &self,
        card: CardRef,
        effect_idx: usize,
        game: &GameDirector,
        is_triggered: bool,
    ) -> bool {
        //  could prevent abilities by effect
        if game.has_modifier(card, PreventAbilities) {
            return false;
        }

        self.effects[effect_idx]
            .condition
            .ctx()
            .with_card(card, &game.game)
            .with_triggered(is_triggered)
            .evaluate(&game.game)
    }
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, GetSize)]
#[serde(rename_all = "snake_case")]
pub enum SupportKind {
    Staff,
    Item,
    Event,
    Tool,
    Mascot,
    Fan,
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
pub struct SupportEffect {
    pub triggers: CardEffectTrigger,
    #[serde(serialize_with = "serialize_conditions")]
    #[serde(deserialize_with = "deserialize_conditions")]
    #[serde(skip_serializing_if = "skip_default_conditions")]
    pub condition: CardEffectCondition,
    #[serde(serialize_with = "serialize_actions")]
    #[serde(deserialize_with = "deserialize_actions")]
    #[serde(skip_serializing_if = "skip_default_actions")]
    pub effect: CardEffect,
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
pub struct CheerCard {
    pub card_number: CardNumber,
    pub name: String,
    pub color: Color,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub text: String,
    pub rarity: Rarity,
    pub illustration_url: IllustrationUrl,
    pub artist: String,
}
