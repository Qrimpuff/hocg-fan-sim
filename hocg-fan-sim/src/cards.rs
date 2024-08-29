use std::collections::HashMap;

use evaluate::EvaluateEffect;
use get_size::GetSize;
use iter_tools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::card_effects::{
    effects::{
        deserialize_actions, deserialize_conditions, serialize_actions, serialize_conditions,
    },
    *,
};
use crate::events::{Bloom, Collab, Event, EventKind, TriggeredEvent};
use crate::gameplay::Zone;
use crate::gameplay::{CardRef, Game};
use crate::modifiers::ModifierKind::*;

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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Set {
    number: String,
    name: String,
    // maybe preset decks
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loadout {
    pub oshi: CardNumber,
    pub main_deck: Vec<CardNumber>,
    pub cheer_deck: Vec<CardNumber>,
    // cosmetic...
}

#[derive(Debug, Clone, Default, GetSize)]
pub struct GlobalLibrary {
    // TODO use a different key because rarity is not include in card number
    pub cards: HashMap<CardNumber, Card>,
}

impl GlobalLibrary {
    /// Any pre-processing of cards that could make my life easier later
    pub fn pre_process(&mut self) {
        // not sure if these are good ideas. might be better to be explicit
        // TODO oshi skill once turn
        // TODO special oshi skill once per game
        // TODO enough holo power to pay the cost for oshi skill
        // TODO enough cheers to perform art for members
        // TODO limited support
        // TODO if you can't select something, it should check that it's there first in condition

        // DON'T REMOVE YET. NOT BEFORE THE FILES ARE MADE
        // default condition to always
        let default_trigger = Trigger::ActivateInMainStep;
        let default_condition = Condition::True;
        let default_action = Action::Noop;
        // let default_damage_mod = DamageModifier::None;
        for card in self.cards.values_mut() {
            match card {
                Card::OshiHoloMember(o) => o.skills.iter_mut().for_each(|s| {
                    if s.triggers.is_empty() {
                        s.triggers.push(default_trigger)
                    }
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
                        if a.effect.is_empty() {
                            a.effect.push(default_action.clone())
                        }
                    })
                }
                Card::Support(s) => {
                    if s.attachment_condition.is_empty() {
                        s.attachment_condition.push(default_condition.clone())
                    }
                    if s.triggers.is_empty() {
                        s.triggers.push(default_trigger)
                    }
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
        // end of: DON'T REMOVE YET. NOT BEFORE THE FILES ARE MADE

        // verify effect serialization consistency (de -> ser -> de)
        fn serialization_round_trip<T>(effect: T) -> crate::card_effects::Result<()>
        where
            T: SerializeEffect + ParseTokens + PartialEq + Clone,
        {
            let string = effect.clone().serialize_effect();
            let de_effect = string.parse_effect::<T>()?;

            if effect != de_effect && Some(de_effect) != T::default_effect() {
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
                        error!("{}: {} - condition - {}", o.card_number, s.name, e);
                        has_errors = true;
                    }
                    if let Err(e) = serialization_round_trip(s.effect.clone()) {
                        error!("{}: {} - effect - {}", o.card_number, s.name, e);
                        has_errors = true;
                    }
                }),
                Card::HoloMember(m) => {
                    m.abilities.iter_mut().for_each(|a| {
                        if let Err(e) = serialization_round_trip(a.condition.clone()) {
                            error!("{}: {} - condition - {}", m.card_number, a.name, e);
                            has_errors = true;
                        }
                        if let Err(e) = serialization_round_trip(a.effect.clone()) {
                            error!("{}: {} - effect - {}", m.card_number, a.name, e);
                            has_errors = true;
                        }
                    });
                    m.arts.iter_mut().for_each(|a| {
                        if let Err(e) = serialization_round_trip(a.condition.clone()) {
                            error!("{}: {} - condition - {}", m.card_number, a.name, e);
                            has_errors = true;
                        }
                        if let Err(e) = serialization_round_trip(a.effect.clone()) {
                            error!("{}: {} - effect - {}", m.card_number, a.name, e);
                            has_errors = true;
                        }
                    })
                }
                Card::Support(s) => {
                    if let Err(e) = serialization_round_trip(s.attachment_condition.clone()) {
                        error!(
                            "{}: {} - attachment_condition - {}",
                            s.card_number, s.name, e
                        );
                        has_errors = true;
                    }
                    if let Err(e) = serialization_round_trip(s.condition.clone()) {
                        error!("{}: {} - condition - {}", s.card_number, s.name, e);
                        has_errors = true;
                    }
                    if let Err(e) = serialization_round_trip(s.effect.clone()) {
                        error!("{}: {} - effect - {}", s.card_number, s.name, e);
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

#[derive(Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
pub enum Card {
    OshiHoloMember(OshiHoloMemberCard),
    HoloMember(HoloMemberCard),
    Support(SupportCard),
    Cheer(CheerCard),
}

impl Card {
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

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, GetSize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, GetSize)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum Color {
    White,
    Green,
    Red,
    Blue,
    Purple,
    Yellow,
    ColorLess,
}

pub type CardNumber = String;
pub type IllustrationPath = String;
pub type OshiLife = u8;
pub type HoloMemberHp = u16;
pub type OshiSkillCost = u8;
pub type HoloMemberArtCost = Vec<Color>;
pub type CardEffectTrigger = Vec<Trigger>;
pub type CardEffectCondition = Vec<Condition>;
pub type CardEffect = Vec<Action>;
pub type HoloMemberBatonPassCost = u8;

#[derive(Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
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

impl OshiHoloMemberCard {
    pub fn can_use_skill(
        &self,
        card: CardRef,
        skill_idx: usize,
        game: &Game,
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
            .evaluate_with_card(&game.state, card, is_triggered)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
pub struct OshiSkill {
    pub kind: OshiSkillKind,
    pub name: String,
    pub cost: OshiSkillCost,
    pub text: String,
    pub triggers: CardEffectTrigger,
    #[serde(serialize_with = "serialize_conditions")]
    #[serde(deserialize_with = "deserialize_conditions")]
    pub condition: CardEffectCondition,
    #[serde(serialize_with = "serialize_actions")]
    #[serde(deserialize_with = "deserialize_actions")]
    pub effect: CardEffect,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, GetSize)]
#[serde(rename_all = "snake_case")]
pub enum OshiSkillKind {
    Normal,
    Special,
}

#[derive(Serialize, Deserialize, Debug, Clone, GetSize)]
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
    pub extra: Option<String>, // TODO will probably be an enum
    pub attributes: Vec<HoloMemberExtraAttribute>,
    pub rarity: Rarity,
    pub illustration: IllustrationPath,
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

    pub fn can_baton_pass(&self, card: CardRef, game: &Game) -> bool {
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
        let cost = std::iter::repeat(Color::ColorLess)
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
        game: &Game,
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
        game: &Game,
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
            .evaluate_with_card(&game.state, card, is_triggered)
    }

    pub fn can_use_art(&self, card: CardRef, art_idx: usize, game: &Game) -> bool {
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
            .evaluate_with_card(&game.state, card, false)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, GetSize)]
#[serde(rename_all = "snake_case")]
pub enum HoloMemberLevel {
    Spot,
    Debut,
    First,
    Second,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, GetSize)]
#[serde(rename_all = "snake_case")]
pub enum HoloMemberHashTag {
    JP,
    ID,
    EN,
    Gen0,
    Gen4,
    SecretSocietyholoX,
    IDGen1,
    Promise,
    Song,
    Drawing,
    AnimalEars,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, GetSize)]
#[serde(rename_all = "snake_case")]
pub enum HoloMemberExtraAttribute {
    Buzz,
    Name(String),
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
pub struct HoloMemberAbility {
    pub kind: MemberAbilityKind,
    pub name: String,
    pub text: String,
    #[serde(serialize_with = "serialize_conditions")]
    #[serde(deserialize_with = "deserialize_conditions")]
    pub condition: CardEffectCondition,
    #[serde(serialize_with = "serialize_actions")]
    #[serde(deserialize_with = "deserialize_actions")]
    pub effect: CardEffect,
}

impl HoloMemberAbility {
    pub fn should_activate(&self, card: CardRef, triggered_event: &TriggeredEvent) -> bool {
        match self.kind {
            MemberAbilityKind::CollabEffect => {
                if let TriggeredEvent::After(Event {
                    kind:
                        EventKind::Collab(Collab {
                            card: collab_card, ..
                        }),
                    ..
                }) = triggered_event
                {
                    collab_card.1 == card
                } else {
                    false
                }
            }
            MemberAbilityKind::BloomEffect => {
                if let TriggeredEvent::After(Event {
                    kind:
                        EventKind::Bloom(Bloom {
                            from_card: bloom_card,
                            ..
                        }),
                    ..
                }) = triggered_event
                {
                    bloom_card.1 == card
                } else {
                    false
                }
            }
            MemberAbilityKind::Gift(_) => unimplemented!(), // TODO not sure what gift does
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, GetSize)]
#[serde(rename_all = "snake_case")]
pub enum MemberAbilityKind {
    CollabEffect,
    BloomEffect,
    Gift(CardEffectTrigger), // TODO verify if gift is correct. what kind of effect they have?
}

#[derive(Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
pub struct HoloMemberArt {
    pub name: String,
    pub cost: HoloMemberArtCost,
    pub damage: HoloMemberArtDamage,
    pub special_damage: Option<(Color, HoloMemberHp)>,
    pub text: String,
    #[serde(serialize_with = "serialize_conditions")]
    #[serde(deserialize_with = "deserialize_conditions")]
    pub condition: CardEffectCondition,
    #[serde(serialize_with = "serialize_actions")]
    #[serde(deserialize_with = "deserialize_actions")]
    pub effect: CardEffect,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, GetSize)]
#[serde(rename_all = "snake_case")]
pub enum HoloMemberArtDamage {
    Basic(HoloMemberHp),
    Plus(HoloMemberHp),
    Minus(HoloMemberHp),
    Uncertain,
}

#[derive(Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
pub struct SupportCard {
    pub card_number: CardNumber,
    pub name: String,
    pub kind: SupportKind,
    pub limited: bool, // TODO limited is needed, but not sure how
    pub text: String,
    #[serde(serialize_with = "serialize_conditions")]
    #[serde(deserialize_with = "deserialize_conditions")]
    pub attachment_condition: CardEffectCondition, // used by Fan
    pub triggers: CardEffectTrigger,
    #[serde(serialize_with = "serialize_conditions")]
    #[serde(deserialize_with = "deserialize_conditions")]
    pub condition: CardEffectCondition,
    #[serde(serialize_with = "serialize_actions")]
    #[serde(deserialize_with = "deserialize_actions")]
    pub effect: CardEffect,
    pub rarity: Rarity,
    pub illustration: IllustrationPath,
    pub artist: String,
}

impl SupportCard {
    pub fn can_use_support(&self, card: CardRef, game: &Game) -> bool {
        if self.limited && game.has_modifier(card, PreventLimitedSupport) {
            return false;
        }

        self.condition.evaluate_with_card(&game.state, card, false)
    }

    pub fn can_attach_target(
        &self,
        _card: CardRef,
        _game: &Game,
        _target: (CardRef, &HoloMemberCard),
    ) -> bool {
        // TODO fan can be attached. how to send the target card in the condition?
        unimplemented!()
    }

    pub fn can_use_ability(&self, card: CardRef, game: &Game, is_triggered: bool) -> bool {
        //  could prevent art by effect
        if game.has_modifier(card, PreventAbilities) {
            return false;
        }

        self.condition
            .evaluate_with_card(&game.state, card, is_triggered)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, GetSize)]
#[serde(rename_all = "snake_case")]
pub enum SupportKind {
    Staff,
    Item,
    Event,
    Fan,
}

#[derive(Serialize, Deserialize, Debug, Clone, GetSize)]
#[serde(rename_all = "snake_case")]
pub struct CheerCard {
    pub card_number: CardNumber,
    pub name: String,
    pub color: Color,
    pub text: String,
    pub rarity: Rarity,
    pub illustration: IllustrationPath,
    pub artist: String,
}
