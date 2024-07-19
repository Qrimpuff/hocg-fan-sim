use std::collections::VecDeque;

use super::effects::*;
use super::error::*;
use super::parse::*;

// TODO try to build a macro for these, or somehow use serde
// TODO clean up this file after the list of effect is finalized

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

impl ParseTokens for Target {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "self" => Target::CurrentCard,
            "center_mem" => Target::CenterHoloMember,
            "select_member" => Target::SelectMember(Box::new(tokens.take_param()?)),
            "_with" => Target::With(Box::new(tokens.take_param()?), tokens.take_param()?),
            "members_on_stage" => Target::MembersOnStage,
            "select_cheers_up_to" => Target::SelectCheersUpTo(
                Box::new(tokens.take_param()?),
                Box::new(tokens.take_param()?),
            ),
            "cheers_in_archive" => Target::CheersInArchive,
            _ => {
                Self::return_string(tokens, s.clone());
                // then it could ba a variable
                Var::take_param(tokens)
                    .map(Target::Var)
                    .map_err(|e| match e {
                        Error::UnexpectedToken(_, t) => Error::UnexpectedToken("target".into(), t),
                        _ => e,
                    })?
            }
        })
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
            "next_dice_number" => Action::NextDiceNumber(tokens.take_param()?),
            "attach" => Action::Attach(tokens.take_param()?),
            _ => {
                Self::return_string(tokens, s.clone());
                return Err(Error::UnexpectedToken("action".into(), s));
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
            "select_dice_number" => Value::SelectDiceNumber,
            "all" => Value::All,
            _ => {
                Self::return_string(tokens, s.clone());
                if let Ok(n) = Number::take_param(tokens) {
                    Value::Number(n)
                } else if let Ok(v) = Var::take_param(tokens) {
                    Value::Var(v)
                } else {
                    return Err(Error::UnexpectedToken("value".into(), s));
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
            "once_per_game" => Condition::OncePerGame,
            "eq" => Condition::Equals(tokens.take_param()?, tokens.take_param()?),
            "has" => Condition::Has(tokens.take_param()?, tokens.take_param()?),
            "neq" => Condition::NotEquals(tokens.take_param()?, tokens.take_param()?),
            "_and" => Condition::And(
                Box::new(tokens.take_param()?),
                Box::new(tokens.take_param()?),
            ),
            "_or" => Condition::Or(
                Box::new(tokens.take_param()?),
                Box::new(tokens.take_param()?),
            ),
            "is_holo_member" => Condition::IsHoloMember,
            _ => {
                Self::return_string(tokens, s.clone());
                return Err(Error::UnexpectedToken("condition".into(), s));
            }
        })
    }
}

impl ParseTokens for Tag {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            // colors
            "color_white" => Tag::ColorWhite,
            "color_green" => Tag::ColorGreen,
            "color_blue" => Tag::ColorBlue,
            "color_red" => Tag::ColorRed,
            "color_purple" => Tag::ColorPurple,
            "color_yellow" => Tag::ColorYellow,
            // stages
            "level_debut" => Tag::LevelDebut,
            "level_first" => Tag::LevelFirst,
            "level_second" => Tag::LevelSecond,
            _ => {
                Self::return_string(tokens, s.clone());
                return Err(Error::UnexpectedToken("tag".into(), s));
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
                Self::return_string(tokens, s.clone());
                return Err(Error::UnexpectedToken("property".into(), s));
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
                Self::return_string(tokens, s.clone());
                return Err(Error::UnexpectedToken("buff".into(), s));
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
                Self::return_string(tokens, s.clone());
                return Err(Error::UnexpectedToken("debuff".into(), s));
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
                Self::return_string(tokens, s.clone());
                return Err(Error::UnexpectedToken("lifetime".into(), s));
            }
        })
    }
}

impl ParseTokens for DamageModifier {
    fn parse_tokens(tokens: &mut VecDeque<Tokens>) -> Result<Self> {
        let s = Self::take_string(tokens)?;
        Ok(match s.as_str() {
            "none" => DamageModifier::None,
            "plus" => DamageModifier::Plus(tokens.take_param()?),
            "minus" => DamageModifier::Minus(tokens.take_param()?),
            "times" => DamageModifier::Times(tokens.take_param()?),
            _ => {
                Self::return_string(tokens, s.clone());
                return Err(Error::UnexpectedToken("damage modifier".into(), s));
            }
        })
    }
}
