use std::collections::HashMap;
use std::ops::Deref;

use iter_tools::Itertools;

use super::effects::*;
use crate::cards::Color;
use crate::cards::*;
use crate::gameplay::Player;
use crate::gameplay::Zone;
use crate::modifiers::DamageMarkers;
use crate::modifiers::LifeTime;
use crate::modifiers::ModifierKind;
use crate::{
    gameplay::{self, *},
    modifiers::{self},
};

static VAR_THIS_CARD: &str = "&_this_card";
static VAR_LEFTOVERS: &str = "&_leftovers";
static VAR_ART_TARGET: &str = "&_art_target";
static VAR_ATTACH_TARGET: &str = "&_attach_target";

pub type EvaluateResult<T> = Result<T, GameOutcome>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LetValue {
    CardReferences(Vec<CardRef>),
    Condition(Condition),
    Number(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EvaluateContext {
    pub active_card: Option<CardRef>,
    pub active_player: Option<gameplay::Player>,
    pub variables: HashMap<String, LetValue>,
    pub is_triggered: bool,
}

impl EvaluateContext {
    pub fn for_card(&self, card: CardRef) -> Self {
        // temporary target for filters
        let mut new = self.clone();
        new.active_card = Some(card);
        new
    }
}

#[derive(Debug)]
pub struct EvaluateBuilder<'a, T> {
    effect: &'a T,
    context: EvaluateContext,
}

impl<T> EvaluateBuilder<'_, T> {
    pub fn with_card(mut self, card: CardRef, game: &Game) -> Self
    where
        Self: Sized,
    {
        // set the active card
        self.context.active_card = Some(card);
        self.context.variables.insert(
            VAR_THIS_CARD.into(),
            LetValue::CardReferences([card].into()),
        );

        // set the active player
        let player = game.player_for_card(card);
        self.context.active_player = Some(player);

        self
    }

    pub fn with_triggered(mut self, is_triggered: bool) -> Self
    where
        Self: Sized,
    {
        // triggered from an event
        self.context.is_triggered = is_triggered;

        self
    }

    pub fn with_art_target(mut self, target: CardRef) -> Self {
        // set target for arts and attachments
        self.context.variables.insert(
            VAR_ART_TARGET.into(),
            LetValue::CardReferences([target].into()),
        );
        self
    }

    pub fn with_attach_target(mut self, target: CardRef) -> Self {
        // set target for arts and attachments
        self.context.variables.insert(
            VAR_ATTACH_TARGET.into(),
            LetValue::CardReferences([target].into()),
        );
        self
    }
}

impl<T> EvaluateBuilder<'_, T>
where
    T: EvaluateEffectMut,
{
    pub async fn evaluate_mut(self, game: &mut GameDirector) -> EvaluateResult<T::Value> {
        let mut context = self.context;
        let card = context.active_card;

        if let Some(card) = card {
            game.game.event_span.open_card_span(card);
        }

        let value = self
            .effect
            .evaluate_with_context_mut(&mut context, game)
            .await;

        if let Some(card) = card {
            game.game.event_span.close_card_span(card);
        }

        game.remove_expiring_modifiers(modifiers::LifeTime::ThisEffect)
            .await?;

        value
    }
}

impl<T> EvaluateBuilder<'_, T>
where
    T: EvaluateEffect,
{
    pub fn evaluate(self, game: &Game) -> T::Value {
        self.effect.evaluate_with_context(&self.context, game)
    }
}

#[allow(async_fn_in_trait)]
pub trait EvaluateEffectMut {
    type Value;

    fn ctx(&self) -> EvaluateBuilder<Self>
    where
        Self: Sized,
    {
        EvaluateBuilder {
            effect: self,
            context: Default::default(),
        }
    }

    async fn evaluate_with_context_mut(
        &self,
        ctx: &mut EvaluateContext,
        game: &mut GameDirector,
    ) -> EvaluateResult<Self::Value>;
}
pub trait EvaluateEffect {
    type Value;

    fn ctx(&self) -> EvaluateBuilder<Self>
    where
        Self: Sized,
    {
        EvaluateBuilder {
            effect: self,
            context: Default::default(),
        }
    }

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &Game) -> Self::Value;
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

impl<I, E, V> EvaluateEffectMut for I
where
    I: Deref<Target = [E]>,
    E: EvaluateEffectMut<Value = V>,
    V: CombineEffect + Default,
{
    type Value = V;

    async fn evaluate_with_context_mut(
        &self,
        ctx: &mut EvaluateContext,
        game: &mut GameDirector,
    ) -> EvaluateResult<Self::Value> {
        let mut acc: Option<Self::Value> = None;
        for eval in self.iter() {
            acc = if let Some(acc) = acc {
                Some(acc.combine_effect(eval.evaluate_with_context_mut(ctx, game).await?))
            } else {
                Some(eval.evaluate_with_context_mut(ctx, game).await?)
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

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &Game) -> Self::Value {
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

///////////////////////////////////////

impl EvaluateEffectMut for Action {
    type Value = ();

    async fn evaluate_with_context_mut(
        &self,
        ctx: &mut EvaluateContext,
        game: &mut GameDirector,
    ) -> EvaluateResult<Self::Value> {
        match self {
            Action::AddGlobalModifier(player, modifier, life_time) => {
                game.add_zone_modifier(
                    player.evaluate_with_context(ctx, &game.game),
                    Zone::All,
                    modifier.evaluate_with_context(ctx, &game.game),
                    life_time.evaluate_with_context(ctx, &game.game),
                )
                .await?;
            }
            Action::AddModifier(cards, modifier, life_time) => {
                for card in cards.evaluate_with_context(ctx, &game.game) {
                    game.add_modifier(
                        card,
                        modifier.evaluate_with_context(ctx, &game.game),
                        life_time.evaluate_with_context(ctx, &game.game),
                    )
                    .await?;
                }
            }
            Action::AddZoneModifier(zone, modifier, life_time) => {
                let (player, zone) = zone.evaluate_with_context(ctx, &game.game);
                game.add_zone_modifier(
                    player,
                    zone,
                    modifier.evaluate_with_context(ctx, &game.game),
                    life_time.evaluate_with_context(ctx, &game.game),
                )
                .await?;
            }
            Action::AttachCards(attachments, target) => {
                let attachments = attachments.evaluate_with_context(ctx, &game.game);
                let target = target.evaluate_with_context(ctx, &game.game);
                if attachments.is_empty() {
                    return Ok(());
                }
                game.attach_cards_to_card(attachments, target).await?;
            }
            Action::DealDamage(targets, amount) => {
                let card = ctx.active_card.expect("there should be an active card");
                let targets = targets.evaluate_with_context(ctx, &game.game);
                let amount = amount.evaluate_with_context(ctx, &game.game);
                game.deal_damage(card, targets, DamageMarkers::from_hp(amount as u16), false)
                    .await?;
            }
            Action::DealSpecialDamage(targets, amount) => {
                let card = ctx.active_card.expect("there should be an active card");
                let targets = targets.evaluate_with_context(ctx, &game.game);
                let amount = amount.evaluate_with_context(ctx, &game.game);
                game.deal_damage(card, targets, DamageMarkers::from_hp(amount as u16), true)
                    .await?;
            }
            Action::Draw(amount) => {
                game.draw_from_main_deck(
                    ctx.active_player
                        .expect("there should be an active player to draw"),
                    amount.evaluate_with_context(ctx, &game.game),
                )
                .await?;
            }
            Action::If(condition, actions) => {
                if condition.evaluate_with_context(ctx, &game.game) {
                    Box::pin(actions.evaluate_with_context_mut(ctx, game)).await?;
                }
            }
            Action::KnockOut(cards) => {
                let cards = cards.evaluate_with_context(ctx, &game.game);
                game.knock_out_members(cards).await?;
            }
            Action::LetCardReferences(let_card) => {
                let value =
                    LetValue::CardReferences(let_card.1.evaluate_with_context(ctx, &game.game));
                // dbg!(&let_card.0, &value, &ctx);
                ctx.variables.insert(let_card.0 .0.clone(), value);
            }
            Action::LetCondition(let_cond) => {
                let value = LetValue::Condition(let_cond.1.clone());
                // dbg!(&let_cond.0, &value, &ctx);
                ctx.variables.insert(let_cond.0 .0.clone(), value);
            }
            Action::LetSelect(let_select) => {
                let value = let_select.1.evaluate_with_context_mut(ctx, game).await?;
                // dbg!(&let_select.0, &value, &ctx);
                ctx.variables.insert(let_select.0 .0.clone(), value);
            }
            Action::LetNumber(let_value) => {
                let value = LetValue::Number(let_value.1.evaluate_with_context(ctx, &game.game));
                // dbg!(&let_value.0, &value, &ctx);
                ctx.variables.insert(let_value.0 .0.clone(), value);
            }
            Action::Noop => {}
            Action::Reveal(cards) => {
                let cards = cards.evaluate_with_context(ctx, &game.game);
                let map: HashMap<(Player, Zone), Vec<CardRef>> =
                    game.group_by_player_and_zone(&cards);
                for ((player, zone), cards) in map {
                    game.reveal_cards(player, zone, &cards).await?;
                }
            }
            Action::SendTo(to_zone, cards) => {
                let (_, to_zone) = to_zone.evaluate_with_context(ctx, &game.game);
                let cards = cards.evaluate_with_context(ctx, &game.game);
                game.send_to_zone(cards, to_zone).await?;
            }
            Action::SendToBottom(to_zone, cards) => {
                let (_, to_zone) = to_zone.evaluate_with_context(ctx, &game.game);
                let cards = cards.evaluate_with_context(ctx, &game.game);
                game.send_to_zone_with_location(cards, to_zone, ZoneAddLocation::Bottom)
                    .await?;
            }
            Action::SendToTop(to_zone, cards) => {
                let (_, to_zone) = to_zone.evaluate_with_context(ctx, &game.game);
                let cards = cards.evaluate_with_context(ctx, &game.game);
                game.send_to_zone_with_location(cards, to_zone, ZoneAddLocation::Top)
                    .await?;
            }
            Action::Shuffle(zone) => {
                let (player, zone) = zone.evaluate_with_context(ctx, &game.game);
                game.shuffle_decks(vec![(player, zone)]).await?;
            }
        }
        Ok(())
    }
}

impl EvaluateEffect for CardReference {
    type Value = CardRef;

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &Game) -> Self::Value {
        match self {
            CardReference::ArtTarget => {
                CardReference::Var(Var(VAR_ART_TARGET.into())).evaluate_with_context(ctx, game)
            }
            CardReference::AttachTarget => {
                CardReference::Var(Var(VAR_ATTACH_TARGET.into())).evaluate_with_context(ctx, game)
            }
            CardReference::EventOrigin => game
                .event_span
                .event_origin_for_evaluate(ctx)
                .expect("there should be an event origin card"),

            CardReference::ThisCard => {
                CardReference::Var(Var(VAR_THIS_CARD.into())).evaluate_with_context(ctx, game)
            }
            CardReference::Var(var) => {
                match ctx.variables.get(&var.0).unwrap_or_else(|| {
                    panic!("the variable should exist: {:?} - ctx: {:?}", var, ctx)
                }) {
                    LetValue::CardReferences(cards) => {
                        if cards.len() > 1 {
                            panic!("more than one card")
                        }
                        if cards.is_empty() {
                            panic!("no cards")
                        }
                        *cards.first().expect("there should be a card")
                    }
                    _ => panic!("wrong value: {:?} - ctx: {:?}", var, ctx),
                }
            }
        }
    }
}

impl EvaluateEffect for CardReferences {
    type Value = Vec<CardRef>;

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &Game) -> Self::Value {
        match self {
            CardReferences::ArtTarget => {
                CardReferences::Var(Var(VAR_ART_TARGET.into())).evaluate_with_context(ctx, game)
            }
            CardReferences::AttachedTo(card) => {
                let card = card.evaluate_with_context(ctx, game);
                game.board_for_card(card).attachments(card)
            }
            CardReferences::AttachTarget => {
                CardReferences::Var(Var(VAR_ATTACH_TARGET.into())).evaluate_with_context(ctx, game)
            }
            CardReferences::EventOrigin => {
                vec![game
                    .event_span
                    .event_origin_for_evaluate(ctx)
                    .expect("there should be an event origin card")]
            }
            CardReferences::From(zone) => {
                let (player, zone) = zone.evaluate_with_context(ctx, game);
                game.board(player).all_cards(zone)
            }
            CardReferences::FromTop(amount, zone) => {
                let amount = amount.evaluate_with_context(ctx, game);
                let (player, zone) = zone.evaluate_with_context(ctx, game);
                game.board(player).get_zone(zone).peek_top_cards(amount)
            }
            CardReferences::Leftovers => {
                CardReferences::Var(Var(VAR_LEFTOVERS.into())).evaluate_with_context(ctx, game)
            }
            CardReferences::ThisCard => {
                CardReferences::Var(Var(VAR_THIS_CARD.into())).evaluate_with_context(ctx, game)
            }
            CardReferences::Var(var) => {
                match ctx.variables.get(&var.0).unwrap_or_else(|| {
                    panic!("the variable should exist: {:?} - ctx: {:?}", var, ctx)
                }) {
                    LetValue::CardReferences(cards) => cards.clone(),
                    _ => panic!("wrong value: {:?} - ctx: {:?}", var, ctx),
                }
            }
            CardReferences::Filter(cards, condition) => {
                let cards = cards.evaluate_with_context(ctx, game);
                cards
                    .into_iter()
                    .filter(|c| condition.evaluate_with_context(&ctx.for_card(*c), game))
                    .collect_vec()
            }
        }
    }
}

impl EvaluateEffect for Condition {
    type Value = bool;

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &Game) -> Self::Value {
        match self {
            Condition::All(cards, condition) => {
                let cards = cards.evaluate_with_context(ctx, game);
                cards
                    .into_iter()
                    .all(|c| -> bool { condition.evaluate_with_context(&ctx.for_card(c), game) })
            }
            Condition::And(condition_1, condition_2) => {
                let condition_1 = condition_1.evaluate_with_context(ctx, game);
                let condition_2 = condition_2.evaluate_with_context(ctx, game);
                condition_1 && condition_2
            }
            Condition::Any(cards, condition) => {
                let cards = cards.evaluate_with_context(ctx, game);
                cards
                    .into_iter()
                    .any(|c| condition.evaluate_with_context(&ctx.for_card(c), game))
            }
            Condition::Anything => true,
            Condition::Equals(value_1, value_2) => {
                let value_1 = value_1.evaluate_with_context(ctx, game);
                let value_2 = value_2.evaluate_with_context(ctx, game);
                value_1 == value_2
            }
            Condition::Exists(cards) => {
                let cards = cards.evaluate_with_context(ctx, game);
                !cards.is_empty()
            }
            Condition::False => false,
            Condition::GreaterThanEquals(value_1, value_2) => {
                let value_1 = value_1.evaluate_with_context(ctx, game);
                let value_2 = value_2.evaluate_with_context(ctx, game);
                value_1 >= value_2
            }
            Condition::HasCheers => {
                let card = ctx.active_card.expect("there should be an active card");
                game.attached_cheers(card).next().is_some()
            }
            Condition::IsAttributeBuzz => {
                let card = ctx.active_card.expect("there should be an active card");
                game.lookup_card(card)
                    .is_attribute(HoloMemberExtraAttribute::Buzz)
            }
            Condition::IsColor(color) => {
                let color = color.evaluate_with_context(ctx, game);
                let card = ctx.active_card.expect("there should be an active card");
                game.lookup_card(card).is_color(color, card, game)
            }
            Condition::IsCheer => {
                let card = ctx.active_card.expect("there should be an active card");
                game.lookup_card(card).is_cheer(card, game)
            }
            Condition::IsEven(value) => {
                let value = value.evaluate_with_context(ctx, game);
                value % 2 == 0
            }
            Condition::IsInZone(zone) => {
                let (player, zone) = zone.evaluate_with_context(ctx, game);
                let card = ctx.active_card.expect("there should be an active card");
                game.board(player).find_card_zone(card) == Some(zone)
            }
            Condition::IsLevelFirst => {
                let card = ctx.active_card.expect("there should be an active card");
                game.lookup_card(card).is_level(HoloMemberLevel::First)
            }
            Condition::IsLevelSecond => {
                let card = ctx.active_card.expect("there should be an active card");
                game.lookup_card(card).is_level(HoloMemberLevel::Second)
            }
            Condition::IsMember => {
                let card = ctx.active_card.expect("there should be an active card");
                game.lookup_card(card).is_member()
            }
            Condition::IsNamedAzki => {
                let card = ctx.active_card.expect("there should be an active card");
                game.lookup_card(card).is_named("AZKi")
            }
            Condition::IsNamedOmaruPolka => {
                let card = ctx.active_card.expect("there should be an active card");
                game.lookup_card(card).is_named("Omaru Polka")
            }
            Condition::IsNamedTokinoSora => {
                let card = ctx.active_card.expect("there should be an active card");
                game.lookup_card(card).is_named("Tokino Sora")
            }
            Condition::IsNamedUsadaPekora => {
                let card = ctx.active_card.expect("there should be an active card");
                game.lookup_card(card).is_named("Usada Pekora")
            }
            Condition::IsCard(is_card) => {
                let is_card = is_card.evaluate_with_context(ctx, game);
                let card = ctx.active_card.expect("there should be an active card");
                card == is_card
            }
            Condition::IsNotCard(not_card) => {
                let not_card = not_card.evaluate_with_context(ctx, game);
                let card = ctx.active_card.expect("there should be an active card");
                card != not_card
            }
            Condition::IsOdd(value) => {
                let value = value.evaluate_with_context(ctx, game);
                value % 2 == 1
            }
            Condition::IsSupportLimited => {
                let card = ctx.active_card.expect("there should be an active card");
                game.lookup_card(card).is_support_limited()
            }
            Condition::LessThanEquals(value_1, value_2) => {
                let value_1 = value_1.evaluate_with_context(ctx, game);
                let value_2 = value_2.evaluate_with_context(ctx, game);
                value_1 <= value_2
            }
            Condition::Not(condition) => {
                let condition = condition.evaluate_with_context(ctx, game);
                !condition
            }
            Condition::Or(condition_1, condition_2) => {
                let condition_1 = condition_1.evaluate_with_context(ctx, game);
                let condition_2 = condition_2.evaluate_with_context(ctx, game);
                condition_1 || condition_2
            }
            Condition::True => true,
            Condition::Var(var) => {
                match ctx.variables.get(&var.0).unwrap_or_else(|| {
                    panic!("the variable should exist: {:?} - ctx: {:?}", var, ctx)
                }) {
                    LetValue::Condition(condition) => condition.evaluate_with_context(ctx, game),
                    _ => panic!("wrong value: {:?} - ctx: {:?}", var, ctx),
                }
            }
            Condition::Yours => {
                let card = ctx.active_card.expect("there should be an active card");
                game.player_for_card(card)
                    == ctx.active_player.expect("there should be an active player")
            }
        }
    }
}

impl EvaluateEffectMut for super::LetValue {
    type Value = LetValue;

    async fn evaluate_with_context_mut(
        &self,
        ctx: &mut EvaluateContext,
        game: &mut GameDirector,
    ) -> EvaluateResult<Self::Value> {
        match self {
            super::LetValue::OptionalActivate => {
                let player = ctx.active_player.expect("there should be an active player");
                Ok(LetValue::Condition(
                    game.prompt_for_optional_activate(player).await.into(),
                ))
            }
            super::LetValue::RollDice => {
                let player = ctx.active_player.expect("there should be an active player");
                let number = game.roll_dice(player).await?;
                Ok(LetValue::Number(number as usize))
            }
            super::LetValue::SelectAny(cards, condition) => {
                let player = ctx.active_player.expect("there should be an active player");
                let cards = cards.evaluate_with_context(ctx, &game.game);
                let choice = game
                    .prompt_for_select(
                        player,
                        cards.clone(),
                        condition.as_ref().clone(),
                        ctx,
                        0,
                        usize::MAX,
                    )
                    .await;
                let leftovers = cards
                    .into_iter()
                    .filter(|c| !choice.contains(c))
                    .collect_vec();
                ctx.variables
                    .insert(VAR_LEFTOVERS.into(), LetValue::CardReferences(leftovers));
                Ok(LetValue::CardReferences(choice))
            }
            super::LetValue::SelectOne(cards, condition) => {
                let player = ctx.active_player.expect("there should be an active player");
                let cards = cards.evaluate_with_context(ctx, &game.game);
                let choice = game
                    .prompt_for_select(player, cards.clone(), condition.as_ref().clone(), ctx, 1, 1)
                    .await;
                let leftovers = cards
                    .into_iter()
                    .filter(|c| !choice.contains(c))
                    .collect_vec();
                ctx.variables
                    .insert(VAR_LEFTOVERS.into(), LetValue::CardReferences(leftovers));
                Ok(LetValue::CardReferences(choice))
            }
            super::LetValue::SelectNumberBetween(min, max) => {
                let player = ctx.active_player.expect("there should be an active player");
                let min = min.evaluate_with_context(ctx, &game.game);
                let max = max.evaluate_with_context(ctx, &game.game);
                Ok(LetValue::Number(
                    game.prompt_for_number(player, min, max).await,
                ))
            }
            super::LetValue::SelectUpTo(amount, cards, condition) => {
                let player = ctx.active_player.expect("there should be an active player");
                let amount = amount.evaluate_with_context(ctx, &game.game);
                let cards = cards.evaluate_with_context(ctx, &game.game);
                let choice = game
                    .prompt_for_select(
                        player,
                        cards.clone(),
                        condition.as_ref().clone(),
                        ctx,
                        0,
                        amount,
                    )
                    .await;
                let leftovers = cards
                    .into_iter()
                    .filter(|c| !choice.contains(c))
                    .collect_vec();
                ctx.variables
                    .insert(VAR_LEFTOVERS.into(), LetValue::CardReferences(leftovers));
                Ok(LetValue::CardReferences(choice))
            }
        }
    }
}

impl EvaluateEffect for super::LifeTime {
    type Value = LifeTime;

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &Game) -> Self::Value {
        match self {
            super::LifeTime::ThisGame => LifeTime::ThisGame,
            super::LifeTime::ThisTurn => LifeTime::ThisTurn,
            super::LifeTime::NextTurn(player) => {
                let player = player.evaluate_with_context(ctx, game);
                LifeTime::NextTurn(player)
            }
            super::LifeTime::ThisStep => LifeTime::ThisStep,
            super::LifeTime::ThisArt => LifeTime::ThisArt,
            super::LifeTime::ThisEffect => LifeTime::ThisEffect,
            super::LifeTime::UntilRemoved => LifeTime::UntilRemoved,
            super::LifeTime::WhileAttached(card) => {
                let card = card.evaluate_with_context(ctx, game);
                LifeTime::WhileAttached(card)
            }
        }
    }
}

impl EvaluateEffect for Modifier {
    type Value = ModifierKind;

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &Game) -> Self::Value {
        match self {
            Modifier::AsArtCost(amount, color) => {
                let amount = amount.evaluate_with_context(ctx, game);
                let color = color.evaluate_with_context(ctx, game);
                ModifierKind::AsArtCost(color, amount)
            }
            Modifier::AsCheer(amount, color) => {
                let amount = amount.evaluate_with_context(ctx, game);
                let color = color.evaluate_with_context(ctx, game);
                ModifierKind::AsCheer(color, amount)
            }
            Modifier::DealLessDamage(amount) => {
                let amount = amount.evaluate_with_context(ctx, game);
                ModifierKind::DealLessDamage(amount)
            }
            Modifier::DealMoreDamage(amount) => {
                let amount = amount.evaluate_with_context(ctx, game);
                ModifierKind::DealMoreDamage(amount)
            }
            Modifier::ReceiveLessDamage(amount) => {
                let amount = amount.evaluate_with_context(ctx, game);
                ModifierKind::ReceiveLessDamage(amount)
            }
            Modifier::ReceiveMoreDamage(amount) => {
                let amount = amount.evaluate_with_context(ctx, game);
                ModifierKind::ReceiveMoreDamage(amount)
            }
            Modifier::NextDiceRoll(number) => {
                let number = number.evaluate_with_context(ctx, game);
                ModifierKind::NextDiceRoll(number)
            }
            Modifier::NoLifeLoss => ModifierKind::NoLifeLoss,
            Modifier::When(condition, modifier) => {
                let modifier = modifier.evaluate_with_context(ctx, game);
                ModifierKind::Conditional(condition.clone(), Box::new(modifier))
            }
        }
    }
}

impl EvaluateEffect for super::Player {
    type Value = Player;

    fn evaluate_with_context(&self, ctx: &EvaluateContext, _game: &Game) -> Self::Value {
        let player = ctx.active_player.expect("there should be an active player");
        let opponent = match player {
            Player::One => Player::Two,
            Player::Two => Player::One,
            Player::Both => unreachable!("cannot be bot"),
        };
        match self {
            super::Player::You => player,
            super::Player::Opponent => opponent,
        }
    }
}

impl EvaluateEffect for Number {
    type Value = usize;

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &Game) -> Self::Value {
        match self {
            Number::Count(cards) => {
                let cards = cards.evaluate_with_context(ctx, game);
                cards.len()
            }
            Number::Literal(number) => number.0,
            Number::Minus(a, b) => {
                let a = a.evaluate_with_context(ctx, game);
                let b = b.evaluate_with_context(ctx, game);
                a - b
            }
            Number::Multiply(a, b) => {
                let a = a.evaluate_with_context(ctx, game);
                let b = b.evaluate_with_context(ctx, game);
                a * b
            }
            Number::Plus(a, b) => {
                let a = a.evaluate_with_context(ctx, game);
                let b = b.evaluate_with_context(ctx, game);
                a + b
            }
            Number::Var(var) => {
                match ctx.variables.get(&var.0).unwrap_or_else(|| {
                    panic!("the variable should exist: {:?} - ctx: {:?}", var, ctx)
                }) {
                    LetValue::Number(value) => *value,
                    _ => panic!("wrong value: {:?} - ctx: {:?}", var, ctx),
                }
            }
            Number::DamageAmount => {
                let card = ctx.active_card.expect("there should be an active card");
                game.get_damage(card).to_hp() as usize
            }
            Number::HealthPointAmount => {
                let card = ctx.active_card.expect("there should be an active card");
                game.remaining_hp(card) as usize
            }
        }
    }
}

impl EvaluateEffect for super::Zone {
    type Value = (Player, Zone);

    fn evaluate_with_context(&self, ctx: &EvaluateContext, _game: &Game) -> Self::Value {
        let player = ctx.active_player.expect("there should be an active player");
        let opponent = match player {
            Player::One => Player::Two,
            Player::Two => Player::One,
            Player::Both => unreachable!("cannot be bot"),
        };
        match self {
            super::Zone::Archive => (player, Zone::Archive),
            super::Zone::BackStage => (player, Zone::BackStage),
            super::Zone::CenterStage => (player, Zone::CenterStage),
            super::Zone::CheerDeck => (player, Zone::CheerDeck),
            super::Zone::Hand => (player, Zone::Hand),
            super::Zone::HoloPower => (player, Zone::HoloPower),
            super::Zone::MainDeck => (player, Zone::MainDeck),
            super::Zone::MainStage => (player, Zone::MainStage),
            super::Zone::OpponentBackStage => (opponent, Zone::BackStage),
            super::Zone::OpponentCenterStage => (opponent, Zone::CenterStage),
            super::Zone::Stage => (player, Zone::Stage),
        }
    }
}

impl EvaluateEffect for super::Color {
    type Value = Color;

    fn evaluate_with_context(&self, _ctx: &EvaluateContext, _game: &Game) -> Self::Value {
        match self {
            super::Color::White => Color::White,
            super::Color::Green => Color::Green,
            super::Color::Red => Color::Red,
            super::Color::Blue => Color::Blue,
            super::Color::Purple => Color::Purple,
            super::Color::Yellow => Color::Yellow,
            super::Color::Colorless => Color::Colorless,
        }
    }
}
