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

                // sync state with server
                event.apply_state_change(&mut self.game);

                if let Some(outcome) = self.game.game_outcome {
                    return Err(outcome);
                }

                // send event to the client
                self.event_handler.handle_event(&self.game, event).await;
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
    fn handle_event(
        &mut self,
        game: &GameState,
        event: Event,
    ) -> impl std::future::Future<Output = ()> + Send;
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
    async fn handle_event(&mut self, _game: &GameState, _event: Event) {
        // do nothing
    }
}
