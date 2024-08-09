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

use std::fmt::Debug;

use hololive_ucg_poc_derive::HoloUcgEffect;
use iter_tools::Itertools;

use crate::{
    events::{EnterStep, Event, ExitStep, TriggeredEvent},
    gameplay::Step,
    Error, ParseTokens, Result, Tokens,
};

use super::{TakeParam, TakeString};

// TODO clean up this file after the list of effect is finalized

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Var(pub String);

impl From<Var> for Tokens {
    fn from(value: Var) -> Self {
        value.0.as_str().into()
    }
}

impl ParseTokens for Var {
    fn parse_tokens(tokens: &[Tokens]) -> Result<(Self, &[Tokens])> {
        if let Ok((s, t)) = tokens.take_string() {
            if s.starts_with('$') {
                return Ok((Var(s.into()), t));
            }
        }
        Err(Error::UnexpectedToken(
            "Var".into(),
            tokens.take_string()?.0.clone(),
        ))
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
    fn parse_tokens(tokens: &[Tokens]) -> Result<(Self, &[Tokens])> {
        if let Ok((s, t)) = tokens.take_string() {
            if let Ok(n) = s.parse() {
                return Ok((Number(n), t));
            }
        }
        Err(Error::UnexpectedToken(
            "Number".into(),
            tokens.take_string()?.0.clone(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// let <var> = <value> | <player> | <target>
pub struct Let<T>(pub Var, pub T);

impl<T> From<Let<T>> for Tokens
where
    Tokens: std::convert::From<T>,
{
    fn from(value: Let<T>) -> Self {
        ["let".into(), value.0.into(), "=".into(), value.1.into()].into()
    }
}

impl<T: ParseTokens + Debug> ParseTokens for Let<T> {
    fn parse_tokens(tokens: &[Tokens]) -> Result<(Self, &[Tokens])> {
        if let Ok((s, t)) = tokens.take_string() {
            if s == "let" {
                if let Ok((v1, t)) = t.take_param() {
                    if let Ok((s, t)) = t.take_string() {
                        if s == "=" {
                            if let Ok((v2, t)) = t.take_param() {
                                return Ok((Let(v1, v2), t));
                            }
                        } else {
                            return Err(Error::UnexpectedToken("Let".into(), s.clone()));
                        }
                    }
                }
            }
        }
        Err(Error::UnexpectedToken(
            "Let".into(),
            tokens.take_string()?.0.clone(),
        ))
    }
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum TargetPlayer {
    #[holo_ucg(token = "self")]
    CurrentPlayer,
    #[holo_ucg(token = "opponent")]
    Opponent,
    #[holo_ucg(transparent)]
    Var(Var),
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum TargetPlayers {
    // one player
    #[holo_ucg(token = "self")]
    CurrentPlayer,
    #[holo_ucg(token = "opponent")]
    Opponent,
    // many players
    #[holo_ucg(token = "all_players")]
    AllPlayers,

    #[holo_ucg(infix = "with")]
    With(Box<TargetPlayers>, Attribute), // TODO not sure about tag, should be condition (player)
    #[holo_ucg(transparent)]
    Var(Var),
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum TargetCard {
    #[holo_ucg(token = "self")]
    CurrentCard,
    #[holo_ucg(token = "oshi")]
    Oshi,
    #[holo_ucg(token = "center_member")]
    CenterHoloMember,
    #[holo_ucg(token = "collab_member")]
    CollabHoloMember,
    #[holo_ucg(token = "select_member")]
    SelectMember(Box<TargetCards>),

    #[holo_ucg(transparent)]
    Var(Var),
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum TargetCards {
    // one card
    #[holo_ucg(token = "self")]
    CurrentCard,
    #[holo_ucg(token = "oshi")]
    Oshi,
    #[holo_ucg(token = "center_member")]
    CenterHoloMember,
    #[holo_ucg(token = "collab_member")]
    CollabHoloMember,
    #[holo_ucg(token = "select_member")]
    SelectMember(Box<TargetCards>),
    // many cards
    #[holo_ucg(token = "main_stage_members")]
    MainStageMembers,
    #[holo_ucg(token = "back_stage_members")]
    BackStageMembers,
    #[holo_ucg(token = "stage_members")]
    StageMembers,
    #[holo_ucg(token = "attached_cheers")]
    AttachedCheers,

    #[holo_ucg(token = "select_cheers_up_to")]
    SelectCheersUpTo(Box<Value>, Box<TargetCards>),
    #[holo_ucg(token = "cheers_in_archive")]
    CheersInArchive,
    #[holo_ucg(infix = "with")]
    With(Box<TargetCards>, Attribute),
    #[holo_ucg(transparent)]
    Var(Var),
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum Action {
    #[holo_ucg(token = "no_action")]
    Noop,
    #[holo_ucg(token = "for")]
    For(TargetCards, Box<Action>),
    #[holo_ucg(token = "buff")]
    Buff(Buff, LifeTime),
    #[holo_ucg(token = "debuff")]
    Debuff(Debuff, LifeTime),
    #[holo_ucg(token = "heal")]
    Heal(Value),
    #[holo_ucg(transparent)]
    LetValue(Let<Value>),
    #[holo_ucg(transparent)]
    LetTargetCard(Let<TargetCard>),
    #[holo_ucg(token = "when")]
    When(Condition, Box<Action>),
    #[holo_ucg(token = "draw")]
    Draw(Value),
    #[holo_ucg(token = "next_dice_number")]
    NextDiceNumber(Value),
    #[holo_ucg(token = "attach")]
    Attach(TargetCards),
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
    For(TargetCards, Box<Value>),
    #[holo_ucg(token = "get")]
    Get(Property),
    #[holo_ucg(transparent)]
    Number(Number),
    #[holo_ucg(transparent)]
    Var(Var),
    #[holo_ucg(infix = "+")]
    Add(Box<Value>, Box<Value>),
    #[holo_ucg(infix = "-")]
    Subtract(Box<Value>, Box<Value>),
    #[holo_ucg(infix = "*")]
    Multiply(Box<Value>, Box<Value>),
    #[holo_ucg(token = "select_dice_number")]
    SelectDiceNumber,
    #[holo_ucg(token = "roll_dice")]
    RollDice,
    #[holo_ucg(token = "all")]
    All,
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    #[holo_ucg(token = "always")]
    Always,
    #[holo_ucg(token = "true")]
    True,
    #[holo_ucg(token = "false")]
    False,
    #[holo_ucg(infix = "==")]
    Equals(Value, Value),
    #[holo_ucg(infix = "has")]
    Has(TargetCard, Attribute),
    #[holo_ucg(infix = "have")]
    Have(TargetCards, Attribute),
    #[holo_ucg(infix = "!=")]
    NotEquals(Value, Value),
    #[holo_ucg(infix = "and")]
    And(Box<Condition>, Box<Condition>),
    #[holo_ucg(infix = "or")]
    Or(Box<Condition>, Box<Condition>),

    #[holo_ucg(token = "is_odd")]
    IsOdd(Value),
    #[holo_ucg(token = "is_even")]
    IsEven(Value),

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
pub enum Attribute {
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
    // names
    #[holo_ucg(token = "name_azki")]
    NameAzki,
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
    #[holo_ucg(token = "more_dmg")]
    MoreDamage(Value),
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum Debuff {
    #[holo_ucg(token = "less_def")]
    LessDefense(Value),
    #[holo_ucg(token = "less_atk")]
    LessAttack(Value),
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum LifeTime {
    #[holo_ucg(token = "this_game")]
    ThisGame,
    #[holo_ucg(token = "this_turn")]
    ThisTurn,
    #[holo_ucg(token = "next_turn")]
    NextTurn(TargetPlayer),
    #[holo_ucg(token = "this_step")]
    ThisStep,
    #[holo_ucg(token = "this_art")]
    ThisArt,
    #[holo_ucg(token = "this_effect")]
    ThisEffect,
    #[holo_ucg(token = "until_removed")]
    UntilRemoved,
}
