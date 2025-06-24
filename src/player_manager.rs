use bincode::config;

use crate::{
    actors::runnable::Runnable,
    event_bus::{EventBusHandle, EventType},
    models::{mpris_metadata::MprisMetadata, mpris_playback::MprisPlayback},
    player_client::PlayerClient,
    services::dbus_service::DBusService,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct PlayerManager {
    players: Arc<Mutex<HashMap<String, PlayerClient>>>,
    dbus_service: Arc<DBusService>,
    event_bus: EventBusHandle,
}

impl PlayerManager {
    pub fn new(event_bus: EventBusHandle, dbus_service: Arc<DBusService>) -> Self {
        Self {
            players: Arc::new(Mutex::new(HashMap::new())),
            dbus_service,
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
            let player = lock.get_mut(&player_id);

            match player {
                Some(player) => player.update_playback_state(playback_state),
                None => {
                    if let Ok(metadata) = self.dbus_service.query_metadata(&player_id) {
                        let mut player = PlayerClient::new(
                            self.event_bus.clone(),
                            &metadata.player_id.clone(),
                            metadata,
                        );
                        player.update_playback_state(playback_state);
                        lock.insert(player_id.clone(), player);
                    } else {
                        println!(
                            "got playback update for unknown player ID, and failed to query player"
                        );
                    }
                }
            };
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
                        PlayerClient::new(
                            self.event_bus.clone(),
                            &metadata.player_id.clone(),
                            metadata,
                        ),
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
