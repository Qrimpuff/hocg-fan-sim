use std::fmt::Debug;

use iter_tools::Itertools;
use rand::{seq::IteratorRandom, thread_rng, Rng};
use tracing::info;

use crate::{
    client::IntentRequestHandler,
    events::{IntentRequest, IntentResponse},
    gameplay::{CardDisplay, Game, MainStepActionDisplay, PerformanceStepActionDisplay},
};

#[derive(Debug, Default)]
pub struct DefaultPrompter {}
impl DefaultPrompter {
    pub fn new() -> Self {
        DefaultPrompter {}
    }
}

impl Prompter for DefaultPrompter {
    fn prompt_choice<'a, T: ToString>(&mut self, text: &str, choices: Vec<T>) -> T {
        info!("choosing first choice for: {text}");
        self.print_choices(&choices);

        let c = choices
            .into_iter()
            .next()
            .expect("always at least one choice");
        info!("{}", c.to_string());
        c
    }

    fn prompt_multi_choices<'a, T: ToString>(
        &mut self,
        text: &str,
        choices: Vec<T>,
        min: usize,
        _max: usize,
    ) -> Vec<T> {
        info!("choosing first choices for: {text}");
        self.print_choices(&choices);

        let c: Vec<_> = choices.into_iter().take(min).collect();
        info!("{}", c.iter().map(T::to_string).collect_vec().join(", "));
        c
    }
}

#[derive(Debug, Default)]
pub struct RandomPrompter {}
impl RandomPrompter {
    pub fn new() -> Self {
        RandomPrompter {}
    }
}

impl Prompter for RandomPrompter {
    fn prompt_choice<'a, T: ToString>(&mut self, text: &str, choices: Vec<T>) -> T {
        info!("choosing random choice for: {text}");
        self.print_choices(&choices);

        let c = choices
            .into_iter()
            .choose(&mut thread_rng())
            .expect("always at least one choice");
        info!("{}", c.to_string());
        c
    }

    fn prompt_multi_choices<'a, T: ToString>(
        &mut self,
        text: &str,
        choices: Vec<T>,
        min: usize,
        max: usize,
    ) -> Vec<T> {
        info!("choosing random choices for: {text}");
        self.print_choices(&choices);

        let max = max.min(choices.len());

        let c = choices
            .into_iter()
            .choose_multiple(&mut thread_rng(), thread_rng().gen_range(min..=max));
        info!("{}", c.iter().map(T::to_string).collect_vec().join(", "));
        c
    }
}

#[derive(Debug, Default)]
pub struct PreferFirstPrompter {}
impl PreferFirstPrompter {
    pub fn new() -> Self {
        PreferFirstPrompter {}
    }
}

impl Prompter for PreferFirstPrompter {
    fn prompt_choice<'a, T: ToString>(&mut self, text: &str, choices: Vec<T>) -> T {
        info!("choosing random (prefer first) choice for: {text}");
        self.print_choices(&choices);

        let last = choices.len() - 1;
        for (i, c) in choices.into_iter().enumerate() {
            if thread_rng().gen_bool(1.1 / 2.0) || i == last {
                info!("{}", c.to_string());
                return c;
            }
        }
        unreachable!()
    }

    fn prompt_multi_choices<'a, T: ToString>(
        &mut self,
        text: &str,
        choices: Vec<T>,
        min: usize,
        max: usize,
    ) -> Vec<T> {
        info!("choosing random (prefer first) choices for: {text}");
        self.print_choices(&choices);

        let max = max.min(choices.len());

        let last = choices.len() - 1;
        let mut cs = vec![];
        for (i, c) in choices.into_iter().enumerate() {
            if thread_rng().gen_bool(1.1 / 2.0) && cs.len() < max
                || min.saturating_sub(cs.len()) > last - i
            {
                cs.push(c);
            }
        }

        info!("{}", cs.iter().map(T::to_string).collect_vec().join(", "));
        cs
    }
}

#[derive(Debug, Default)]
pub struct BufferedPrompter {
    buffer: Vec<Vec<usize>>,
}
impl BufferedPrompter {
    pub fn new(buffer: &[&[usize]]) -> Self {
        BufferedPrompter {
            buffer: buffer
                .iter()
                .map(|b| b.iter().copied().collect_vec())
                .collect_vec(),
        }
    }
}

impl Prompter for BufferedPrompter {
    fn prompt_choice<'a, T: ToString>(&mut self, text: &str, mut choices: Vec<T>) -> T {
        info!("choosing buffered choice for: {text}");
        self.print_choices(&choices);

        let mut buf = self.buffer.remove(0);
        assert!(buf.len() == 1);
        let c = choices.remove(buf.remove(0));
        info!("{}", c.to_string());
        c
    }

    fn prompt_multi_choices<'a, T: ToString>(
        &mut self,
        text: &str,
        choices: Vec<T>,
        min: usize,
        max: usize,
    ) -> Vec<T> {
        info!("choosing buffered choices for: {text}");
        self.print_choices(&choices);

        let max = max.min(choices.len());

        let buf = self.buffer.remove(0);
        assert!(buf.len() >= min);
        assert!(buf.len() <= max);
        let c = choices
            .into_iter()
            .enumerate()
            .filter(|(i, _)| buf.contains(i))
            .map(|(_, c)| c)
            .collect_vec();
        info!("{}", c.iter().map(T::to_string).collect_vec().join(", "));
        c
    }
}

pub trait Prompter: Debug {
    fn prompt_choice<T: ToString>(&mut self, text: &str, choices: Vec<T>) -> T;
    fn prompt_multi_choices<T: ToString>(
        &mut self,
        text: &str,
        choices: Vec<T>,
        min: usize,
        max: usize,
    ) -> Vec<T>;

    fn print_choices<T: ToString>(&mut self, choices: &[T]) {
        info!(
            "options:\n{}",
            choices
                .iter()
                .map(|c| format!("  - {}", c.to_string()))
                .collect_vec()
                .join("\n")
        );
    }
}

impl<P: Prompter> IntentRequestHandler for P {
    fn handle_intent_request(&mut self, game: &Game, req: IntentRequest) -> IntentResponse {
        match req {
            IntentRequest::Rps { player, select_rps } => {
                let rps = self.prompt_choice("choose rock, paper or scissor:", select_rps);
                IntentResponse::Rps {
                    player,
                    select_rps: rps,
                }
            }
            IntentRequest::Mulligan { player, .. } => {
                let mulligan =
                    self.prompt_choice("do you want to mulligan?", vec!["Yes", "No"]) == "Yes";
                IntentResponse::Mulligan {
                    player,
                    select_yes_no: mulligan,
                }
            }
            IntentRequest::ActivateEffect { player, .. } => {
                let activate = self
                    .prompt_choice("do you want to activate the effect?", vec!["Yes", "No"])
                    == "Yes";
                IntentResponse::ActivateEffect {
                    player,
                    select_yes_no: activate,
                }
            }
            IntentRequest::LookSelectZoneToZone {
                player,
                select_cards,
                min_amount,
                max_amount,
                ..
            } => {
                let select_cards = select_cards
                    .into_iter()
                    .map(|c| CardDisplay::new(c, game))
                    .collect_vec();
                let selected = self
                    .prompt_multi_choices("choose cards:", select_cards, min_amount, max_amount)
                    .into_iter()
                    .map(|c| c.card)
                    .collect();
                IntentResponse::LookSelectZoneToZone {
                    player,
                    select_cards: selected,
                }
            }
            IntentRequest::SelectToAttach {
                player,
                select_cards,
                ..
            } => {
                let select_cards = select_cards
                    .into_iter()
                    .map(|c| CardDisplay::new(c, game))
                    .collect_vec();
                let selected = self.prompt_choice("choose card:", select_cards).card;
                IntentResponse::SelectToAttach {
                    player,
                    select_card: selected,
                }
            }
            IntentRequest::MainStepAction {
                player,
                select_actions,
            } => {
                let select_actions = select_actions
                    .into_iter()
                    .map(|a| MainStepActionDisplay::new(a, game))
                    .collect_vec();
                let selected = self
                    .prompt_choice("main step action:", select_actions)
                    .action;
                IntentResponse::MainStepAction {
                    player,
                    select_action: selected,
                }
            }
            IntentRequest::SelectAttachments {
                player,
                select_attachments,
                min_amount,
                max_amount,
                ..
            } => {
                let select_attachments = select_attachments
                    .into_iter()
                    .map(|c| CardDisplay::new(c, game))
                    .collect_vec();
                let selected = self
                    .prompt_multi_choices(
                        "choose attachments:",
                        select_attachments,
                        min_amount,
                        max_amount,
                    )
                    .into_iter()
                    .map(|c| c.card)
                    .collect();
                IntentResponse::SelectAttachments {
                    player,
                    select_attachments: selected,
                }
            }
            IntentRequest::PerformanceStepAction {
                player,
                select_actions,
            } => {
                let select_actions = select_actions
                    .into_iter()
                    .map(|a| PerformanceStepActionDisplay::new(a, game))
                    .collect_vec();
                let selected = self
                    .prompt_choice("performance step action:", select_actions)
                    .action;
                IntentResponse::PerformanceStepAction {
                    player,
                    select_action: selected,
                }
            }
            IntentRequest::SelectNumber {
                player,
                select_numbers,
            } => {
                let number = self.prompt_choice("choose a number:", select_numbers);
                IntentResponse::SelectNumber {
                    player,
                    select_number: number,
                }
            }
        }
    }
}
