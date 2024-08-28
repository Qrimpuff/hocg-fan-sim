use async_channel::{Receiver, Sender};
use tracing::debug;

use crate::{
    events::*,
    gameplay::{GameContinue, GameOutcome, GameResult, GameState},
};

pub struct Client<E, I> {
    pub game: GameState,
    pub send: Sender<ClientSend>,
    pub receive: Receiver<ClientReceive>,
    pub event_handler: E,
    pub intent_handler: I,
}
impl<E, I> Client<E, I>
where
    E: EventHandler,
    I: IntentRequestHandler,
{
    pub fn new(
        channels: (Sender<ClientSend>, Receiver<ClientReceive>),
        event_handler: E,
        intent_handler: I,
    ) -> Self {
        Client {
            game: GameState::new(),
            send: channels.0,
            receive: channels.1,
            event_handler,
            intent_handler,
        }
    }

    pub async fn handle_request(&mut self) -> GameResult {
        let req = self
            .receive
            .recv()
            .await
            .map_err(|_| self.game.game_outcome.expect("game should be over"))?;
        match req {
            ClientReceive::Event(event) => {
                debug!("RECEIVED EVENT = {:?}", event);

                match &event.kind {
                    // EventKind::Setup(_) => todo!(),
                    // EventKind::Shuffle(_) => todo!(),
                    // EventKind::RpsOutcome(_) => todo!(),
                    // EventKind::PlayerGoingFirst(_) => todo!(),
                    // EventKind::Reveal(_) => todo!(),
                    // TODO add to card mapping, by reveal, or gradually
                    EventKind::CardMapping(CardMapping { card_map }, ..) => {
                        self.game.card_map.clone_from(card_map);
                    }
                    // EventKind::GameStart(_) => todo!(),
                    EventKind::GameOver(game_over) => {
                        self.game.game_outcome = Some(game_over.game_outcome);
                        return Err(game_over.game_outcome);
                    }
                    // EventKind::StartTurn(_) => todo!(),
                    // EventKind::EndTurn(_) => todo!(),
                    // EventKind::EnterStep(_) => todo!(),
                    // EventKind::ExitStep(_) => todo!(),
                    // EventKind::AddCardModifiers(_) => todo!(),
                    // EventKind::RemoveCardModifiers(_) => todo!(),
                    // EventKind::ClearCardModifiers(_) => todo!(),
                    // EventKind::AddZoneModifiers(_) => todo!(),
                    // EventKind::RemoveZoneModifiers(_) => todo!(),
                    // EventKind::AddDamageMarkers(_) => todo!(),
                    // EventKind::RemoveDamageMarkers(_) => todo!(),
                    // EventKind::ClearDamageMarkers(_) => todo!(),
                    // EventKind::LookAndSelect(_) => todo!(),
                    // EventKind::ZoneToZone(_) => todo!(),
                    // EventKind::ZoneToAttach(_) => todo!(),
                    // EventKind::AttachToAttach(_) => todo!(),
                    // EventKind::AttachToZone(_) => todo!(),
                    // EventKind::Draw(_) => todo!(),
                    // EventKind::Collab(_) => todo!(),
                    // EventKind::LoseLives(_) => todo!(),
                    // EventKind::Bloom(_) => todo!(),
                    // EventKind::BatonPass(_) => todo!(),
                    // EventKind::ActivateSupportCard(_) => todo!(),
                    // EventKind::ActivateSupportAbility(_) => todo!(),
                    // EventKind::ActivateOshiSkill(_) => todo!(),
                    // EventKind::ActivateHoloMemberAbility(_) => todo!(),
                    // EventKind::ActivateHoloMemberArtEffect(_) => todo!(),
                    // EventKind::PerformArt(_) => todo!(),
                    // EventKind::WaitingForPlayerIntent(_) => todo!(),
                    // EventKind::HoloMemberDefeated(_) => todo!(),
                    // EventKind::DealDamage(_) => todo!(),
                    // EventKind::RollDice(_) => todo!(),
                    _ => {}
                }

                self.event_handler.handle_event(&self.game, event);
            }
            ClientReceive::IntentRequest(req) => {
                debug!("RECEIVED INTENT = {:?}", req);
                let resp = self.intent_handler.handle_intent_request(&self.game, req);
                debug!("SENT INTENT = {:?}", resp);
                self.send
                    .send(ClientSend::IntentResponse(resp))
                    .await
                    .unwrap();
            }
        }

        Ok(GameContinue)
    }

    pub async fn receive_requests(mut self) -> GameOutcome {
        // loop until the end of the game
        while self.handle_request().await.is_ok() {}

        debug!("GAME OUTCOME = {:?}", self.game.game_outcome);
        self.game.game_outcome.expect("game should be over")
    }
}

pub trait EventHandler {
    fn handle_event(&mut self, game: &GameState, event: Event);
}
pub trait IntentRequestHandler {
    fn handle_intent_request(&mut self, game: &GameState, req: IntentRequest) -> IntentResponse;
}

#[derive(Default)]
pub struct DefaultEventHandler {}
impl DefaultEventHandler {
    pub fn new() -> Self {
        Self {}
    }
}
impl EventHandler for DefaultEventHandler {
    fn handle_event(&mut self, _game: &GameState, _event: Event) {
        // do nothing
    }
}
