use std::collections::HashMap;
use std::ops::Deref;

use iter_tools::Itertools;

use super::effects::*;
use crate::cards::*;
use crate::events::Shuffle;
use crate::gameplay::Player;
use crate::gameplay::Zone;
use crate::modifiers::LifeTime;
use crate::modifiers::ModifierKind;
use crate::{
    events::Event,
    gameplay::{self, *},
    modifiers::{self},
};

static VAR_THIS_CARD: &str = "&_this_card";
static VAR_LEFTOVERS: &str = "&_leftovers";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EvaluateContext {
    pub active_card: Option<CardRef>,
    pub active_player: Option<gameplay::Player>,
    // pub card_target: Option<CardRef>,
    // pub player_target: Option<gameplay::Player>,
    pub variables: HashMap<String, LetValue>,
    // pub event: Option<&'a Event>,
    pub is_triggered: bool,
}

impl EvaluateContext {
    pub fn new() -> Self {
        EvaluateContext {
            active_card: None,
            active_player: None,
            // card_target: None,
            // player_target: None,
            variables: HashMap::new(),
            // event: None,
            is_triggered: false,
        }
    }
    pub fn with_card(card: CardRef, game: &GameState, is_trigger: bool) -> Self {
        let player = game.player_for_card(card);
        let mut variables = HashMap::new();
        variables.insert(
            VAR_THIS_CARD.into(),
            LetValue::CardReferences([card].into()),
        );
        EvaluateContext {
            active_card: Some(card),
            active_player: Some(player),
            // card_target: Some(card),
            // player_target: Some(player),
            variables,
            // event: None,
            is_triggered: is_trigger,
        }
    }

    pub fn for_card(&self, card: CardRef) -> Self {
        let mut new = self.clone();
        new.active_card = Some(card);
        new
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LetValue {
    CardReferences(Vec<CardRef>),
    Condition(Condition),
    Number(usize),
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

    fn evaluate_with_card_mut(
        &self,
        game: &mut Game,
        card: CardRef,
        is_triggered: bool,
    ) -> EvaluateResult<Self::Value>
    where
        Self: Sized,
    {
        game.state.event_span.open_card_span(card);
        let value = self
            .evaluate_with_context_mut(&mut EvaluateContext::with_card(card, &game.state, is_triggered), game);
        game.state.event_span.close_card_span(card);

        game.remove_expiring_modifiers(None, modifiers::LifeTime::ThisEffect)?;

        value
    }
    // fn evaluate_with_card_event_mut(
    //     &self,
    //     game: &mut Game,
    //     card: CardRef,
    //     event: &Event,
    // ) -> EvaluateResult<Self::Value>
    // where
    //     Self: Sized,
    // {
    //     game.state.event_span.open_card_span(card);
    //     let mut ctx = EvaluateContext::with_card(card, &game.state);
    //     ctx.event = Some(event);
    //     let value = self.evaluate_with_context_mut(&mut ctx, game);
    //     game.state.event_span.close_card_span(card);

    //     game.remove_expiring_modifiers(None, modifiers::LifeTime::ThisEffect)?;

    //     value
    // }
}
pub trait EvaluateEffect {
    type Value;

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &GameState) -> Self::Value;

    fn evaluate_with_card(&self, game: &GameState, card: CardRef, is_triggered: bool) -> Self::Value
    where
        Self: Sized,
    {
        self.evaluate_with_context(&EvaluateContext::with_card(card, game, is_triggered), game)
    }
    // fn evaluate_with_card_event(
    //     &self,
    //     game: &GameState,
    //     card: CardRef,
    //     event: &Event,
    // ) -> Self::Value
    // where
    //     Self: Sized,
    // {
    //     let mut ctx = EvaluateContext::with_card(card, game);
    //     ctx.event = Some(event);
    //     self.evaluate_with_context(&ctx, game)
    // }
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

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &GameState) -> Self::Value {
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

    fn evaluate_with_context_mut(
        &self,
        ctx: &mut EvaluateContext,
        game: &mut Game,
    ) -> EvaluateResult<Self::Value> {
        match self {
            Action::AddGlobalModifier(player, modifier, life_time) => {
                game.add_zone_modifier(
                    ctx.active_card,
                    player.evaluate_with_context(ctx, &game.state),
                    Zone::All,
                    modifier.evaluate_with_context(ctx, &game.state),
                    life_time.evaluate_with_context(ctx, &game.state),
                )?;
            }
            Action::AddModifier(cards, modifier, life_time) => {
                for card in cards.evaluate_with_context(ctx, &game.state) {
                    game.add_modifier(
                        ctx.active_card,
                        card,
                        modifier.evaluate_with_context(ctx, &game.state),
                        life_time.evaluate_with_context(ctx, &game.state),
                    )?;
                }
            }
            Action::AddZoneModifier(zone, modifier, life_time) => {
                let (player, zone) = zone.evaluate_with_context(ctx, &game.state);
                game.add_zone_modifier(
                    ctx.active_card,
                    player,
                    zone,
                    modifier.evaluate_with_context(ctx, &game.state),
                    life_time.evaluate_with_context(ctx, &game.state),
                )?;
            }
            Action::AttachCards(attachments, target) => {
                let attachments = attachments.evaluate_with_context(ctx, &game.state);
                let target = target.evaluate_with_context(ctx, &game.state);
                if attachments.is_empty() {
                    return Ok(());
                }
                let player = game
                    .player_for_card(*attachments.first().expect("should have at least one card"));
                game.attach_cards_to_card(ctx.active_card, player, attachments, target)?;
            }
            Action::Draw(amount) => {
                game.draw_from_main_deck(
                    ctx.active_card,
                    ctx.active_player
                        .expect("there should be an active player to draw"),
                    amount.evaluate_with_context(ctx, &game.state),
                )?;
            }
            Action::If(condition, actions) => {
                if condition.evaluate_with_context(ctx, &game.state) {
                    actions.evaluate_with_context_mut(ctx, game)?;
                }
            }
            Action::LetCardReferences(let_card) => {
                let value =
                    LetValue::CardReferences(let_card.1.evaluate_with_context(ctx, &game.state));
                // dbg!(&let_card.0, &value, &ctx);
                ctx.variables.insert(let_card.0 .0.clone(), value);
            }
            Action::LetCondition(let_cond) => {
                let value = LetValue::Condition(let_cond.1.clone());
                // dbg!(&let_cond.0, &value, &ctx);
                ctx.variables.insert(let_cond.0 .0.clone(), value);
            }
            Action::LetSelect(let_select) => {
                let value = let_select.1.evaluate_with_context_mut(ctx, game)?;
                // dbg!(&let_select.0, &value, &ctx);
                ctx.variables.insert(let_select.0 .0.clone(), value);
            }
            Action::LetNumber(let_value) => {
                let value = LetValue::Number(let_value.1.evaluate_with_context(ctx, &game.state));
                // dbg!(&let_value.0, &value, &ctx);
                ctx.variables.insert(let_value.0 .0.clone(), value);
            }
            Action::Noop => {}
            Action::Reveal(cards) => {
                let cards = cards.evaluate_with_context(ctx, &game.state);
                let map: HashMap<(Player, Zone), Vec<CardRef>> =
                    game.group_by_player_and_zone(cards);
                for ((player, zone), cards) in map {
                    game.reveal_cards(ctx.active_card, player, zone, &cards)?;
                }
            }
            Action::SendTo(to_zone, cards) => {
                let (_, to_zone) = to_zone.evaluate_with_context(ctx, &game.state);
                let cards = cards.evaluate_with_context(ctx, &game.state);
                if let Some(c) = cards.first() {
                    let player = game.player_for_card(*c);
                    game.send_cards_to_zone(
                        ctx.active_card,
                        player,
                        cards,
                        to_zone,
                        to_zone.default_add_location(),
                    )?;
                }
            }
            Action::SendToBottom(to_zone, cards) => {
                let (_, to_zone) = to_zone.evaluate_with_context(ctx, &game.state);
                let cards = cards.evaluate_with_context(ctx, &game.state);
                if let Some(c) = cards.first() {
                    let player = game.player_for_card(*c);
                    game.send_cards_to_zone(
                        ctx.active_card,
                        player,
                        cards,
                        to_zone,
                        ZoneAddLocation::Bottom,
                    )?;
                }
            }
            Action::SendToTop(to_zone, cards) => {
                let (_, to_zone) = to_zone.evaluate_with_context(ctx, &game.state);
                let cards = cards.evaluate_with_context(ctx, &game.state);
                if let Some(c) = cards.first() {
                    let player = game.player_for_card(*c);
                    game.send_cards_to_zone(
                        ctx.active_card,
                        player,
                        cards,
                        to_zone,
                        ZoneAddLocation::Top,
                    )?;
                }
            }
            Action::Shuffle(zone) => {
                let (player, zone) = zone.evaluate_with_context(ctx, &game.state);
                game.send_event(ctx.active_card, Shuffle { player, zone }.into())?;
            }
        }
        Ok(())
    }
}

impl EvaluateEffect for CardReference {
    type Value = CardRef;

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &GameState) -> Self::Value {
        match self {
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

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &GameState) -> Self::Value {
        match self {
            CardReferences::Attached(card) => {
                let card = card.evaluate_with_context(ctx, game);
                game.board_for_card(card).attachments(card)
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

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &GameState) -> Self::Value {
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
            Condition::Exist(cards) => {
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
            Condition::IsColorGreen => {
                let card = ctx.active_card.expect("there should be an active card");
                game.lookup_card(card).is_color(Color::Green)
            }
            Condition::IsColorWhite => {
                let card = ctx.active_card.expect("there should be an active card");
                game.lookup_card(card).is_color(Color::White)
            }
            Condition::IsCheer => {
                let card = ctx.active_card.expect("there should be an active card");
                game.lookup_card(card).is_cheer()
            }
            Condition::IsEven(value) => {
                let value = value.evaluate_with_context(ctx, game);
                value % 2 == 0
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
            Condition::IsNamedTokinoSora => {
                let card = ctx.active_card.expect("there should be an active card");
                game.lookup_card(card).is_named("Tokino Sora")
            }
            Condition::IsNot(not_card) => {
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

    fn evaluate_with_context_mut(
        &self,
        ctx: &mut EvaluateContext,
        game: &mut Game,
    ) -> EvaluateResult<Self::Value> {
        match self {
            super::LetValue::OptionalActivate => {
                let player = ctx.active_player.expect("there should be an active player");
                Ok(LetValue::Condition(
                    game.prompt_for_optional_activate(player).into(),
                ))
            }
            super::LetValue::RollDice => {
                let player = ctx.active_player.expect("there should be an active player");
                let number = game.roll_dice(ctx.active_card, player)?;
                Ok(LetValue::Number(number as usize))
            }
            super::LetValue::SelectAny(cards, condition) => {
                let player = ctx.active_player.expect("there should be an active player");
                let cards = cards.evaluate_with_context(ctx, &game.state);
                let choice = game.prompt_for_select(
                    player,
                    cards.clone(),
                    condition.as_ref().clone(),
                    ctx,
                    0,
                    usize::MAX,
                );
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
                let cards = cards.evaluate_with_context(ctx, &game.state);
                let choice = game.prompt_for_select(
                    player,
                    cards.clone(),
                    condition.as_ref().clone(),
                    ctx,
                    1,
                    1,
                );
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
                let min = min.evaluate_with_context(ctx, &game.state);
                let max = max.evaluate_with_context(ctx, &game.state);
                Ok(LetValue::Number(game.prompt_for_number(player, min, max)))
            }
            super::LetValue::SelectUpTo(amount, cards, condition) => {
                let player = ctx.active_player.expect("there should be an active player");
                let amount = amount.evaluate_with_context(ctx, &game.state);
                let cards = cards.evaluate_with_context(ctx, &game.state);
                let choice = game.prompt_for_select(
                    player,
                    cards.clone(),
                    condition.as_ref().clone(),
                    ctx,
                    0,
                    amount,
                );
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

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &GameState) -> Self::Value {
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
        }
    }
}

impl EvaluateEffect for Modifier {
    type Value = ModifierKind;

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &GameState) -> Self::Value {
        match self {
            Modifier::MoreDamage(amount) => {
                let amount = amount.evaluate_with_context(ctx, game);
                ModifierKind::MoreDamage(amount)
            }
            Modifier::NextDiceRoll(number) => {
                let number = number.evaluate_with_context(ctx, game);
                ModifierKind::NextDiceRoll(number)
            }
            Modifier::When(condition, modifier) => {
                let modifier = modifier.evaluate_with_context(ctx, game);
                ModifierKind::Conditional(condition.clone(), Box::new(modifier))
            }
        }
    }
}

impl EvaluateEffect for super::Player {
    type Value = Player;

    fn evaluate_with_context(&self, ctx: &EvaluateContext, _game: &GameState) -> Self::Value {
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

    fn evaluate_with_context(&self, ctx: &EvaluateContext, game: &GameState) -> Self::Value {
        match self {
            Number::Count(cards) => {
                let cards = cards.evaluate_with_context(ctx, game);
                cards.len()
            }
            Number::Literal(number) => number.0,
            Number::Var(var) => {
                match ctx.variables.get(&var.0).unwrap_or_else(|| {
                    panic!("the variable should exist: {:?} - ctx: {:?}", var, ctx)
                }) {
                    LetValue::Number(value) => *value,
                    _ => panic!("wrong value: {:?} - ctx: {:?}", var, ctx),
                }
            }
        }
    }
}

impl EvaluateEffect for super::Zone {
    type Value = (Player, Zone);

    fn evaluate_with_context(&self, ctx: &EvaluateContext, _game: &GameState) -> Self::Value {
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