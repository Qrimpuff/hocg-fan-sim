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

for target discard_all_cheer
*/

use crate::{gameplay::Step, HoloMemberHp};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuiltIn {
    CurrentCard,
    CenterHoloMember,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Var(pub String);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Number(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    BuiltIn(BuiltIn),
    Var(Var),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Noop,
    For(Target, Box<Action>),
    Buff(Buff, LifeTime),
    Debuff(Debuff, LifeTime),
    Heal(Value),
    Let(Var, Value),
    When(Condition, Box<Action>),
    Draw(Value),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    For(Target, Box<Value>),
    Get(Property),
    Number(Number),
    Var(Var),
    Add(Box<Value>, Box<Value>),
    Subtract(Box<Value>, Box<Value>),
    Multiply(Box<Value>, Box<Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    Always,
    OncePerTurn,
    Equals(Value, Value),
    Has(Target, Tag),
    NotEquals(Value, Value),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Trigger {
    AtStartOfTurn,
    AtEndOfTurn,
    AtStartOfStep(Step),
    AtEndOfStep(Step),
    AtStartOfPerformArt,
    AtEndOfPerformArt,
    OnBeforeDiceRoll,
    OnAfterDiceRoll,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageModifier {
    Plus(HoloMemberHp),
    Minus(HoloMemberHp),
    Times(HoloMemberHp),
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Property {
    HealthPoint,
    RetreatCost,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Buff {
    MoreDefense(Value),
    MoreAttack(Value),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Debuff {
    LessDefense(Value),
    LessAttack(Value),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LifeTime {
    ThisAttack,
    ThisTurn,
    NextTurn,
    Limitless,
}
