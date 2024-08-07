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

use std::collections::VecDeque;

use hololive_ucg_poc_derive::HoloUcgEffect;
use iter_tools::Itertools;

use crate::{
    events::{EnterStep, Event, ExitStep, TriggeredEvent},
    gameplay::Step,
    Error, ParseTokens, Result, Tokens,
};

// TODO clean up this file after the list of effect is finalized

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Var(pub String);

impl From<Var> for Tokens {
    fn from(value: Var) -> Self {
        value.0.as_str().into()
    }
}

impl ParseTokens for Var {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        let s = Self::take_string(tokens)?;
        if s.starts_with('$') {
            Ok(Var(s))
        } else {
            Self::return_string(tokens, s.clone());
            Err(Error::UnexpectedToken("var".into(), s))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Number(pub u32);

impl From<Number> for Tokens {
    fn from(value: Number) -> Self {
        value.0.to_string().as_str().into()
    }
}

impl ParseTokens for Number {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        let s = Self::take_string(tokens)?;
        if let Ok(n) = s.parse() {
            Ok(Number(n))
        } else {
            Self::return_string(tokens, s.clone());
            Err(Error::UnexpectedToken("number".into(), s))
        }
    }
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum Target {
    #[holo_ucg(token = "self")]
    CurrentCard,
    #[holo_ucg(token = "center_member")]
    CenterHoloMember,
    #[holo_ucg(token = "select_member")]
    SelectMember(Box<Target>),
    #[holo_ucg(token = "select_cheers_up_to")]
    SelectCheersUpTo(Box<Value>, Box<Target>),
    #[holo_ucg(token = "members_on_stage")]
    MembersOnStage,
    #[holo_ucg(token = "cheers_in_archive")]
    CheersInArchive,
    #[holo_ucg(token = "_with", infix = "with")]
    With(Box<Target>, Tag),
    #[holo_ucg(transparent)]
    Var(Var),
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum Action {
    #[holo_ucg(token = "no_action")]
    Noop,
    #[holo_ucg(token = "for")]
    For(Target, Box<Action>),
    #[holo_ucg(token = "buff")]
    Buff(Buff, LifeTime),
    #[holo_ucg(token = "debuff")]
    Debuff(Debuff, LifeTime),
    #[holo_ucg(token = "heal")]
    Heal(Value),
    #[holo_ucg(token = "let")]
    Let(Var, Value),
    #[holo_ucg(token = "when")]
    When(Condition, Box<Action>),
    #[holo_ucg(token = "draw")]
    Draw(Value),
    #[holo_ucg(token = "next_dice_number")]
    NextDiceNumber(Value),
    #[holo_ucg(token = "attach")]
    Attach(Target),
}

pub fn serialize_actions(actions: Vec<Action>) -> String {
    actions
        .into_iter()
        .map(|a| {
            let s = Tokens::from(a).to_string();
            let mut chars = s.chars();
            chars.next();
            chars.next_back();
            chars.as_str().to_owned()
        })
        .collect_vec()
        .join("\n")
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum Value {
    #[holo_ucg(token = "for")]
    For(Target, Box<Value>),
    #[holo_ucg(token = "get")]
    Get(Property),
    #[holo_ucg(transparent)]
    Number(Number),
    #[holo_ucg(transparent)]
    Var(Var),
    #[holo_ucg(token = "add", infix = "+")]
    Add(Box<Value>, Box<Value>),
    #[holo_ucg(token = "sub", infix = "-")]
    Subtract(Box<Value>, Box<Value>),
    #[holo_ucg(token = "mul", infix = "*")]
    Multiply(Box<Value>, Box<Value>),
    #[holo_ucg(token = "select_dice_number")]
    SelectDiceNumber,
    #[holo_ucg(token = "all")]
    All,
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    #[holo_ucg(token = "always")]
    Always,
    #[holo_ucg(token = "eq", infix = "==")]
    Equals(Value, Value),
    #[holo_ucg(token = "_has", infix = "has")]
    Has(Target, Tag),
    #[holo_ucg(token = "neq", infix = "!=")]
    NotEquals(Value, Value),
    #[holo_ucg(token = "_and", infix = "and")]
    And(Box<Condition>, Box<Condition>),
    #[holo_ucg(token = "_or", infix = "or")]
    Or(Box<Condition>, Box<Condition>),

    #[holo_ucg(token = "once_per_turn")]
    OncePerTurn,
    #[holo_ucg(token = "once_per_game")]
    OncePerGame,
    #[holo_ucg(token = "is_holo_member")]
    IsHoloMember,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    ActivateInMainStep,
    OnStartTurn,
    OnEndTurn,
    OnEnterStep(Step),
    OnExitStep(Step),
    OnBeforePerformArt,
    OnAfterPerformArt,
    OnBeforeRollDice,
    OnAfterRollDice,
}

impl Trigger {
    pub fn should_activate(&self, triggered_event: &TriggeredEvent) -> bool {
        match self {
            Trigger::ActivateInMainStep => false,
            Trigger::OnStartTurn => {
                matches!(triggered_event, TriggeredEvent::After(Event::StartTurn(_)))
            }
            Trigger::OnEndTurn => {
                matches!(triggered_event, TriggeredEvent::After(Event::EndTurn(_)))
            }
            Trigger::OnEnterStep(step) => {
                if let TriggeredEvent::After(Event::EnterStep(EnterStep { active_step, .. })) =
                    triggered_event
                {
                    active_step == step
                } else {
                    false
                }
            }
            Trigger::OnExitStep(step) => {
                if let TriggeredEvent::After(Event::ExitStep(ExitStep { active_step, .. })) =
                    triggered_event
                {
                    active_step == step
                } else {
                    false
                }
            }
            Trigger::OnBeforePerformArt => {
                matches!(
                    triggered_event,
                    TriggeredEvent::Before(Event::PerformArt(_))
                )
            }
            Trigger::OnAfterPerformArt => {
                matches!(triggered_event, TriggeredEvent::After(Event::PerformArt(_)))
            }
            Trigger::OnBeforeRollDice => {
                matches!(triggered_event, TriggeredEvent::Before(Event::RollDice(_)))
            }
            Trigger::OnAfterRollDice => {
                matches!(triggered_event, TriggeredEvent::After(Event::RollDice(_)))
            }
        }
    }
}

#[derive(HoloUcgEffect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageModifier {
    #[holo_ucg(token = "none")]
    None,
    #[holo_ucg(token = "plus")]
    Plus(Number),
    #[holo_ucg(token = "minus")]
    Minus(Number),
    #[holo_ucg(token = "times")]
    Times(Number),
}

#[derive(HoloUcgEffect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    // colors
    #[holo_ucg(token = "color_white")]
    ColorWhite,
    #[holo_ucg(token = "color_green")]
    ColorGreen,
    #[holo_ucg(token = "color_blue")]
    ColorBlue,
    #[holo_ucg(token = "color_red")]
    ColorRed,
    #[holo_ucg(token = "color_purple")]
    ColorPurple,
    #[holo_ucg(token = "color_yellow")]
    ColorYellow,
    // stages
    #[holo_ucg(token = "level_debut")]
    LevelDebut,
    #[holo_ucg(token = "level_first")]
    LevelFirst,
    #[holo_ucg(token = "level_second")]
    LevelSecond,
    //abilities
}

#[derive(HoloUcgEffect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Property {
    #[holo_ucg(token = "hp")]
    HealthPoint,
    #[holo_ucg(token = "r_cost")]
    RetreatCost,
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum Buff {
    #[holo_ucg(token = "more_def")]
    MoreDefense(Value),
    #[holo_ucg(token = "more_atk")]
    MoreAttack(Value),
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum Debuff {
    #[holo_ucg(token = "less_def")]
    LessDefense(Value),
    #[holo_ucg(token = "less_atk")]
    LessAttack(Value),
}

#[derive(HoloUcgEffect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifeTime {
    #[holo_ucg(token = "this_attack")]
    ThisAttack,
    #[holo_ucg(token = "this_turn")]
    ThisTurn,
    #[holo_ucg(token = "next_turn")]
    NextTurn,
    #[holo_ucg(token = "_")]
    Limitless,
}
