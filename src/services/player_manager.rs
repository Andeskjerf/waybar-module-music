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
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct PlayerManager {
    players: Arc<Mutex<HashMap<String, PlayerClient>>>,
    dbus_client: Arc<DBusClient>,
    event_bus: EventBusHandle,
}

impl PlayerManager {
    pub fn new(event_bus: EventBusHandle, dbus_client: Arc<DBusClient>) -> Self {
        Self {
            players: Arc::new(Mutex::new(HashMap::new())),
            dbus_client,
            event_bus,
        }
    }

    fn init_listener_threads(self: Arc<Self>) {
        {
            let player_manager = Arc::clone(&self);
            let players = Arc::clone(&player_manager.players);
            thread::spawn(move || player_manager.listen_playback_changed(players));
        }
        {
            let player_manager = Arc::clone(&self);
            let players = Arc::clone(&player_manager.players);
            thread::spawn(move || player_manager.listen_song_changed(players));
        }
    }

    fn listen_playback_changed(
        self: Arc<Self>,
        players: Arc<Mutex<HashMap<String, PlayerClient>>>,
    ) {
        let rx = self
            .event_bus
            .subscribe(EventType::PlaybackChanged)
            .expect("failed to subscribe to PlaybackChanged");

        loop {
            let msg = rx.recv();
            let (playback_state, _): (MprisPlayback, usize) = match msg {
                Ok(encoded) => {
                    bincode::decode_from_slice(&encoded[..], config::standard()).unwrap()
                }
                Err(err) => {
                    println!("failed to decode message in PlayerManager!\n----\n{err}");
                    continue;
                }
            };

            let mut lock = players.lock().unwrap();
            let player_id = playback_state.player_id.clone();

            if !lock.contains_key(&player_id) {
                if let Ok(metadata) = self.dbus_client.query_metadata(&player_id) {
                    lock.insert(
                        player_id.clone(),
                        PlayerClient::new(self.event_bus.clone(), metadata),
                    );
                } else {
                    println!(
                        "got playback update for unknown player ID, and failed to query player"
                    );
                    continue;
                }
            }

            let player = lock.get_mut(&player_id).unwrap();
            player.update_playback_state(playback_state);

            // if the latest player is not playing, find the most recent one that is still playing and display that instead
            if !player.playing() {
                let (player_id, _) =
                    lock.iter()
                        .fold((String::new(), 0u64), |(player_id, ts), (key, value)| {
                            if value.playing() && value.last_updated > ts {
                                (key.clone(), value.last_updated)
                            } else {
                                (player_id, ts)
                            }
                        });
                if let Some(player) = lock.get_mut(&player_id) {
                    player.publish_state();
                }
            }
        }
    }

    fn listen_song_changed(&self, players: Arc<Mutex<HashMap<String, PlayerClient>>>) {
        let rx = self
            .event_bus
            .subscribe(EventType::PlayerSongChanged)
            .expect("failed to subscribe to PlayerSongChanged");

        loop {
            let msg = rx.recv();
            let (metadata, _): (MprisMetadata, usize) = match msg {
                Ok(encoded) => {
                    bincode::decode_from_slice(&encoded[..], config::standard()).unwrap()
                }
                Err(err) => {
                    println!("failed to decode message in PlayerManager!\n----\n{err}");
                    continue;
                }
            };

            let mut lock = players.lock().unwrap();
            let player = lock.get_mut(&metadata.player_id);

            match player {
                Some(player) => {
                    player.update_metadata(metadata);
                }
                None => {
                    lock.insert(
                        metadata.player_id.clone(),
                        PlayerClient::new(self.event_bus.clone(), metadata),
                    );
                }
            };
        }
    }
}

impl Runnable for PlayerManager {
    fn run(self: Arc<Self>) -> JoinHandle<()> {
        thread::spawn(move || {
            self.init_listener_threads();
        })
    }
}
