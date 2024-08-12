use std::sync::{
    mpsc::{Receiver, Sender},
    Arc,
};

use tracing::debug;

use crate::{cards::GlobalLibrary, events::*, gameplay::GameState};

pub struct Client<E, I> {
    game: GameState,
    send: Sender<ClientSend>,
    receive: Receiver<ClientReceive>,
    event_handler: E,
    intent_handler: I,
}
impl<E, I> Client<E, I>
where
    E: EventHandler,
    I: IntentRequestHandler,
{
    pub fn new(
        library: Arc<GlobalLibrary>,
        channels: (Sender<ClientSend>, Receiver<ClientReceive>),
        event_handler: E,
        intent_handler: I,
    ) -> Self {
        Client {
            game: GameState::new(library),
            send: channels.0,
            receive: channels.1,
            event_handler,
            intent_handler,
        }
    }

    pub fn handle_request(&mut self) {
        let req = self.receive.recv().unwrap();
        match req {
            ClientReceive::Event(event) => {
                // TODO add to card mapping
                debug!("RECEIVED EVENT = {:?}", event);
                if let Event {
                    kind: EventKind::CardMapping(CardMapping { card_map }, ..),
                    ..
                } = &event
                {
                    self.game.card_map.clone_from(card_map);
                }

                self.event_handler.handle_event(&self.game, event);
            }
            ClientReceive::IntentRequest(req) => {
                debug!("RECEIVED INTENT = {:?}", req);
                let resp = self.intent_handler.handle_intent_request(&self.game, req);
                debug!("SENT INTENT = {:?}", resp);
                self.send.send(ClientSend::IntentResponse(resp)).unwrap();
            }
        }
    }

    pub fn receive_requests(&mut self) {
        loop {
            self.handle_request();
        }
    }
}

pub trait EventHandler {
    fn handle_event(&mut self, game: &GameState, event: Event);
}
pub trait IntentRequestHandler {
    fn handle_intent_request(&mut self, game: &GameState, req: IntentRequest) -> IntentResponse;
}

pub struct DefaultEventHandler {}
impl EventHandler for DefaultEventHandler {
    fn handle_event(&mut self, _game: &GameState, _event: Event) {
        // do nothing
    }
}
