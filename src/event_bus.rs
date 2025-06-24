use std::{collections::HashMap, sync::mpsc};

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum EventType {
    PlayerSongChanged,
    PlaybackChanged,
    ParseError,
    Unknown(String),
}

#[derive(Debug)]
pub struct EventBus {
    senders: HashMap<EventType, Vec<mpsc::Sender<Vec<u8>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            senders: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, event_type: EventType) -> mpsc::Receiver<Vec<u8>> {
        let (tx, rx) = mpsc::channel();
        self.senders.entry(event_type).or_default().push(tx);
        rx
    }

    pub fn publish(&self, event_type: EventType, event: Vec<u8>) {
        if let Some(senders) = self.senders.get(&event_type) {
            for sender in senders {
                let _ = sender.send(event.clone());
            }
        }
    }
}
