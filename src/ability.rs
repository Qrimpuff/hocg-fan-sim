mod de;
mod error;
mod ser;

pub use de::{from_str, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_string, Serializer};
use serde::{Deserialize, Serialize};

/* examples:
buff active_holo more_def 20 next_turn
heal 10
buff self more_def 30 _

to think about:
debuff less_atk mul 10 get_dmg_count this_turn
debuff _ less_atk mul 10 get_dmg_count _ this_turn
debuff _ less_atk mul 10 get dmg_count _ this_turn
debuff_t _ less_atk (mul 10 get_t dmg_count _) this_turn
debuff less_atk (mul 10 get dmg_count) this_attack

for self (debuff less_atk (mul 10 get dmg_count) this_attack)
for def_holo (debuff less_atk (mul 10 (for self get dmg_count)) this_attack)

for traget discard_all_cheer 
*/

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BuiltIn {
    #[serde(rename = "active_holo")]
    ActiveHoloMember,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(try_from = "String")]
pub struct Var(String);

impl TryFrom<String> for Var {
    type Error = error::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        if value.starts_with("$") {
            Ok(Self(value))
        } else {
            Err(Self::Error::Message("not a var".into()))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum Target {
    BuiltIn(BuiltIn),
    Var(Var),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Buff(Target, Buff, LifeTime),
    Debuff(Target, Debuff, LifeTime),
    Heal(Target, Value),
    Let(Var, Value),
    When(Condition, Box<Action>),
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Value {
    Get(Target, Property),
    #[serde(untagged)]
    Number(u32),
    #[serde(untagged)]
    Var(Var),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    Equals(Value, Value),
    Has(Target, Tag),
    NotEquals(Value, Value),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Tag {
    // colors
    ColorWhite,
    ColorGreen,
    ColorBlue,
    ColorRed,
    ColorPurple,
    ColorYellow,
    // stages
    StageDebut,
    StageFirst,
    StageSecond,
    //abilities
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Property {
    HealtPoint,
    RetreatCost,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Buff {
    #[serde(rename = "more_def")]
    MoreDefence(Value),
    #[serde(rename = "more_atk")]
    MoreAttack(Value),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Debuff {}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LifeTime {
    ThisTurn,
    NextTurn,
    Limitless,
}
