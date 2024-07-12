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

use super::error::Error;
use super::parse::*;
use std::collections::VecDeque;

#[derive(Debug, PartialEq)]
pub enum BuiltIn {
    CurrentCard,
    ActiveHoloMember,
}

impl ParseTokens for BuiltIn {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self, Error> {
        dbg!(&tokens);
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "self" => BuiltIn::CurrentCard,
            "active_holo" => BuiltIn::ActiveHoloMember,
            _ => {
                Self::return_string(tokens, s);
                return Err(Error::Message("TODO expected built in".into()));
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Var(String);

impl ParseTokens for Var {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self, Error> {
        dbg!(&tokens);
        let s = Self::take_string(tokens)?;
        if s.starts_with("$") {
            Ok(Var(s.into()))
        } else {
            Self::return_string(tokens, s);
            Err(Error::Message("TODO expected var".into()))
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Number(u32);

impl ParseTokens for Number {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self, Error> {
        dbg!(&tokens);
        let s = Self::take_string(tokens)?;
        if let Ok(n) = s.parse() {
            Ok(Number(n))
        } else {
            Self::return_string(tokens, s);
            Err(Error::Message("TODO expected number".into()))
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Target {
    BuiltIn(BuiltIn),
    Var(Var),
}

impl ParseTokens for Target {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self, Error> {
        dbg!(&tokens);
        if let Ok(b) = BuiltIn::take_param(tokens) {
            Ok(Target::BuiltIn(b))
        } else if let Ok(v) = Var::take_param(tokens) {
            Ok(Target::Var(v))
        } else {
            Err(Error::Message("TODO expected built in or var".into()))
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Action {
    For(Target, Box<Action>),
    Buff(Buff, LifeTime),
    Debuff(Debuff, LifeTime),
    Heal(Value),
    Let(Var, Value),
    When(Condition, Box<Action>),
}

impl ParseTokens for Action {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self, Error> {
        dbg!(&tokens);
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "for" => Action::For(tokens.take_param()?, Box::new(tokens.take_param()?)),
            "buff" => Action::Buff(tokens.take_param()?, tokens.take_param()?),
            "debuff" => Action::Debuff(tokens.take_param()?, tokens.take_param()?),
            "heal" => Action::Heal(tokens.take_param()?),
            "let" => Action::Let(tokens.take_param()?, tokens.take_param()?),
            "when" => Action::When(tokens.take_param()?, Box::new(tokens.take_param()?)),
            _ => {
                Self::return_string(tokens, s);
                return Err(Error::Message("TODO expected action".into()));
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Value {
    For(Target, Box<Value>),
    Get(Property),
    Number(Number),
    Var(Var),
}

impl ParseTokens for Value {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self, Error> {
        dbg!(&tokens);
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "for" => Value::For(tokens.take_param()?, Box::new(tokens.take_param()?)),
            "get" => Value::Get(tokens.take_param()?),
            _ => {
                Self::return_string(tokens, s);
                if let Ok(n) = Number::take_param(tokens) {
                    Value::Number(n)
                } else if let Ok(v) = Var::take_param(tokens) {
                    Value::Var(v)
                } else {
                    return Err(Error::Message("TODO expected value".into()));
                }
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Condition {
    Equals(Value, Value),
    Has(Target, Tag),
    NotEquals(Value, Value),
}

impl ParseTokens for Condition {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self, Error> {
        dbg!(&tokens);
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "eq" => Condition::Equals(tokens.take_param()?, tokens.take_param()?),
            "has" => Condition::Equals(tokens.take_param()?, tokens.take_param()?),
            "neq" => Condition::Equals(tokens.take_param()?, tokens.take_param()?),
            _ => {
                Self::return_string(tokens, s);
                return Err(Error::Message("TODO expected condition".into()));
            }
        })
    }
}

#[derive(Debug, PartialEq)]
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

impl ParseTokens for Tag {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self, Error> {
        dbg!(&tokens);
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            // colors
            "c_white" => Tag::ColorWhite,
            "c_green" => Tag::ColorGreen,
            "c_blue" => Tag::ColorBlue,
            "c_red" => Tag::ColorRed,
            "c_purple" => Tag::ColorPurple,
            "c_yellow" => Tag::ColorYellow,
            // stages
            "s_debut" => Tag::StageDebut,
            "s_first" => Tag::StageFirst,
            "s_second" => Tag::StageSecond,
            _ => {
                Self::return_string(tokens, s);
                return Err(Error::Message("TODO expected tag".into()));
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Property {
    HealtPoint,
    RetreatCost,
}

impl ParseTokens for Property {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self, Error> {
        dbg!(&tokens);
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "hp" => Property::HealtPoint,
            "r_cost" => Property::RetreatCost,
            _ => {
                Self::return_string(tokens, s);
                return Err(Error::Message("TODO expected property".into()));
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Buff {
    MoreDefence(Value),
    MoreAttack(Value),
}

impl ParseTokens for Buff {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self, Error> {
        dbg!(&tokens);
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "more_def" => Buff::MoreDefence(tokens.take_param()?),
            "more_atk" => Buff::MoreAttack(tokens.take_param()?),
            _ => {
                Self::return_string(tokens, s);
                return Err(Error::Message("TODO expected buff".into()));
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Debuff {
    LessDefence(Value),
    LessAttack(Value),}

impl ParseTokens for Debuff {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self, Error> {
        dbg!(&tokens);
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "less_def" => Debuff::LessDefence(tokens.take_param()?),
            "less_atk" => Debuff::LessAttack(tokens.take_param()?),
            _ => {
                Self::return_string(tokens, s);
                return Err(Error::Message("TODO expected debuff".into()));
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum LifeTime {
    ThisAttack,
    ThisTurn,
    NextTurn,
    Limitless,
}

impl ParseTokens for LifeTime {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self, Error> {
        dbg!(&tokens);
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "this_attack" => LifeTime::ThisAttack,
            "this_turn" => LifeTime::ThisTurn,
            "next_turn" => LifeTime::NextTurn,
            "_" => LifeTime::Limitless,
            _ => {
                Self::return_string(tokens, s);
                return Err(Error::Message("TODO expected lifetime".into()));
            }
        })
    }
}
