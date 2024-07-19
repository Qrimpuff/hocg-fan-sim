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

// TODO clean up this file after the list of effect is finalized

#[derive(Debug, Clone, PartialEq)]
pub struct Var(pub String);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Number(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    CurrentCard,
    CenterHoloMember,
    SelectMember(Box<Target>),
    SelectCheersUpTo(Box<Value>, Box<Target>),
    MembersOnStage,
    CheersInArchive,
    With(Box<Target>, Tag),
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
    NextDiceNumber(Value),
    Attach(Target),
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
    SelectDiceNumber,
    All,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    Always,
    Equals(Value, Value),
    Has(Target, Tag),
    NotEquals(Value, Value),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),

    OncePerTurn,
    OncePerGame,
    IsHoloMember,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Trigger {
    ActivateInMainStep,
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
    None,
    Plus(Number),
    Minus(Number),
    Times(Number),
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
    LevelDebut,
    LevelFirst,
    LevelSecond,
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
