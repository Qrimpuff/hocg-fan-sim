use std::collections::VecDeque;

use super::effects::*;
use super::error::*;
use super::parse::*;

// TODO try to build a macro for these, or somehow use serde

impl ParseTokens for BuiltIn {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "self" => BuiltIn::CurrentCard,
            "center_mem" => BuiltIn::CenterHoloMember,
            _ => {
                Self::return_string(tokens, s);
                return Err(Error::Message("TODO expected built in".into()));
            }
        })
    }
}

impl ParseTokens for Var {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        let s = Self::take_string(tokens)?;
        if s.starts_with('$') {
            Ok(Var(s))
        } else {
            Self::return_string(tokens, s);
            Err(Error::Message("TODO expected var".into()))
        }
    }
}

impl ParseTokens for Number {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        let s = Self::take_string(tokens)?;
        if let Ok(n) = s.parse() {
            Ok(Number(n))
        } else {
            Self::return_string(tokens, s);
            Err(Error::Message("TODO expected number".into()))
        }
    }
}

impl ParseTokens for Target {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        if let Ok(b) = BuiltIn::take_param(tokens) {
            Ok(Target::BuiltIn(b))
        } else if let Ok(v) = Var::take_param(tokens) {
            Ok(Target::Var(v))
        } else {
            Err(Error::Message("TODO expected built in or var".into()))
        }
    }
}

impl ParseTokens for Action {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "no_action" => Action::Noop,
            "for" => Action::For(tokens.take_param()?, Box::new(tokens.take_param()?)),
            "buff" => Action::Buff(tokens.take_param()?, tokens.take_param()?),
            "debuff" => Action::Debuff(tokens.take_param()?, tokens.take_param()?),
            "heal" => Action::Heal(tokens.take_param()?),
            "let" => Action::Let(tokens.take_param()?, tokens.take_param()?),
            "when" => Action::When(tokens.take_param()?, Box::new(tokens.take_param()?)),
            "draw" => Action::Draw(tokens.take_param()?),
            _ => {
                Self::return_string(tokens, s);
                return Err(Error::Message("TODO expected action".into()));
            }
        })
    }
}

impl ParseTokens for Value {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "for" => Value::For(tokens.take_param()?, Box::new(tokens.take_param()?)),
            "get" => Value::Get(tokens.take_param()?),
            "_add" => Value::Add(
                Box::new(tokens.take_param()?),
                Box::new(tokens.take_param()?),
            ),
            "_sub" => Value::Subtract(
                Box::new(tokens.take_param()?),
                Box::new(tokens.take_param()?),
            ),
            "_mul" => Value::Multiply(
                Box::new(tokens.take_param()?),
                Box::new(tokens.take_param()?),
            ),
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

impl ParseTokens for Condition {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "always" => Condition::Always,
            "once_per_turn" => Condition::OncePerTurn,
            "eq" => Condition::Equals(tokens.take_param()?, tokens.take_param()?),
            "has" => Condition::Has(tokens.take_param()?, tokens.take_param()?),
            "neq" => Condition::NotEquals(tokens.take_param()?, tokens.take_param()?),
            _ => {
                Self::return_string(tokens, s);
                return Err(Error::Message("TODO expected condition".into()));
            }
        })
    }
}

impl ParseTokens for Tag {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
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

impl ParseTokens for Property {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "hp" => Property::HealthPoint,
            "r_cost" => Property::RetreatCost,
            _ => {
                Self::return_string(tokens, s);
                return Err(Error::Message("TODO expected property".into()));
            }
        })
    }
}

impl ParseTokens for Buff {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "more_def" => Buff::MoreDefense(tokens.take_param()?),
            "more_atk" => Buff::MoreAttack(tokens.take_param()?),
            _ => {
                Self::return_string(tokens, s);
                return Err(Error::Message("TODO expected buff".into()));
            }
        })
    }
}

impl ParseTokens for Debuff {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "less_def" => Debuff::LessDefense(tokens.take_param()?),
            "less_atk" => Debuff::LessAttack(tokens.take_param()?),
            _ => {
                Self::return_string(tokens, s);
                return Err(Error::Message("TODO expected debuff".into()));
            }
        })
    }
}

impl ParseTokens for LifeTime {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
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
