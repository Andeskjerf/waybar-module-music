use bincode::config;

use crate::{
    actors::runnable::Runnable,
    event_bus::{EventBus, EventType},
    models::mpris_metadata::MprisMetadata,
    player_client::PlayerClient,
};
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

/* APPROACHES

so: PlayerManager needs to do a few things.
- we need to parse any matched signals coming in from DBus
- we need to create player clients, or objects, whatever, that hold onto the last known state for each player

- should PlayerManager deal with them in its own struct?
- should we spawn new threads with each player client, and let them handle the incoming data from PlayerManager?

- finally, we need to draw it all somehow
- i'm imagining a display thread that listens to events on an event bus
*/

pub struct PlayerManager {
    players: Arc<Mutex<Vec<PlayerClient>>>,
    event_bus: Arc<Mutex<EventBus>>,
}

impl PlayerManager {
    pub fn new(event_bus: Arc<Mutex<EventBus>>) -> Self {
        Self {
            players: Arc::new(Mutex::new(vec![])),
            event_bus,
        }
    }

    fn listen_event_bus(&self) {
        let rx = self
            .event_bus
            .lock()
            .unwrap()
            .subscribe(EventType::PlayerSongChanged);

        loop {
            let msg = rx.recv();
            let metadata: (MprisMetadata, usize) = match msg {
                Ok(encoded) => {
                    bincode::decode_from_slice(&encoded[..], config::standard()).unwrap()
                }
                Err(err) => {
                    println!("failed to decode message in PlayerManager!\n----\n{err}");
                    continue;
                }
            };
        }
    }
}

impl Runnable for PlayerManager {
    fn run(self: Arc<Self>) -> JoinHandle<()> {
        thread::spawn(move || {
            self.listen_event_bus();
        })
    }
}
