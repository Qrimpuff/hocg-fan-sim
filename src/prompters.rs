use std::fmt::Debug;

use iter_tools::Itertools;
use rand::{seq::IteratorRandom, thread_rng, Rng};

use crate::{
    client::IntentRequestHandler,
    events::{IntentRequest, IntentResponse},
    gameplay::{CardDisplay, GameState, MainStepActionDisplay, PerformanceStepActionDisplay},
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
        println!("choosing first choice for: {text}");
        self.print_choices(&choices);

        let c = choices
            .into_iter()
            .next()
            .expect("always at least one choice");
        println!("{}", c.to_string());
        c
    }

    fn prompt_multi_choices<'a, T: ToString>(
        &mut self,
        text: &str,
        choices: Vec<T>,
        min: usize,
        _max: usize,
    ) -> Vec<T> {
        println!("choosing first choices for: {text}");
        self.print_choices(&choices);

        let c: Vec<_> = choices.into_iter().take(min).collect();
        println!("{}", c.iter().map(T::to_string).collect_vec().join(", "));
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
        println!("choosing random choice for: {text}");
        self.print_choices(&choices);

        let c = choices
            .into_iter()
            .choose(&mut thread_rng())
            .expect("always at least one choice");
        println!("{}", c.to_string());
        c
    }

    fn prompt_multi_choices<'a, T: ToString>(
        &mut self,
        text: &str,
        choices: Vec<T>,
        min: usize,
        max: usize,
    ) -> Vec<T> {
        println!("choosing random choices for: {text}");
        self.print_choices(&choices);

        let max = max.min(choices.len());

        let c = choices
            .into_iter()
            .choose_multiple(&mut thread_rng(), thread_rng().gen_range(min..=max));
        println!("{}", c.iter().map(T::to_string).collect_vec().join(", "));
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
        println!(
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
    fn handle_intent_request(&mut self, game: &GameState, req: IntentRequest) -> IntentResponse {
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
                zones,
                select_cards,
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
                card,
                select_attachments,
                min_amount,
                max_amount,
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
