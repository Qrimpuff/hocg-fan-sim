/* examples:

    hSD01-001 - Tokino Sora
        let $back_mem = select_one back_stage_members opponent's
        let $center_mem = center_members opponent's
        send_to opponent_back_stage $center_mem
        send_to opponent_center_stage $back_mem

    hSD01-011 - AZKi
        roll_dice
        if is_odd $_dice_value (
            add_mod this_card more_dmg 50 this_art
        )
        if $_dice_value == 1 (
            add_mod this_card more_dmg 50 this_art
        )

    hSD01-015 - Hakui Koyori
        let $center_mem = from_zone center_stage where is_member
        if $center_mem any name_tokino_sora (
            draw 1
        )
        if $center_mem any name_azki (
            let $cheer = from_zone_top 1 cheer_deck
            reveal $cheer
            attach_cards $cheer $center_mem
        )
*/

use std::fmt::Debug;

use hocg_fan_sim_derive::HocgFanSimCardEffect;
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
pub struct Number(pub usize);

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

/////////////////////////////////////

#[derive(HocgFanSimCardEffect, Debug, Clone, PartialEq, Eq)]
pub enum Action {
    // add_global_mod <mod> <life_time> -> action
    #[hocg_fan_sim(token = "add_global_mod")]
    AddGlobalModifier(Player, Modifier, LifeTime),
    // add_mod <[card_ref]> <mod> <life_time> -> <action>
    #[hocg_fan_sim(token = "add_mod")]
    AddModifier(CardReferences, Modifier, LifeTime),
    // add_zone_mod <zone> <mod> <life_time> -> action
    #[hocg_fan_sim(token = "add_zone_mod")]
    AddZoneModifier(Zone, Modifier, LifeTime),
    // attach_cards <[card_ref]> <card_ref> -> <action>
    #[hocg_fan_sim(token = "attach_cards")]
    AttachCards(CardReferences, CardReference),
    // draw <value> -> <action>
    #[hocg_fan_sim(token = "draw")]
    Draw(Value),
    // if <condition> <[action]> -> <action>
    #[hocg_fan_sim(token = "if")]
    If(Condition, Vec<Action>),
    // let <$var> = <[card_ref]> -> <action>
    #[hocg_fan_sim(transparent)]
    LetCardReferences(Let<CardReferences>),
    // let <$var> = <condition> -> <action>
    #[hocg_fan_sim(transparent)]
    LetCondition(Let<Condition>),
    // let <$var> = <select> -> <action>
    #[hocg_fan_sim(transparent)]
    LetSelect(Let<LetValue>),
    // let <$var> = <value> -> <action>
    #[hocg_fan_sim(transparent)]
    LetValue(Let<Value>),
    // no_action -> <action>
    #[hocg_fan_sim(token = "no_action")]
    Noop,
    // reveal <[card_ref]> -> <action>
    #[hocg_fan_sim(token = "reveal")]
    Reveal(CardReferences),
    // send_to <zone> <[card_ref]> -> <action>
    #[hocg_fan_sim(token = "send_to")]
    SendTo(Zone, CardReferences),
    // send_to_bottom <zone> <[card_ref]> -> <action>
    #[hocg_fan_sim(token = "send_to_bottom")]
    SendToBottom(Zone, CardReferences),
    // send_to_top <zone> <[card_ref]> -> <action>
    #[hocg_fan_sim(token = "send_to_top")]
    SendToTop(Zone, CardReferences),
    // shuffle <zone> -> <action>
    #[hocg_fan_sim(token = "shuffle")]
    Shuffle(Zone),
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

#[derive(HocgFanSimCardEffect, Debug, Clone, PartialEq, Eq)]
pub enum CardReference {
    // TODO figure out the conversion with CardReferences, could panic
    // this_card -> <card_ref>
    #[hocg_fan_sim(token = "this_card")]
    ThisCard,
    // <$var> -> <card_ref>
    #[hocg_fan_sim(transparent)]
    Var(Var),
}

#[derive(HocgFanSimCardEffect, Debug, Clone, PartialEq, Eq)]
pub enum CardReferences {
    // attached <card_ref> -> <[card_ref]>
    #[hocg_fan_sim(token = "attached")]
    Attached(CardReference),
    // from <zone> -> <[card_ref]>
    #[hocg_fan_sim(token = "from")]
    From(Zone),
    // from_top <value> <zone> -> <[card_ref]>
    #[hocg_fan_sim(token = "from_top")]
    FromTop(Box<Value>, Zone),
    // this_card -> <[card_ref]>
    #[hocg_fan_sim(token = "this_card")]
    ThisCard,
    // <$var> -> <[card_ref]>
    #[hocg_fan_sim(transparent)]
    Var(Var),
    // filter <[card_ref]> <condition> -> <[card_ref]>
    #[hocg_fan_sim(token = "filter")]
    Filter(Box<CardReferences>, Box<Condition>),
}

impl From<bool> for Condition {
    fn from(value: bool) -> Self {
        match value {
            true => Condition::True,
            false => Condition::False,
        }
    }
}

#[derive(HocgFanSimCardEffect, Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    // all <[card_ref]> <condition> -> <condition>
    #[hocg_fan_sim(token = "all")]
    All(CardReferences, Box<Condition>),
    // <condition> and <condition> -> <condition>
    #[hocg_fan_sim(infix = "and")]
    And(Box<Condition>, Box<Condition>),
    // any <[card_ref]> <condition> -> <condition>
    #[hocg_fan_sim(token = "any")]
    Any(CardReferences, Box<Condition>),
    // anything -> <condition>
    #[hocg_fan_sim(token = "anything")]
    Anything,
    // <value> == <value> -> <condition>
    #[hocg_fan_sim(infix = "==")]
    Equals(Value, Value),
    // exist <[card_ref]> -> <condition>
    #[hocg_fan_sim(token = "exist")]
    Exist(CardReferences),
    // false -> <condition>
    #[hocg_fan_sim(token = "false")]
    False,
    // <value> >= <value> -> <condition>
    #[hocg_fan_sim(infix = ">=")]
    GreaterThanEquals(Value, Value),
    // has_cheers -> <condition>
    #[hocg_fan_sim(token = "has_cheers")]
    HasCheers,
    // is_attribute_buzz -> <condition>
    #[hocg_fan_sim(token = "is_attribute_buzz")]
    IsAttributeBuzz,
    // is_color_green -> <condition>
    #[hocg_fan_sim(token = "is_color_green")]
    IsColorGreen,
    // is_color_white -> <condition>
    #[hocg_fan_sim(token = "is_color_white")]
    IsColorWhite,
    // is_cheer -> <condition>
    #[hocg_fan_sim(token = "is_cheer")]
    IsCheer,
    // is_even <value> -> <condition>
    #[hocg_fan_sim(token = "is_even")]
    IsEven(Value),
    // is_level_first -> <condition>
    #[hocg_fan_sim(token = "is_level_first")]
    IsLevelFirst,
    // is_level_second -> <condition>
    #[hocg_fan_sim(token = "is_level_second")]
    IsLevelSecond,
    // is_member -> <condition>
    #[hocg_fan_sim(token = "is_member")]
    IsMember,
    // is_named_azki -> <condition>
    #[hocg_fan_sim(token = "is_named_azki")]
    IsNamedAzki,
    // is_named_tokino_sora -> <condition>
    #[hocg_fan_sim(token = "is_named_tokino_sora")]
    IsNamedTokinoSora,
    // is_not <card_ref> -> <condition>
    #[hocg_fan_sim(token = "is_not")]
    IsNot(CardReference),
    // is_odd <value> -> <condition>
    #[hocg_fan_sim(token = "is_odd")]
    IsOdd(Value),
    // is_support_limited -> <condition>
    #[hocg_fan_sim(token = "is_support_limited")]
    IsSupportLimited,
    // <value> <= <value> -> <condition>
    #[hocg_fan_sim(infix = "<=")]
    LessThanEquals(Value, Value),
    // not <condition> -> <condition>
    #[hocg_fan_sim(token = "not")]
    Not(Box<Condition>),
    // <condition> or <condition> -> <condition>
    #[hocg_fan_sim(infix = "or")]
    Or(Box<Condition>, Box<Condition>),
    // true -> <condition>
    #[hocg_fan_sim(token = "true")]
    True,
    // <$var> -> <condition>
    #[hocg_fan_sim(transparent)]
    Var(Var),
    // yours -> <condition>
    #[hocg_fan_sim(token = "yours")]
    Yours,
}

#[derive(HocgFanSimCardEffect, Debug, Clone, PartialEq, Eq)]
pub enum LetValue {
    // optional_activate -> <condition>
    #[hocg_fan_sim(token = "optional_activate")]
    OptionalActivate,
    // roll_dice -> <value>
    #[hocg_fan_sim(token = "roll_dice")]
    RollDice,
    // select_any <[card_ref]> <condition> -> <[card_ref]> $_leftovers
    #[hocg_fan_sim(token = "select_any")]
    SelectAny(Box<CardReferences>, Box<Condition>),
    // select_one <[card_ref]> <condition> -> <[card_ref]> $_leftovers
    #[hocg_fan_sim(token = "select_one")]
    SelectOne(Box<CardReferences>, Box<Condition>),
    // select_number_between <value> <value> -> <value>
    #[hocg_fan_sim(token = "select_number_between")]
    SelectNumberBetween(Box<Value>, Box<Value>),
    // select_up_to <value> <[card_ref]> <condition> -> <[card_ref]> $_leftovers
    #[hocg_fan_sim(token = "select_up_to")]
    SelectUpTo(Box<Value>, Box<CardReferences>, Box<Condition>),
}

#[derive(HocgFanSimCardEffect, Debug, Clone, PartialEq, Eq)]
pub enum LifeTime {
    // this_game -> <life_time>
    #[hocg_fan_sim(token = "this_game")]
    ThisGame,
    // this_turn -> <life_time>
    #[hocg_fan_sim(token = "this_turn")]
    ThisTurn,
    // next_turn <player> -> <life_time>
    #[hocg_fan_sim(token = "next_turn")]
    NextTurn(Player),
    // this_step -> <life_time>
    #[hocg_fan_sim(token = "this_step")]
    ThisStep,
    // this_art -> <life_time>
    #[hocg_fan_sim(token = "this_art")]
    ThisArt,
    // this_effect -> <life_time>
    #[hocg_fan_sim(token = "this_effect")]
    ThisEffect,
    // until_removed -> <life_time>
    #[hocg_fan_sim(token = "until_removed")]
    UntilRemoved,
}

#[derive(HocgFanSimCardEffect, Debug, Clone, PartialEq, Eq)]
pub enum Modifier {
    // more_dmg <value> -> <mod>
    #[hocg_fan_sim(token = "more_dmg")]
    MoreDamage(Value),
    // next_dice_roll <value> -> <mod>
    #[hocg_fan_sim(token = "next_dice_roll")]
    NextDiceRoll(Value),
    // when <condition> <mod>  -> <mod>
    #[hocg_fan_sim(token = "when")]
    When(Condition, Box<Modifier>),
}

#[derive(HocgFanSimCardEffect, Debug, Clone, PartialEq, Eq)]
pub enum Player {
    // you -> <player>
    #[hocg_fan_sim(token = "you")]
    You,
    // opponent -> <player>
    #[hocg_fan_sim(token = "opponent")]
    Opponent,
}

#[derive(HocgFanSimCardEffect, Debug, Clone, PartialEq, Eq)]
pub enum Value {
    // count <[card_ref]> -> <value>
    #[hocg_fan_sim(token = "count")]
    Count(CardReferences),
    // 123 -> <value>
    #[hocg_fan_sim(transparent)]
    Number(Number),
    // <$var> -> <value>
    #[hocg_fan_sim(transparent)]
    Var(Var),
}

#[derive(HocgFanSimCardEffect, Debug, Clone, PartialEq, Eq)]
pub enum Zone {
    // archive -> <zone>
    #[hocg_fan_sim(token = "archive")]
    Archive,
    // back_stage -> <zone>
    #[hocg_fan_sim(token = "back_stage")]
    BackStage,
    // center_stage -> <zone>
    #[hocg_fan_sim(token = "center_stage")]
    CenterStage,
    // cheer_deck -> <zone>
    #[hocg_fan_sim(token = "cheer_deck")]
    CheerDeck,
    // hand -> <zone>
    #[hocg_fan_sim(token = "hand")]
    Hand,
    // holo_power -> <zone>
    #[hocg_fan_sim(token = "holo_power")]
    HoloPower,
    // main_deck -> <zone>
    #[hocg_fan_sim(token = "main_deck")]
    MainDeck,
    // main_stage -> <zone>
    #[hocg_fan_sim(token = "main_stage")]
    MainStage,
    // opponent_back_stage -> <zone>
    #[hocg_fan_sim(token = "opponent_back_stage")]
    OpponentBackStage,
    // opponent_center_stage -> <zone>
    #[hocg_fan_sim(token = "opponent_center_stage")]
    OpponentCenterStage,
    // stage -> <zone>
    #[hocg_fan_sim(token = "stage")]
    Stage,
}

//////////////////////////////////////

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
