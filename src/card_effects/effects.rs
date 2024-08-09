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
pub enum Action {
    // add_global_mod <mod> <life_time> -> action
    #[holo_ucg(token = "add_global_mod")]
    AddGlobalModifier(Modifier, LifeTime),
    // add_mod <[card_ref]> <mod> <life_time> -> <action>
    #[holo_ucg(token = "add_mod")]
    AddModifier(CardReferences, Modifier, LifeTime),
    // add_zone_mod <zone> <mod> <life_time> -> action
    #[holo_ucg(token = "add_zone_mod")]
    AddZoneModifier(Zone, Modifier, LifeTime),
    // attach_cards <[card_ref]> <card_ref> -> <action>
    #[holo_ucg(token = "attach_cards")]
    AttachCards(CardReferences, CardReference),
    // draw <value> -> <action>
    #[holo_ucg(token = "draw")]
    Draw(Value),
    // if <condition> <[action]> -> <action>
    #[holo_ucg(token = "if")]
    If(Condition, Vec<Action>),
    // let <$var> = <[card_ref]> -> <action>
    #[holo_ucg(transparent)]
    LetCardReferences(Let<CardReferences>),
    // let <$var> = <condition> -> <action>
    #[holo_ucg(transparent)]
    LetCondition(Let<Condition>),
    // let <$var> = <value> -> <action>
    #[holo_ucg(transparent)]
    LetValue(Let<Value>),
    // no_action -> <action>
    #[holo_ucg(token = "no_action")]
    Noop,
    // reveal <[card_ref]> -> <action>
    #[holo_ucg(token = "reveal")]
    Reveal(CardReferences),
    // roll_dice -> <action> $_dice_value
    #[holo_ucg(token = "roll_dice")]
    RollDice,
    // send_to <zone> <[card_ref]> -> <action>
    #[holo_ucg(token = "send_to")]
    SendTo(Zone, CardReferences),
    // send_to_bottom <zone> <[card_ref]> -> <action>
    #[holo_ucg(token = "send_to_bottom")]
    SendToBottom(Zone, CardReferences),
    // send_to_top <zone> <[card_ref]> -> <action>
    #[holo_ucg(token = "send_to_top")]
    SendToTop(Zone, CardReferences),
    // shuffle <zone> -> <action>
    #[holo_ucg(token = "shuffle")]
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

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum CardReference {
    // TODO figure out the conversion with CardReferences, could panic
    // this_card -> <card_ref>
    #[holo_ucg(token = "this_card")]
    ThisCard,
    // <$var> -> <card_ref>
    #[holo_ucg(transparent)]
    Var(Var),
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum CardReferences {
    // attached <card_ref> -> <[card_ref]>
    #[holo_ucg(token = "attached")]
    Attached(CardReference),
    // from_zone <zone> -> <[card_ref]>
    #[holo_ucg(token = "from_zone")]
    FromZone(Zone),
    // from_zone_top <value> <zone> -> <[card_ref]>
    #[holo_ucg(token = "from_zone_top")]
    FromZoneTop(Box<Value>, Zone),
    // from_zone_bottom <value> <zone> -> <[card_ref]>
    #[holo_ucg(token = "from_zone_bottom")]
    FromZoneBottom(Box<Value>, Zone),
    // select_any <[card_ref]> <condition> -> <[card_ref]> $_leftovers
    #[holo_ucg(token = "select_any")]
    SelectAny(Box<CardReferences>, Box<Condition>),
    // select_one <[card_ref]> <condition> -> <[card_ref]> $_leftovers
    #[holo_ucg(token = "select_one")]
    SelectOne(Box<CardReferences>, Box<Condition>),
    // select_up_to <value> <[card_ref]> <condition> -> <[card_ref]> $_leftovers
    #[holo_ucg(token = "select_up_to")]
    SelectUpTo(Box<Value>, Box<CardReferences>, Box<Condition>),
    // this_card -> <[card_ref]>
    #[holo_ucg(token = "this_card")]
    ThisCard,
    // <$var> -> <[card_ref]>
    #[holo_ucg(transparent)]
    Var(Var),
    // <[card_ref]> where <condition> -> <[card_ref]>
    #[holo_ucg(infix = "where")]
    Where(Box<CardReferences>, Box<Condition>),
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    // <[card_ref]> all <condition> -> <condition>
    #[holo_ucg(infix = "all")]
    All(CardReferences, Box<Condition>),
    // <condition> and <condition> -> <condition>
    #[holo_ucg(infix = "and")]
    And(Box<Condition>, Box<Condition>),
    // <[card_ref]> any <condition> -> <condition>
    #[holo_ucg(infix = "any")]
    Any(CardReferences, Box<Condition>),
    // any -> <condition>
    #[holo_ucg(token = "any")]
    AnyTrue,
    // <value> == <value> -> <condition>
    #[holo_ucg(infix = "==")]
    Equals(Value, Value),
    // exist <[card_ref]> -> <condition>
    #[holo_ucg(token = "exist")]
    Exist(CardReferences),
    // false -> <condition>
    #[holo_ucg(token = "false")]
    False,
    // <value> >= <value> -> <condition>
    #[holo_ucg(infix = ">=")]
    GreaterThanEquals(Value, Value),
    // has_cheers -> <condition>
    #[holo_ucg(token = "has_cheers")]
    HasCheers,
    // is_attribute_buzz -> <condition>
    #[holo_ucg(token = "is_attribute_buzz")]
    IsAttributeBuzz,
    // is_color_green -> <condition>
    #[holo_ucg(token = "is_color_green")]
    IsColorGreen,
    // is_color_white -> <condition>
    #[holo_ucg(token = "is_color_white")]
    IsColorWhite,
    // is_cheer -> <condition>
    #[holo_ucg(token = "is_cheer")]
    IsCheer,
    // is_even <value> -> <condition>
    #[holo_ucg(token = "is_even")]
    IsEven(Value),
    // is_level_first -> <condition>
    #[holo_ucg(token = "is_level_first")]
    IsLevelFirst,
    // is_level_second -> <condition>
    #[holo_ucg(token = "is_level_second")]
    IsLevelSecond,
    // is_member -> <condition>
    #[holo_ucg(token = "is_member")]
    IsMember,
    // is_named_azki -> <condition>
    #[holo_ucg(token = "is_named_azki")]
    IsNamedAzki,
    // is_named_tokino_sora -> <condition>
    #[holo_ucg(token = "is_named_tokino_sora")]
    IsNamedTokinoSora,
    // is_not <card_ref> -> <condition>
    #[holo_ucg(token = "is_not")]
    IsNot(CardReference),
    // is_odd <value> -> <condition>
    #[holo_ucg(token = "is_odd")]
    IsOdd(Value),
    // is_support_limited -> <condition>
    #[holo_ucg(token = "is_support_limited")]
    IsSupportLimited,
    // <value> <= <value> -> <condition>
    #[holo_ucg(infix = "<=")]
    LessThanEuqals(Value, Value),
    // not <condition> -> <condition>
    #[holo_ucg(token = "not")]
    Not(Box<Condition>),
    // optional -> <condition>
    #[holo_ucg(token = "optional")]
    Optional,
    // <condition> or <condition> -> <condition>
    #[holo_ucg(infix = "or")]
    Or(Box<Condition>, Box<Condition>),
    // true -> <condition>
    #[holo_ucg(token = "true")]
    True,
    // <$var> -> <condition>
    #[holo_ucg(transparent)]
    Var(Var),
    // yours -> <condition>
    #[holo_ucg(token = "yours")]
    Yours,
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum LifeTime {
    // this_game -> <life_time>
    #[holo_ucg(token = "this_game")]
    ThisGame,
    // this_turn -> <life_time>
    #[holo_ucg(token = "this_turn")]
    ThisTurn,
    // next_turn <player> -> <life_time>
    #[holo_ucg(token = "next_turn")]
    NextTurn(Player),
    // this_step -> <life_time>
    #[holo_ucg(token = "this_step")]
    ThisStep,
    // this_art -> <life_time>
    #[holo_ucg(token = "this_art")]
    ThisArt,
    // this_effect -> <life_time>
    #[holo_ucg(token = "this_effect")]
    ThisEffect,
    // until_removed -> <life_time>
    #[holo_ucg(token = "until_removed")]
    UntilRemoved,
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum Modifier {
    // more_dmg <value> -> <mod>
    #[holo_ucg(token = "more_dmg")]
    MoreDamage(Value),
    // next_dice_roll <value> -> <mod>
    #[holo_ucg(token = "next_dice_roll")]
    NextDiceRoll(Value),
    // when <condition> <mod>  -> <mod>
    #[holo_ucg(token = "when")]
    When(Condition, Box<Modifier>),
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum Player {
    // you -> <zone>
    #[holo_ucg(token = "you")]
    You,
    // opponent -> <zone>
    #[holo_ucg(token = "opponent")]
    Opponent,
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum Value {
    // count <[card_ref]> -> <value>
    #[holo_ucg(token = "count")]
    Count(CardReferences),
    // 123 -> <value>
    #[holo_ucg(transparent)]
    Number(Number),
    // select_number_between <value> <value> -> <value>
    #[holo_ucg(token = "select_number_between")]
    SelectNumberBetween(Box<Value>, Box<Value>),
    // <$var> -> <value>
    #[holo_ucg(transparent)]
    Var(Var),
}

#[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
pub enum Zone {
    // archive -> <zone>
    #[holo_ucg(token = "archive")]
    Archive,
    // back_stage -> <zone>
    #[holo_ucg(token = "back_stage")]
    BackStage,
    // center_stage -> <zone>
    #[holo_ucg(token = "center_stage")]
    CenterStage,
    // cheer_deck -> <zone>
    #[holo_ucg(token = "cheer_deck")]
    CheerDeck,
    // hand -> <zone>
    #[holo_ucg(token = "hand")]
    Hand,
    // holo_power -> <zone>
    #[holo_ucg(token = "holo_power")]
    HoloPower,
    // main_deck -> <zone>
    #[holo_ucg(token = "main_deck")]
    MainDeck,
    // opponent_back_stage -> <zone>
    #[holo_ucg(token = "opponent_back_stage")]
    OpponentBackStage,
    // opponent_center_stage -> <zone>
    #[holo_ucg(token = "opponent_center_stage")]
    OpponentCenterStage,
    // stage -> <zone>
    #[holo_ucg(token = "stage")]
    Stage,
}

//////////////////////////////////////

// #[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
// pub enum TargetPlayer {
//     #[holo_ucg(token = "self")]
//     CurrentPlayer,
//     #[holo_ucg(token = "opponent")]
//     Opponent,
//     #[holo_ucg(transparent)]
//     Var(Var),
// }

// #[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
// pub enum TargetPlayers {
//     // one player
//     #[holo_ucg(token = "self")]
//     CurrentPlayer,
//     #[holo_ucg(token = "opponent")]
//     Opponent,
//     // many players
//     #[holo_ucg(token = "all_players")]
//     AllPlayers,

//     #[holo_ucg(infix = "with")]
//     With(Box<TargetPlayers>, Attribute), // TODO not sure about tag, should be condition (player)
//     #[holo_ucg(transparent)]
//     Var(Var),
// }

// #[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
// pub enum TargetCard {
//     #[holo_ucg(token = "self")]
//     CurrentCard,
//     #[holo_ucg(token = "oshi")]
//     Oshi,
//     #[holo_ucg(token = "center_member")]
//     CenterHoloMember,
//     #[holo_ucg(token = "collab_member")]
//     CollabHoloMember,
//     #[holo_ucg(token = "select_member")]
//     SelectMember(Box<TargetCards>),

//     #[holo_ucg(transparent)]
//     Var(Var),
// }

// #[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
// pub enum TargetCards {
//     // one card
//     #[holo_ucg(token = "self")]
//     CurrentCard,
//     #[holo_ucg(token = "oshi")]
//     Oshi,
//     #[holo_ucg(token = "center_member")]
//     CenterHoloMember,
//     #[holo_ucg(token = "collab_member")]
//     CollabHoloMember,
//     #[holo_ucg(token = "select_member")]
//     SelectMember(Box<TargetCards>),
//     // many cards
//     #[holo_ucg(token = "main_stage_members")]
//     MainStageMembers,
//     #[holo_ucg(token = "back_stage_members")]
//     BackStageMembers,
//     #[holo_ucg(token = "stage_members")]
//     StageMembers,
//     #[holo_ucg(token = "attached_cheers")]
//     AttachedCheers,

//     #[holo_ucg(token = "select_cheers_up_to")]
//     SelectCheersUpTo(Box<Value>, Box<TargetCards>),
//     #[holo_ucg(token = "cheers_in_archive")]
//     CheersInArchive,
//     #[holo_ucg(infix = "with")]
//     With(Box<TargetCards>, Attribute),
//     #[holo_ucg(transparent)]
//     Var(Var),
// }

// #[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
// pub enum Value {
//     #[holo_ucg(token = "for")]
//     For(TargetCards, Box<Value>),
//     #[holo_ucg(token = "get")]
//     Get(Property),
//     #[holo_ucg(transparent)]
//     Number(Number),
//     #[holo_ucg(transparent)]
//     Var(Var),
//     #[holo_ucg(infix = "+")]
//     Add(Box<Value>, Box<Value>),
//     #[holo_ucg(infix = "-")]
//     Subtract(Box<Value>, Box<Value>),
//     #[holo_ucg(infix = "*")]
//     Multiply(Box<Value>, Box<Value>),
//     #[holo_ucg(token = "select_dice_number")]
//     SelectDiceNumber,
//     #[holo_ucg(token = "roll_dice")]
//     RollDice,
//     #[holo_ucg(token = "all")]
//     All,
// }

// #[derive(HoloUcgEffect, Debug, Clone, Copy, PartialEq, Eq)]
// pub enum DamageModifier {
//     #[holo_ucg(token = "none")]
//     None,
//     #[holo_ucg(token = "plus")]
//     Plus(Number),
//     #[holo_ucg(token = "minus")]
//     Minus(Number),
//     #[holo_ucg(token = "times")]
//     Times(Number),
// }

// #[derive(HoloUcgEffect, Debug, Clone, Copy, PartialEq, Eq)]
// pub enum Attribute {
//     // colors
//     #[holo_ucg(token = "color_white")]
//     ColorWhite,
//     #[holo_ucg(token = "color_green")]
//     ColorGreen,
//     #[holo_ucg(token = "color_blue")]
//     ColorBlue,
//     #[holo_ucg(token = "color_red")]
//     ColorRed,
//     #[holo_ucg(token = "color_purple")]
//     ColorPurple,
//     #[holo_ucg(token = "color_yellow")]
//     ColorYellow,
//     // stages
//     #[holo_ucg(token = "level_debut")]
//     LevelDebut,
//     #[holo_ucg(token = "level_first")]
//     LevelFirst,
//     #[holo_ucg(token = "level_second")]
//     LevelSecond,
//     // names
//     #[holo_ucg(token = "name_azki")]
//     NameAzki,
// }

// #[derive(HoloUcgEffect, Debug, Clone, Copy, PartialEq, Eq)]
// pub enum Property {
//     #[holo_ucg(token = "hp")]
//     HealthPoint,
//     #[holo_ucg(token = "r_cost")]
//     RetreatCost,
// }

// #[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
// pub enum Buff {
//     #[holo_ucg(token = "more_dmg")]
//     MoreDamage(Value),
// }

// #[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
// pub enum Debuff {
//     #[holo_ucg(token = "less_def")]
//     LessDefense(Value),
//     #[holo_ucg(token = "less_atk")]
//     LessAttack(Value),
// }

// #[derive(HoloUcgEffect, Debug, Clone, PartialEq, Eq)]
// pub enum Condition {
//     #[holo_ucg(token = "always")]
//     Always,
//     #[holo_ucg(token = "true")]
//     True,
//     #[holo_ucg(token = "false")]
//     False,
//     #[holo_ucg(infix = "==")]
//     Equals(Value, Value),
//     #[holo_ucg(infix = "has")]
//     Has(TargetCard, Attribute),
//     #[holo_ucg(infix = "have")]
//     Have(TargetCards, Attribute),
//     #[holo_ucg(infix = "!=")]
//     NotEquals(Value, Value),
//     #[holo_ucg(infix = "and")]
//     And(Box<Condition>, Box<Condition>),
//     #[holo_ucg(infix = "or")]
//     Or(Box<Condition>, Box<Condition>),

//     #[holo_ucg(token = "is_odd")]
//     IsOdd(Value),
//     #[holo_ucg(token = "is_even")]
//     IsEven(Value),

//     #[holo_ucg(token = "once_per_turn")]
//     OncePerTurn,
//     #[holo_ucg(token = "once_per_game")]
//     OncePerGame,
//     #[holo_ucg(token = "is_holo_member")]
//     IsHoloMember,
// }

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
