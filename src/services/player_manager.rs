use bincode::config;

use crate::{
    event_bus::{EventBusHandle, EventType},
    interfaces::dbus_client::DBusClient,
    models::{
        mpris_metadata::MprisMetadata, mpris_playback::MprisPlayback, player_client::PlayerClient,
    },
    services::runnable::Runnable,
};
use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
};

enum PlayerManagerMessage {
    GotMetadata(MprisMetadata),
    GotPlaybackState(MprisPlayback),
}

pub struct PlayerManager {
    dbus_client: Arc<DBusClient>,
    event_bus: EventBusHandle,
}

impl PlayerManager {
    pub fn new(event_bus: EventBusHandle, dbus_client: Arc<DBusClient>) -> Self {
        Self {
            dbus_client,
            event_bus,
        }
    }

    fn init_threads_and_listen(self: Arc<Self>) {
        let (tx, rx) = mpsc::channel();

        {
            let rx = self
                .event_bus
                .subscribe(EventType::PlaybackChanged)
                .unwrap();
            let tx = tx.clone();
            thread::spawn(move || PlayerManager::listen_playback_changed(rx, tx));
        }

        {
            let rx = self
                .event_bus
                .subscribe(EventType::PlayerSongChanged)
                .unwrap();
            let tx = tx.clone();
            thread::spawn(move || PlayerManager::listen_song_changed(rx, tx));
        }

        let mut players: HashMap<String, PlayerClient> = HashMap::new();

        loop {
            let msg: PlayerManagerMessage = match rx.recv() {
                Ok(msg) => msg,
                Err(err) => {
                    eprintln!("failed to recv PlayerManagerMessage\n{err}");
                    continue;
                }
            };

            match msg {
                PlayerManagerMessage::GotMetadata(mpris_metadata) => {
                    let id = &mpris_metadata.player_id;
                    match players.get_mut(id) {
                        Some(player) => player.update_metadata(mpris_metadata),
                        None => {
                            players.insert(
                                id.clone(),
                                PlayerClient::new(self.event_bus.clone(), mpris_metadata),
                            );
                        }
                    }
                }
                PlayerManagerMessage::GotPlaybackState(mpris_playback) => {
                    let id = &mpris_playback.player_id;
                    if !players.contains_key(id) {
                        if let Ok(metadata) = self.dbus_client.query_metadata(id) {
                            players.insert(
                                id.clone(),
                                PlayerClient::new(self.event_bus.clone(), metadata),
                            );
                        } else {
                            eprintln!(
                                "got playback update for unknown player ID, and failed to query player"
                            );
                            continue;
                        }
                    }

                    let player = players.get_mut(id).unwrap();
                    player.update_playback_state(mpris_playback);

                    // if the latest player is not playing, find the most recent one that is still playing and display that instead
                    if !player.playing() {
                        let (player_id, _) = players.iter().fold(
                            (String::new(), 0u64),
                            |(player_id, ts), (key, value)| {
                                if value.playing() && value.last_updated > ts {
                                    (key.clone(), value.last_updated)
                                } else {
                                    (player_id, ts)
                                }
                            },
                        );
                        if let Some(player) = players.get_mut(&player_id) {
                            player.publish_state();
                        }
                    }
                }
            };
        }
    }

    fn listen_playback_changed(
        subscription_rx: Receiver<Vec<u8>>,
        tx: Sender<PlayerManagerMessage>,
    ) {
        loop {
            let msg = subscription_rx.recv();
            let (playback_state, _): (MprisPlayback, usize) = match msg {
                Ok(encoded) => {
                    bincode::decode_from_slice(&encoded[..], config::standard()).unwrap()
                }
                Err(err) => {
                    println!("failed to decode message in PlayerManager!\n----\n{err}");
                    continue;
                }
            };

            if let Err(err) = tx.send(PlayerManagerMessage::GotPlaybackState(playback_state)) {
                eprintln!("failed to send playback update in PlayerManager\n{err}");
            }
        }
    }

    fn listen_song_changed(subscription_rx: Receiver<Vec<u8>>, tx: Sender<PlayerManagerMessage>) {
        loop {
            let msg = subscription_rx.recv();
            let (metadata, _): (MprisMetadata, usize) = match msg {
                Ok(encoded) => {
                    bincode::decode_from_slice(&encoded[..], config::standard()).unwrap()
                }
                Err(err) => {
                    println!("failed to decode message in PlayerManager!\n----\n{err}");
                    continue;
                }
            };

            if let Err(err) = tx.send(PlayerManagerMessage::GotMetadata(metadata)) {
                eprintln!("failed to send metadata message\n{err}");
            }
        }
    }
}

impl Runnable for PlayerManager {
    fn run(self: Arc<Self>) -> JoinHandle<()> {
        thread::spawn(move || {
            self.init_threads_and_listen();
        })
    }
}
