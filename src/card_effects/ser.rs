use super::effects::*;
use super::parse::*;

// TODO try to build a macro for these, or somehow use serde

impl From<BuiltIn> for Tokens {
    fn from(value: BuiltIn) -> Self {
        match value {
            BuiltIn::CurrentCard => "self".into(),
            BuiltIn::CenterHoloMember => "center_mem".into(),
        }
    }
}

impl From<Var> for Tokens {
    fn from(value: Var) -> Self {
        value.0.as_str().into()
    }
}

impl From<Number> for Tokens {
    fn from(value: Number) -> Self {
        value.0.to_string().as_str().into()
    }
}

impl From<Target> for Tokens {
    fn from(value: Target) -> Self {
        match value {
            Target::BuiltIn(b) => b.into(),
            Target::Var(v) => v.into(),
        }
    }
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
        .collect::<Vec<_>>()
        .join("\n")
}

impl From<Action> for Tokens {
    fn from(value: Action) -> Self {
        match value {
            Action::Noop => "no_action".into(),
            Action::For(a, b) => ["for".into(), a.into(), (*b).into()].into(),
            Action::Buff(t, l) => ["buff".into(), t.into(), l.into()].into(),
            Action::Debuff(b, l) => ["debuff".into(), b.into(), l.into()].into(),
            Action::Heal(v) => ["heal".into(), v.into()].into(),
            Action::Let(r, v) => ["let".into(), r.into(), v.into()].into(),
            Action::When(c, a) => ["when".into(), c.into(), (*a).into()].into(),
            Action::Draw(d) => ["draw".into(), d.into()].into(),
        }
    }
}

impl From<Value> for Tokens {
    fn from(value: Value) -> Self {
        match value {
            Value::For(a, b) => ["for".into(), a.into(), (*b).into()].into(),
            Value::Get(a) => ["get".into(), a.into()].into(),
            Value::Number(a) => a.into(),
            Value::Var(a) => a.into(),
        }
    }
}

impl From<Condition> for Tokens {
    fn from(value: Condition) -> Self {
        match value {
            Condition::Always => "always".into(),
            Condition::OncePerTurn => "once_per_turn".into(),
            Condition::Equals(a, b) => ["eq".into(), a.into(), b.into()].into(),
            Condition::Has(a, b) => ["has".into(), a.into(), b.into()].into(),
            Condition::NotEquals(a, b) => ["neq".into(), a.into(), b.into()].into(),
        }
    }
}

impl From<Tag> for Tokens {
    fn from(value: Tag) -> Self {
        match value {
            Tag::ColorWhite => "c_white".into(),
            Tag::ColorGreen => "c_green".into(),
            Tag::ColorBlue => "c_blue".into(),
            Tag::ColorRed => "c_red".into(),
            Tag::ColorPurple => "c_purple".into(),
            Tag::ColorYellow => "c_yellow".into(),
            Tag::StageDebut => "s_debut".into(),
            Tag::StageFirst => "s_first".into(),
            Tag::StageSecond => "s_second".into(),
        }
    }
}

impl From<Property> for Tokens {
    fn from(value: Property) -> Self {
        match value {
            Property::HealthPoint => "hp".into(),
            Property::RetreatCost => "r_cost".into(),
        }
    }
}

impl From<Buff> for Tokens {
    fn from(value: Buff) -> Self {
        match value {
            Buff::MoreDefense(a) => ["more_def".into(), a.into()].into(),
            Buff::MoreAttack(a) => ["more_atk".into(), a.into()].into(),
        }
    }
}

impl From<Debuff> for Tokens {
    fn from(value: Debuff) -> Self {
        match value {
            Debuff::LessDefense(a) => ["less_def".into(), a.into()].into(),
            Debuff::LessAttack(a) => ["less_atk".into(), a.into()].into(),
        }
    }
}

impl From<LifeTime> for Tokens {
    fn from(value: LifeTime) -> Self {
        match value {
            LifeTime::ThisAttack => "this_attack".into(),
            LifeTime::ThisTurn => "this_turn".into(),
            LifeTime::NextTurn => "next_turn".into(),
            LifeTime::Limitless => "_".into(),
        }
    }
}
