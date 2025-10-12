use bincode::config;
use log::{debug, error, info, warn};

use crate::{
    event_bus::{EventBusHandle, EventType},
    interfaces::dbus_client::DBusClient,
    models::{
        mpris_metadata::MprisMetadata, mpris_playback::MprisPlayback, mpris_seeked::MprisSeeked,
        player_client::PlayerClient,
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
    Metadata(MprisMetadata),
    PlaybackState(MprisPlayback),
    Seeked(MprisSeeked),
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

        PlayerManager::subscribe_to_event(
            self.event_bus.clone(),
            EventType::PlaybackChanged,
            tx.clone(),
            PlayerManagerMessage::PlaybackState,
        );
        PlayerManager::subscribe_to_event(
            self.event_bus.clone(),
            EventType::PlayerSongChanged,
            tx.clone(),
            PlayerManagerMessage::Metadata,
        );
        PlayerManager::subscribe_to_event(
            self.event_bus.clone(),
            EventType::Seeked,
            tx.clone(),
            PlayerManagerMessage::Seeked,
        );

        self.handle_events(rx);
    }

    fn subscribe_to_event<T, F>(
        event_bus: EventBusHandle,
        event_type: EventType,
        tx: Sender<PlayerManagerMessage>,
        message_constructor: F,
    ) where
        T: bincode::Decode<()>,
        F: Fn(T) -> PlayerManagerMessage + Send + 'static,
    {
        match event_bus.subscribe(event_type.clone()) {
            Some(rx) => {
                thread::spawn(move || PlayerManager::listen_for_event(rx, tx, message_constructor));
            }
            None => error!("failed to spawn listener for {event_type:?}"),
        }
    }

    fn listen_for_event<T, F>(
        subscription_rx: Receiver<Vec<u8>>,
        tx: Sender<PlayerManagerMessage>,
        message: F,
    ) where
        T: bincode::Decode<()>,
        F: Fn(T) -> PlayerManagerMessage,
    {
        loop {
            let msg = subscription_rx.recv();
            let (playback_state, _): (T, usize) = match msg {
                Ok(encoded) => {
                    bincode::decode_from_slice(&encoded[..], config::standard()).unwrap()
                }
                Err(err) => {
                    warn!("failed to decode message in PlayerManager: {err}");
                    continue;
                }
            };

            if let Err(err) = tx.send(message(playback_state)) {
                warn!("failed to send update in PlayerManager: {err}");
            }
        }
    }

    fn handle_events(&self, rx: Receiver<PlayerManagerMessage>) {
        let mut players: HashMap<String, PlayerClient> = HashMap::new();

        loop {
            let msg: PlayerManagerMessage = match rx.recv() {
                Ok(msg) => msg,
                Err(err) => {
                    warn!("failed to recv PlayerManagerMessage: {err}");
                    continue;
                }
            };

            match msg {
                PlayerManagerMessage::Metadata(mpris_metadata) => {
                    PlayerManager::handle_metadata_event(
                        &self.event_bus,
                        &mut players,
                        mpris_metadata,
                        &self.dbus_client,
                    )
                }
                PlayerManagerMessage::PlaybackState(mpris_playback) => {
                    PlayerManager::handle_playback_event(
                        &self.event_bus,
                        &mut players,
                        mpris_playback,
                        &self.dbus_client,
                    )
                }
                PlayerManagerMessage::Seeked(mpris_seeked) => PlayerManager::handle_seeked_event(
                    &self.event_bus,
                    &mut players,
                    mpris_seeked,
                    &self.dbus_client,
                ),
            };
        }
    }

    fn handle_metadata_event(
        event_bus: &EventBusHandle,
        players: &mut HashMap<String, PlayerClient>,
        mpris_metadata: MprisMetadata,
        dbus_client: &Arc<DBusClient>,
    ) {
        let player_id = mpris_metadata.player_id.clone();
        match players.entry(player_id.clone()) {
            Entry::Occupied(mut e) => e.get_mut().update_metadata(mpris_metadata),
            Entry::Vacant(e) => {
                let identity = dbus_client.query_mediaplayer_identity(&player_id);
                match identity {
                    Ok(identity) => {
                        e.insert(PlayerClient::new(
                            identity,
                            event_bus.clone(),
                            mpris_metadata,
                        ));
                    }
                    Err(err) => {
                        error!("failed to query media player identity, skipping message: {err}");
                    }
                };
            }
        }
    }

    fn handle_playback_event(
        event_bus: &EventBusHandle,
        players: &mut HashMap<String, PlayerClient>,
        mpris_playback: MprisPlayback,
        dbus_client: &Arc<DBusClient>,
    ) {
        let id = &mpris_playback.player_id;
        PlayerManager::query_player_if_not_exists(event_bus, dbus_client, players, id);

        if let Some(player) = players.get_mut(id) {
            player.update_playback_state(mpris_playback);

            // if the latest player is not playing, find the most recent one that is still playing and display that instead
            if !player.playing() {
                PlayerManager::set_most_recent_player_as_active(players);
            }
        } else {
            error!("failed to get player during PlaybackState update");
        }
    }

    fn handle_seeked_event(
        event_bus: &EventBusHandle,
        players: &mut HashMap<String, PlayerClient>,
        mpris_seeked: MprisSeeked,
        dbus_client: &Arc<DBusClient>,
    ) {
        let id = &mpris_seeked.player_id;
        PlayerManager::query_player_if_not_exists(event_bus, dbus_client, players, id);

        if let Some(player) = players.get_mut(id) {
            player.update_position(mpris_seeked);
        } else {
            error!("failed to get player during Seeked update");
        }
    }

    fn query_player_if_not_exists(
        event_bus: &EventBusHandle,
        dbus_client: &Arc<DBusClient>,
        players: &mut HashMap<String, PlayerClient>,
        id: &str,
    ) {
        if !players.contains_key(id) {
            debug!(
                "got seeked message but player does not exist, attempting to query for metadata"
            );
            if let Ok(metadata) = dbus_client.query_metadata(id) {
                match dbus_client.query_mediaplayer_identity(id) {
                    Ok(identity) => {
                        players.insert(
                            id.to_owned(),
                            PlayerClient::new(identity, event_bus.clone(), metadata),
                        );
                    }
                    Err(err) => {
                        error!("failed to query media player identity, skipping message: {err}");
                    }
                };
            } else {
                error!("got playback update for unknown player ID, and failed to query player");
            }
        }
    }

    fn set_most_recent_player_as_active(players: &mut HashMap<String, PlayerClient>) {
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
