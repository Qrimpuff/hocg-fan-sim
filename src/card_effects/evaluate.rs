use super::effects::*;
use crate::{gameplay::*, modifiers::DamageMarkers, HoloMemberHp};

pub struct EvaluateContext<'a, P: Prompter> {
    active_card: CardRef,
    card_target: CardRef,
    player_target: Player,
    game: &'a mut Game<P>,
}

impl<'a, P: Prompter> EvaluateContext<'a, P> {
    pub fn new(game: &'a mut Game<P>, card: CardRef) -> Self {
        EvaluateContext {
            active_card: card,
            card_target: card,
            player_target: game.player_for_card(card),
            game,
        }
    }
}

pub trait EvaluateEffect {
    type Value;

    fn evaluate<P: Prompter>(&self, ctx: &mut EvaluateContext<P>) -> Self::Value;

    fn start_evaluate<P: Prompter>(&self, game: &mut Game<P>, card: CardRef) -> Self::Value {
        self.evaluate(&mut EvaluateContext::new(game, card))
    }
}

impl<T> EvaluateEffect for Vec<T>
where
    T: EvaluateEffect<Value = ()>,
{
    type Value = ();

    fn evaluate<P: Prompter>(&self, ctx: &mut EvaluateContext<P>) -> Self::Value {
        for action in self {
            action.evaluate(ctx);
        }
    }
}

impl EvaluateEffect for Target {
    type Value = CardRef;

    fn evaluate<P: Prompter>(&self, ctx: &mut EvaluateContext<P>) -> Self::Value {
        match self {
            Target::BuiltIn(b) => match b {
                BuiltIn::CurrentCard => ctx.active_card,
                BuiltIn::CenterHoloMember => ctx
                    .game
                    .board(ctx.player_target)
                    .get_zone(Zone::MainStageCenter)
                    .peek_top_card()
                    .expect("there should be a center member"),
            },
            Target::Var(_) => todo!(),
        }
    }
}

impl EvaluateEffect for Action {
    type Value = ();

    fn evaluate<P: Prompter>(&self, ctx: &mut EvaluateContext<P>) -> Self::Value {
        match self {
            Action::Noop => {
                println!("*nothing happens*")
            }
            Action::For(t, a) => {
                // FIXME only handles card for now
                let past_target = ctx.card_target;
                let target = t.evaluate(ctx);
                ctx.card_target = target;
                a.evaluate(ctx);
                ctx.card_target = past_target;
            }
            Action::Buff(_, _) => todo!(),
            Action::Debuff(_, _) => todo!(),
            Action::Heal(h) => {
                let heal = h.evaluate(ctx);
                let card = ctx.card_target;
                let mem = ctx
                    .game
                    .lookup_holo_member(card)
                    .expect("can only heal members");

                println!("heal {} for card {}", heal, mem.name);
                ctx.game
                    .remove_damage(card, DamageMarkers::from_hp(heal as HoloMemberHp));
            }
            Action::Let(_, _) => todo!(),
            Action::When(_, _) => todo!(),
            Action::Draw(d) => {
                let draw = d.evaluate(ctx);

                println!("draw {} card(s)", draw);
                ctx.game.active_board_mut().draw(draw as usize);
            }
        }
    }
}

impl EvaluateEffect for Value {
    type Value = u32;

    fn evaluate<P: Prompter>(&self, _ctx: &mut EvaluateContext<P>) -> Self::Value {
        match self {
            Value::For(_, _) => todo!(),
            Value::Get(_) => todo!(),
            Value::Number(n) => n.0,
            Value::Var(_) => todo!(),
        }
    }
}

impl EvaluateEffect for Condition {
    type Value = bool;

    fn evaluate<P: Prompter>(&self, _ctx: &mut EvaluateContext<P>) -> Self::Value {
        match self {
            Condition::Always => true,
            Condition::OncePerTurn => todo!(),
            Condition::Equals(_, _) => todo!(),
            Condition::Has(_, _) => todo!(),
            Condition::NotEquals(_, _) => todo!(),
        }
    }
}

impl EvaluateEffect for Option<Condition> {
    type Value = bool;

    fn evaluate<P: Prompter>(&self, ctx: &mut EvaluateContext<P>) -> Self::Value {
        match self {
            Some(c) => c.evaluate(ctx),
            None => Condition::Always.evaluate(ctx),
        }
    }
}
