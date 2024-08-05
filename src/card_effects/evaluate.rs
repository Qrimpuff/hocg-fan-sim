use std::ops::Deref;

use super::effects::*;
use crate::{gameplay::*, modifiers::DamageMarkers, HoloMemberHp};

// TODO clean up this file after the list of effect is finalized

pub struct EvaluateContext {
    active_card: Option<CardRef>,
    active_player: Option<Player>,
    card_target: Option<CardRef>,
    player_target: Option<Player>,
}

impl EvaluateContext {
    pub fn new() -> Self {
        EvaluateContext {
            active_card: None,
            active_player: None,
            card_target: None,
            player_target: None,
        }
    }
    pub fn with_card(card: CardRef, game: &Game) -> Self {
        let player = game.player_for_card(card);
        EvaluateContext {
            active_card: Some(card),
            active_player: Some(player),
            card_target: Some(card),
            player_target: Some(player),
        }
    }
}

pub trait CombineEffect {
    fn combine_effect(self, other: Self) -> Self;
}

impl CombineEffect for () {
    fn combine_effect(self, _other: Self) -> Self {}
}
impl CombineEffect for bool {
    fn combine_effect(self, other: Self) -> Self {
        self && other
    }
}

pub type EvaluateResult<T> = Result<T, GameOutcome>;

pub trait EvaluateEffectMut {
    type Value;

    fn evaluate_with_context_mut(
        &self,
        ctx: &mut EvaluateContext,
        game: &mut Game,
    ) -> EvaluateResult<Self::Value>;

    fn evaluate_mut(&self, game: &mut Game) -> EvaluateResult<Self::Value>
    where
        Self: Sized,
    {
        self.evaluate_with_context_mut(&mut EvaluateContext::new(), game)
    }
    fn evaluate_with_card_mut(&self, game: &mut Game, card: CardRef) -> EvaluateResult<Self::Value>
    where
        Self: Sized,
    {
        self.evaluate_with_context_mut(&mut EvaluateContext::with_card(card, game), game)
    }
}
pub trait EvaluateEffect {
    type Value;

    fn evaluate_with_context(&self, ctx: &mut EvaluateContext, game: &Game) -> Self::Value;

    fn evaluate(&self, game: &Game) -> Self::Value
    where
        Self: Sized,
    {
        self.evaluate_with_context(&mut EvaluateContext::new(), game)
    }
    fn evaluate_with_card(&self, game: &Game, card: CardRef) -> Self::Value
    where
        Self: Sized,
    {
        self.evaluate_with_context(&mut EvaluateContext::with_card(card, game), game)
    }
}

impl<I, E, V> EvaluateEffectMut for I
where
    I: Deref<Target = [E]>,
    E: EvaluateEffectMut<Value = V>,
    V: CombineEffect + Default,
{
    type Value = V;

    fn evaluate_with_context_mut(
        &self,
        ctx: &mut EvaluateContext,
        game: &mut Game,
    ) -> EvaluateResult<Self::Value> {
        let mut acc: Option<Self::Value> = None;
        for eval in self.iter() {
            acc = if let Some(acc) = acc {
                Some(acc.combine_effect(eval.evaluate_with_context_mut(ctx, game)?))
            } else {
                Some(eval.evaluate_with_context_mut(ctx, game)?)
            };
        }

        Ok(acc.unwrap_or_default())
    }
}
impl<I, E, V> EvaluateEffect for I
where
    I: Deref<Target = [E]>,
    E: EvaluateEffect<Value = V>,
    V: CombineEffect + Default,
{
    type Value = V;

    fn evaluate_with_context(&self, ctx: &mut EvaluateContext, game: &Game) -> Self::Value {
        let mut acc: Option<Self::Value> = None;
        for eval in self.iter() {
            acc = if let Some(acc) = acc {
                Some(acc.combine_effect(eval.evaluate_with_context(ctx, game)))
            } else {
                Some(eval.evaluate_with_context(ctx, game))
            };
        }

        acc.unwrap_or_default()
    }
}

impl EvaluateEffect for Target {
    type Value = CardRef;

    fn evaluate_with_context(&self, ctx: &mut EvaluateContext, game: &Game) -> Self::Value {
        match self {
            Target::CurrentCard => ctx.active_card.expect("there should be an active card"),
            Target::CenterHoloMember => game
                .board(ctx.active_player.expect("there should be an active player"))
                .get_zone(Zone::MainStageCenter)
                .peek_top_card()
                .expect("there should be a center member"),
            Target::Var(_) => todo!(),
            Target::SelectMember(_) => todo!(),
            Target::MembersOnStage => todo!(),
            Target::With(_, _) => todo!(),
            Target::SelectCheersUpTo(_, _) => todo!(),
            Target::CheersInArchive => todo!(),
        }
    }
}

impl EvaluateEffectMut for Action {
    type Value = ();

    fn evaluate_with_context_mut(
        &self,
        ctx: &mut EvaluateContext,
        game: &mut Game,
    ) -> EvaluateResult<Self::Value> {
        match self {
            Action::Noop => {
                println!("*nothing happens*")
            }
            Action::For(t, a) => {
                // FIXME only handles card for now
                let past_target = ctx.card_target;
                let target = t.evaluate_with_context(ctx, game);
                ctx.card_target = Some(target);
                a.evaluate_with_context_mut(ctx, game)?;
                ctx.card_target = past_target;
            }
            Action::Buff(_, _) => todo!(),
            Action::Debuff(_, _) => todo!(),
            Action::Heal(h) => {
                let heal = h.evaluate_with_context(ctx, game);
                let card = ctx.card_target.expect("there should be a target card");
                let mem = game
                    .lookup_holo_member(card)
                    .expect("can only heal members");

                println!("heal {} for card {}", heal, mem.name);
                game.remove_damage(card, DamageMarkers::from_hp(heal as HoloMemberHp))?;
            }
            Action::Let(_, _) => todo!(),
            Action::When(_, _) => todo!(),
            Action::Draw(d) => {
                let draw = d.evaluate_with_context(ctx, game);

                println!("draw {} card(s)", draw);
                // game.active_board_mut().draw(draw as usize);
                game.draw_from_main_deck(
                    ctx.player_target.expect("there should be an active player"),
                    draw as usize,
                )?;
            }
            Action::NextDiceNumber(_) => todo!(),
            Action::Attach(_) => todo!(),
        };
        Ok(())
    }
}

impl EvaluateEffect for Value {
    type Value = u32;

    #[allow(clippy::only_used_in_recursion)]
    fn evaluate_with_context(&self, ctx: &mut EvaluateContext, game: &Game) -> Self::Value {
        match self {
            Value::For(_, _) => todo!(),
            Value::Get(_) => todo!(),
            Value::Number(n) => n.0,
            Value::Var(_) => todo!(),
            Value::Add(a, b) => {
                a.evaluate_with_context(ctx, game) + b.evaluate_with_context(ctx, game)
            }
            Value::Subtract(a, b) => {
                a.evaluate_with_context(ctx, game) - b.evaluate_with_context(ctx, game)
            }
            Value::Multiply(a, b) => {
                a.evaluate_with_context(ctx, game) * b.evaluate_with_context(ctx, game)
            }
            Value::SelectDiceNumber => todo!(),
            Value::All => u32::MAX,
        }
    }
}

impl EvaluateEffect for Condition {
    type Value = bool;

    #[allow(clippy::only_used_in_recursion)]
    fn evaluate_with_context(&self, ctx: &mut EvaluateContext, game: &Game) -> Self::Value {
        match self {
            Condition::Always => true,
            Condition::OncePerTurn => todo!(),
            Condition::Equals(_, _) => todo!(),
            Condition::Has(_, _) => todo!(),
            Condition::NotEquals(_, _) => todo!(),
            Condition::And(a, b) => {
                a.evaluate_with_context(ctx, game) && b.evaluate_with_context(ctx, game)
            }
            Condition::Or(a, b) => {
                a.evaluate_with_context(ctx, game) || b.evaluate_with_context(ctx, game)
            }

            Condition::IsHoloMember => todo!(),
            Condition::OncePerGame => todo!(),
        }
    }
}

// impl EvaluateEffect for Option<Condition> {
//     type Value = bool;

//     fn evaluate(&self, ctx: &mut EvaluateContext) -> Self::Value {
//         match self {
//             Some(c) => c.evaluate(ctx),
//             None => Condition::Always.evaluate(ctx),
//         }
//     }
// }
