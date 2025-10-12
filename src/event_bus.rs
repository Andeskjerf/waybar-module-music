use std::{collections::HashMap, fmt::Display, sync::mpsc};

use log::{debug, error, info, warn};

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub enum EventType {
    PlayerStateChanged,
    PlayerSongChanged,
    PlaybackChanged,
    Seeked,
    ParseError,
    Unknown(String),
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EventType::PlayerStateChanged => "PlayerStateChanged",
                EventType::PlayerSongChanged => "PlayerSongChanged",
                EventType::PlaybackChanged => "PlaybackChanged",
                EventType::Seeked => "Seeked",
                EventType::ParseError => "ParseError",
                EventType::Unknown(_) => "Unknown",
            }
        )
    }
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

impl Display for EventBusMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (msg_type, event_type) = match self {
            EventBusMessage::Publish {
                event_type,
                data: _,
            } => ("Publish", event_type),
            EventBusMessage::Subscribe {
                event_type,
                response_tx: _,
            } => ("Subscribe", event_type),
        };
        write!(f, "{msg_type}: {event_type}")
    }
}

#[derive(Clone, Debug)]
pub struct EventBusHandle {
    tx: mpsc::Sender<EventBusMessage>,
}

impl EventBusHandle {
    pub fn publish(&self, event_type: EventType, data: Vec<u8>) {
        let msg = EventBusMessage::Publish { event_type, data };
        if let Err(err) = self.tx.send(msg) {
            error!("failed to publish message on bus: {err}");
        }
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

    pub fn run(mut self) {
        info!("starting EventBus thread");
        while let Ok(msg) = self.rx.recv() {
            debug!("{msg}");
            match msg {
                EventBusMessage::Publish { event_type, data } => {
                    match self.senders.get(&event_type) {
                        Some(senders) => {
                            for sender in senders {
                                if let Err(err) = sender.send(data.clone()) {
                                    error!("failed to send data to subscribers: {err}");
                                }
                            }
                        }
                        None => {
                            warn!("tried to get subscriber with type '{event_type}' but none found")
                        }
                    }
                }
                EventBusMessage::Subscribe {
                    event_type,
                    response_tx,
                } => {
                    let (tx, rx) = mpsc::channel();
                    self.senders.entry(event_type).or_default().push(tx);
                    if let Err(err) = response_tx.send(rx) {
                        error!("failed to send receiver: {err}");
                    }
                }
            }
        }
        info!("EventBus thread is stopping");
    }
}
