use bincode::config;
use log::{debug, error, info, warn};

use crate::{
    event_bus::{EventBusHandle, EventType},
    interfaces::dbus_client::DBusClient,
    models::{
        mpris_metadata::MprisMetadata, mpris_playback::MprisPlayback, player_client::PlayerClient,
    },
    services::runnable::Runnable,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
};

#[derive(Debug)]
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

    fn init_worker(self: Arc<Self>) {
        let (tx, rx) = mpsc::channel();

        if let Some(rx) = self.event_bus.subscribe(EventType::PlaybackChanged) {
            let tx = tx.clone();
            thread::spawn(move || PlayerManager::listen_playback_changed(rx, tx));
        } else {
            error!("failed to spawn listener for playback changes");
        }

        if let Some(rx) = self.event_bus.subscribe(EventType::PlayerSongChanged) {
            let tx = tx;
            thread::spawn(move || PlayerManager::listen_song_changed(rx, tx));
        } else {
            error!("failed to spawn listener for song changes");
        }

        self.handle_events(rx);
    }

    // TODO: this and listen_song_changed could potentially be a generic function, very similar
    fn listen_playback_changed(
        subscription_rx: Receiver<Vec<u8>>,
        tx: Sender<PlayerManagerMessage>,
    ) {
        loop {
            let msg = subscription_rx.recv();
            debug!("received playback update");
            let (playback_state, _): (MprisPlayback, usize) = match msg {
                Ok(encoded) => {
                    bincode::decode_from_slice(&encoded[..], config::standard()).unwrap()
                }
                Err(err) => {
                    warn!("failed to decode message in PlayerManager: {err}");
                    continue;
                }
            };

            if let Err(err) = tx.send(PlayerManagerMessage::GotPlaybackState(playback_state)) {
                warn!("failed to send playback update in PlayerManager: {err}");
            }
        }
    }

    fn listen_song_changed(subscription_rx: Receiver<Vec<u8>>, tx: Sender<PlayerManagerMessage>) {
        loop {
            let msg = subscription_rx.recv();
            debug!("received metadata update");
            let (metadata, _): (MprisMetadata, usize) = match msg {
                Ok(encoded) => {
                    bincode::decode_from_slice(&encoded[..], config::standard()).unwrap()
                }
                Err(err) => {
                    warn!("failed to decode message in PlayerManager: {err}");
                    continue;
                }
            };

            if let Err(err) = tx.send(PlayerManagerMessage::GotMetadata(metadata)) {
                warn!("failed to send metadata message: {err}");
            }
        }
    }

    fn handle_events(&self, rx: Receiver<PlayerManagerMessage>) {
        let mut players: HashMap<String, PlayerClient> = HashMap::new();

        loop {
            debug!("waiting for message");
            let msg: PlayerManagerMessage = match rx.recv() {
                Ok(msg) => msg,
                Err(err) => {
                    warn!("failed to recv PlayerManagerMessage: {err}");
                    continue;
                }
            };

            debug!("{:?}", msg);

            match msg {
                PlayerManagerMessage::GotMetadata(mpris_metadata) => {
                    let player_id = mpris_metadata.player_id.clone();
                    match players.entry(player_id.clone()) {
                        Entry::Occupied(mut e) => e.get_mut().update_metadata(mpris_metadata),
                        Entry::Vacant(e) => {
                            let identity = self.dbus_client.query_mediaplayer_identity(&player_id);
                            match identity {
                                Ok(identity) => e.insert(PlayerClient::new(
                                    identity,
                                    self.event_bus.clone(),
                                    mpris_metadata,
                                )),
                                Err(err) => {
                                    error!("failed to query media player identity, skipping message: {err}");
                                    continue;
                                }
                            };
                        }
                    }
                }
                PlayerManagerMessage::GotPlaybackState(mpris_playback) => {
                    let id = &mpris_playback.player_id;
                    if !players.contains_key(id) {
                        debug!("got playback state but player does not exist, attempting to query for metadata");
                        if let Ok(metadata) = self.dbus_client.query_metadata(id) {
                            debug!("metadata queried successfully: {:?}", metadata);
                            let identity = self.dbus_client.query_mediaplayer_identity(id);
                            match identity {
                                Ok(identity) => players.insert(
                                    id.clone(),
                                    PlayerClient::new(identity, self.event_bus.clone(), metadata),
                                ),
                                Err(err) => {
                                    error!("failed to query media player identity, skipping message: {err}");
                                    continue;
                                }
                            };
                        } else {
                            error!(
                                "got playback update for unknown player ID, and failed to query player"
                            );
                            continue;
                        }
                    }

                    if let Some(player) = players.get_mut(id) {
                        player.update_playback_state(mpris_playback);

                        // if the latest player is not playing, find the most recent one that is still playing and display that instead
                        if !player.playing() {
                            self.set_most_recent_player_as_active(&mut players);
                        }
                    } else {
                        error!("failed to get player during PlaybackState update");
                        continue;
                    }
                }
            };
        }
    }

    fn set_most_recent_player_as_active(&self, players: &mut HashMap<String, PlayerClient>) {
        if let Some((_, player)) = players
            .iter_mut()
            .filter(|(_, p)| p.playing())
            .max_by_key(|(_, p)| p.last_updated)
        {
            player.publish_state();
        };
    }
}

impl Runnable for PlayerManager {
    fn run(self: Arc<Self>) -> JoinHandle<()> {
        thread::spawn(move || {
            info!("starting PlayerManager thread");
            self.init_worker();
            info!("PlayerManager thread is stopping");
        })
    }
}
