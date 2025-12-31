use bincode::config;
use log::{debug, error, info, warn};

use crate::{
    event_bus::{EventBusHandle, EventType},
    interfaces::dbus_client::DBusClient,
    models::{
        mpris_metadata::MprisMetadata, mpris_playback::MprisPlayback, mpris_seeked::MprisSeeked,
        player_client::PlayerClient, player_state::PlayerState, player_timer::PlayerTimer,
    },
    services::runnable::Runnable,
    utils::time::get_current_timestamp,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

#[derive(Debug, Clone)]
enum PlayerManagerMessage {
    Metadata(MprisMetadata),
    PlaybackState(MprisPlayback),
    Seeked(MprisSeeked),
    PlayerTick((String, u128)),
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
        let (timer_tx, timer_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();

        // we spawn a dedicated thread to tick player progress
        {
            let player_manager = self.clone();
            let tx = tx.clone();
            thread::spawn(move || player_manager.update_player_progress(tx, timer_rx));
        }

        self.subscribe_to_event(
            EventType::PlaybackChanged,
            tx.clone(),
            PlayerManagerMessage::PlaybackState,
        );
        self.subscribe_to_event(
            EventType::PlayerSongChanged,
            tx.clone(),
            PlayerManagerMessage::Metadata,
        );
        self.subscribe_to_event(EventType::Seeked, tx.clone(), PlayerManagerMessage::Seeked);

        self.handle_events(rx, timer_tx);
    }

    fn update_player_progress(
        &self,
        main_tx: Sender<PlayerManagerMessage>,
        rx: Receiver<PlayerManagerMessage>,
    ) {
        let mut players: HashMap<String, PlayerTimer> = HashMap::new();

        loop {
            // FIXME: what do we do when this thread sleeps, waiting to publish a tick event?
            // we could recieve a new player event while we sleep, and we won't handle it until we're done sleeping
            // not ideal!!
            // we could make the tick event happen more often, like every 250ms instead of every second?

            // if there are no players currently playing, we want to block and listen until we recieve a message
            let msg = if players.iter().any(|(_, p)| p.is_playing()) {
                match rx.try_recv() {
                    Ok(msg) => msg,
                    Err(err) => {
                        warn!("PlayerManager timer thread failed to receive message, continuing anyway: {err}");
                        continue;
                    }
                }
            } else {
                match rx.recv() {
                    Ok(msg) => msg,
                    Err(err) => {
                        warn!("PlayerManager timer thread failed to receive message, continuing anyway: {err}");
                        continue;
                    }
                }
            };

            match msg {
                // we must handle metadata events so that we know the full length of the media playing
                PlayerManagerMessage::Metadata(mpris_metadata) => {
                    let id = mpris_metadata.player_id;
                    match players.entry(id.clone()) {
                        Entry::Occupied(mut e) => {
                            e.insert(PlayerTimer::new(id.clone(), mpris_metadata.length.unwrap()));
                        }
                        Entry::Vacant(e) => {
                            e.insert(PlayerTimer::new(id.clone(), mpris_metadata.length.unwrap()));
                        }
                    }
                }
                PlayerManagerMessage::PlaybackState(mpris_playback) => {
                    let id = mpris_playback.player_id;
                }
                // and we must obviously handle seeked events to update where we're at in the media
                PlayerManagerMessage::Seeked(mpris_seeked) => todo!(),
                // we don't care about any other events
                _ => (),
            }

            let (id, player) = match players
                .iter_mut()
                .filter(|(_, player)| player.is_playing())
                .max_by_key(|(_, player)| player.time_ms_since_last_update())
            {
                Some(player) => player,
                None => continue,
            };

            // TODO: need to take into account the rate of playback
            let increment_ms: u128 = 250;
            thread::sleep(Duration::from_millis(
                (increment_ms - player.time_ms_since_last_update()) as u64,
            ));

            player.tick(increment_ms);
            if let Err(error) = main_tx.send(PlayerManagerMessage::PlayerTick((
                id.clone(),
                player.position(),
            ))) {
                error!("PlayerManager timer thread: failed to publish tick event! {error}");
            }
        }
    }

    fn subscribe_to_event<T, F>(
        self: &Arc<Self>,
        event_type: EventType,
        tx: Sender<PlayerManagerMessage>,
        message_constructor: F,
    ) where
        T: bincode::Decode<()>,
        F: Fn(T) -> PlayerManagerMessage + Send + 'static,
    {
        match self.event_bus.subscribe(event_type.clone()) {
            Some(rx) => {
                thread::spawn(move || loop {
                    let msg = rx.recv();
                    let (playback_state, _): (T, usize) = match msg {
                        Ok(encoded) => {
                            bincode::decode_from_slice(&encoded[..], config::standard()).unwrap()
                        }
                        Err(err) => {
                            warn!("failed to decode message in PlayerManager: {err}");
                            continue;
                        }
                    };

                    if let Err(err) = tx.send(message_constructor(playback_state)) {
                        warn!("failed to send update in PlayerManager: {err}");
                    }
                });
            }
            None => error!("failed to spawn listener for {event_type:?}"),
        }
    }

    fn handle_events(
        &self,
        rx: Receiver<PlayerManagerMessage>,
        timer_tx: Sender<PlayerManagerMessage>,
    ) {
        let mut players: HashMap<String, PlayerClient> = HashMap::new();
        loop {
            let msg: PlayerManagerMessage = match rx.recv() {
                Ok(msg) => msg,
                Err(err) => {
                    warn!("failed to recv PlayerManagerMessage: {err}");
                    continue;
                }
            };

            if let Err(err) = timer_tx.send(msg.clone()) {
                warn!("PlayerManager: failed to re-send message to timer thread! {err}");
            }
            match msg {
                PlayerManagerMessage::Metadata(mpris_metadata) => {
                    self.handle_metadata_event(&mut players, mpris_metadata);
                }
                PlayerManagerMessage::PlaybackState(mpris_playback) => {
                    self.handle_playback_event(&mut players, mpris_playback)
                }
                PlayerManagerMessage::Seeked(mpris_seeked) => {
                    self.handle_seeked_event(&mut players, mpris_seeked)
                }
                PlayerManagerMessage::PlayerTick((id, position)) => match players.get_mut(&id) {
                    Some(player) => player.update_position(position),
                    None => warn!(
                        "PlayerTick event: tried to get player '{id}', but no such player exists"
                    ),
                },
            };
        }
    }

    fn handle_metadata_event(
        &self,
        players: &mut HashMap<String, PlayerClient>,
        mpris_metadata: MprisMetadata,
    ) {
        let player_id = mpris_metadata.player_id.clone();
        match players.entry(player_id.clone()) {
            Entry::Occupied(mut e) => {
                e.get_mut().update_metadata(mpris_metadata);
                self.publish_player_state(e.get_mut());
            }
            Entry::Vacant(e) => {
                let identity = self.dbus_client.query_mediaplayer_identity(&player_id);
                match identity {
                    Ok(identity) => {
                        e.insert(PlayerClient::new(identity, mpris_metadata));
                    }
                    Err(err) => {
                        error!("failed to query media player identity, skipping message: {err}");
                    }
                };
            }
        }
    }

    fn handle_playback_event(
        &self,
        players: &mut HashMap<String, PlayerClient>,
        mpris_playback: MprisPlayback,
    ) {
        let id = &mpris_playback.player_id;
        self.query_player_if_not_exists(players, id);

        if let Some(player) = players.get_mut(id) {
            player.update_playback_state(mpris_playback);
            self.publish_player_state(player);

            // if the latest player is not playing, find the most recent one that is still playing and display that instead
            if !player.playing() {
                self.set_most_recent_player_as_active(players);
            } else {
                self.publish_player_state(player);
            }
        } else {
            error!("failed to get player during PlaybackState update");
        }
    }

    fn handle_seeked_event(
        &self,
        players: &mut HashMap<String, PlayerClient>,
        mpris_seeked: MprisSeeked,
    ) {
        let id = &mpris_seeked.player_id;
        self.query_player_if_not_exists(players, id);

        if let Some(player) = players.get_mut(id) {
            info!(
                "we're updating based on a seek event, using the following data: {:?}",
                mpris_seeked
            );
            player.update_position(mpris_seeked.position);
            self.publish_player_state(player);
        } else {
            error!("failed to get player during Seeked update");
        }
    }

    fn query_player_if_not_exists(&self, players: &mut HashMap<String, PlayerClient>, id: &str) {
        if !players.contains_key(id) {
            debug!(
                "got seeked message but player does not exist, attempting to query for metadata"
            );
            if let Ok(metadata) = self.dbus_client.query_metadata(id) {
                match self.dbus_client.query_mediaplayer_identity(id) {
                    Ok(identity) => {
                        players.insert(id.to_owned(), PlayerClient::new(identity, metadata));
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

    fn set_most_recent_player_as_active(&self, players: &mut HashMap<String, PlayerClient>) {
        if let Some((_, player)) = players
            .iter_mut()
            .filter(|(_, p)| p.playing())
            .max_by_key(|(_, p)| p.last_updated)
        {
            self.publish_player_state(player);
        };
    }

    pub fn publish_player_state(&self, player: &mut PlayerClient) {
        player.last_updated = get_current_timestamp();

        match PlayerState::from_mpris_data(
            player.name().to_owned(),
            player.metadata(),
            player.playback_state(),
        ) {
            Some(state) => match bincode::encode_to_vec(state, config::standard()) {
                Ok(encoded) => self
                    .event_bus
                    .publish(EventType::PlayerStateChanged, encoded),
                Err(err) => {
                    warn!("failed to encode player state, skipping publish\n\n{err}");
                }
            },
            None => {
                warn!("failed to construct PlayerState. did we get empty metadata? skipping publish: {:?}", player.metadata());
            }
        }
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
