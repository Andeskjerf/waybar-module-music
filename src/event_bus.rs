use std::{collections::HashMap, sync::mpsc};

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum EventType {
    PlayerStateChanged,
    PlayerSongChanged,
    PlaybackChanged,
    ParseError,
    Unknown(String),
}

pub enum EventBusMessage {
    Publish {
        event_type: EventType,
        data: Vec<u8>,
    },
    Subscribe {
        event_type: EventType,
        response_tx: mpsc::Sender<mpsc::Receiver<Vec<u8>>>,
    },
}

#[derive(Clone, Debug)]
pub struct EventBusHandle {
    tx: mpsc::Sender<EventBusMessage>,
}

impl EventBusHandle {
    pub fn publish(&self, event_type: EventType, data: Vec<u8>) {
        let msg = EventBusMessage::Publish { event_type, data };
        let _ = self.tx.send(msg);
    }

    pub fn subscribe(&self, event_type: EventType) -> Option<mpsc::Receiver<Vec<u8>>> {
        let (response_tx, response_rx) = mpsc::channel();
        let msg = EventBusMessage::Subscribe {
            event_type,
            response_tx,
        };

        if self.tx.send(msg).is_ok() {
            response_rx.recv().ok()
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct EventBus {
    rx: mpsc::Receiver<EventBusMessage>,
    senders: HashMap<EventType, Vec<mpsc::Sender<Vec<u8>>>>,
}

impl EventBus {
    pub fn new() -> (Self, EventBusHandle) {
        let (tx, rx) = mpsc::channel();

        let bus = Self {
            rx,
            senders: HashMap::new(),
        };

        let handle = EventBusHandle { tx };
        (bus, handle)
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

    pub fn run(mut self) {
        while let Ok(msg) = self.rx.recv() {
            match msg {
                EventBusMessage::Publish { event_type, data } => {
                    if let Some(senders) = self.senders.get(&event_type) {
                        for sender in senders {
                            let _ = sender.send(data.clone());
                        }
                    }
                }
                EventBusMessage::Subscribe {
                    event_type,
                    response_tx,
                } => {
                    let (tx, rx) = mpsc::channel();
                    self.senders
                        .entry(event_type)
                        .or_insert_with(Vec::new)
                        .push(tx);
                    let _ = response_tx.send(rx);
                }
            }
        }
    }
}
