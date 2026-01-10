use bincode::config;
use log::{debug, error, info, warn};

use crate::{
    event_bus::{EventBusHandle, EventType},
    interfaces::dbus_client::DBusClient,
    models::{
        mpris_identity::MprisIdentity, mpris_metadata::MprisMetadata,
        mpris_playback::MprisPlayback, mpris_rate::MprisRate, mpris_seeked::MprisSeeked,
        player_client::PlayerClient, player_state::PlayerState, player_timer::PlayerTimer,
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
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
enum PlayerManagerMessage {
    Metadata(MprisMetadata),
    PlaybackState(MprisPlayback),
    Seeked(MprisSeeked),
    Rate(MprisRate),
    Identity(MprisIdentity),
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
        self.subscribe_to_event(EventType::Rate, tx.clone(), PlayerManagerMessage::Rate);

        self.handle_events(rx, timer_tx);
    }

    fn update_player_progress(
        &self,
        main_tx: Sender<PlayerManagerMessage>,
        rx: Receiver<PlayerManagerMessage>,
    ) {
        // since we're trying to avoid locks & shared state, we want to rely on channels & messages,
        // we unfortunately have to maintain two separate HashMap's of our player data
        // this HashMap only contains data relevant to tick a player's position
        let mut players: HashMap<String, PlayerTimer> = HashMap::new();

        loop {
            // FIXME: what do we do when this thread sleeps, waiting to publish a tick event?
            // we could recieve a new player event while we sleep, and we won't handle it until we're done sleeping
            // not ideal!!
            // we could make the tick event happen more often, like every 250ms instead of every second?

            // if there are no players currently playing, we want to block and listen until we recieve a message
            let msg = if players.iter().any(|(_, p)| p.is_playing()) {
                rx.try_recv().ok()
            } else {
                match rx.recv() {
                    Ok(msg) => Some(msg),
                    Err(_) => continue,
                }
            };

            // TODO: this could be improved
            // every message is handled almost identically
            // the only difference being the value we wish to set & get
            if let Some(msg) = msg {
                match msg {
                    PlayerManagerMessage::PlaybackState(mpris_playback) => {
                        let id = &mpris_playback.player_id;
                        match players.entry(id.clone()) {
                            Entry::Occupied(mut e) => {
                                e.get_mut().set_playing(mpris_playback.is_playing());
                            }
                            Entry::Vacant(e) => {
                                let mut timer = PlayerTimer::new();
                                timer.set_playing(mpris_playback.is_playing());
                                e.insert(timer);
                            }
                        }
                    }
                    PlayerManagerMessage::Seeked(mpris_seeked) => {
                        let id = &mpris_seeked.player_id;
                        match players.entry(id.clone()) {
                            Entry::Occupied(mut e) => {
                                e.get_mut().set_position(mpris_seeked.position);
                            }
                            Entry::Vacant(e) => {
                                let mut timer = PlayerTimer::new();
                                timer.set_position(mpris_seeked.position);
                                e.insert(timer);
                            }
                        }
                    }
                    PlayerManagerMessage::Rate(mpris_rate) => {
                        let id = &mpris_rate.player_id;
                        match players.entry(id.clone()) {
                            Entry::Occupied(mut e) => {
                                e.get_mut().set_rate(mpris_rate.rate);
                            }
                            Entry::Vacant(e) => {
                                let mut timer = PlayerTimer::new();
                                timer.set_rate(mpris_rate.rate);
                                e.insert(timer);
                            }
                        }
                    }
                    // we don't care about any other events
                    _ => continue,
                }
            }

            // TODO: to better sync the module & player,
            // we could figure out the difference between our clock and the player
            // e.g, on first run, delay the amount required to get an even, for example, 250ms difference
            // between the player and the module
            // we could also query the media player directly for its position every now and then
            let increment_ms: u128 = 250;
            players
                .iter_mut()
                .filter(|(_, p)| p.is_playing())
                .for_each(|(id, player)| {
                    player.tick(increment_ms);

                    if let Err(error) = main_tx.send(PlayerManagerMessage::PlayerTick((
                        id.clone(),
                        player.position(),
                    ))) {
                        error!("PlayerManager timer thread: failed to publish tick event! {error}");
                    }
                });
            thread::sleep(Duration::from_millis((increment_ms) as u64));
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

            match msg.clone() {
                PlayerManagerMessage::Metadata(mpris_metadata) => {
                    self.handle_metadata_event(&mut players, mpris_metadata);
                }
                PlayerManagerMessage::PlaybackState(mpris_playback) => {
                    if let Err(err) = timer_tx.send(msg) {
                        warn!("PlayerManager: failed to re-send message to timer thread! {err}");
                    }
                    self.handle_playback_event(&mut players, mpris_playback)
                }
                PlayerManagerMessage::Seeked(mpris_seeked) => {
                    if let Err(err) = timer_tx.send(msg) {
                        warn!("PlayerManager: failed to re-send message to timer thread! {err}");
                    }
                    self.handle_seeked_event(&mut players, mpris_seeked)
                }
                PlayerManagerMessage::Rate(_) => {
                    if let Err(err) = timer_tx.send(msg) {
                        warn!("PlayerManager: failed to re-send message to timer thread! {err}");
                    }
                }
                PlayerManagerMessage::Identity(mpris_identity) => {
                    if let Some(p) = players.get_mut(&mpris_identity.player_id) {
                        p.set_name(mpris_identity.identity);
                    }
                }
                PlayerManagerMessage::PlayerTick((id, position)) => {
                    let last_updated_player = self.get_last_updated_player(&players);
                    match players
                        .iter_mut()
                        .filter(|(_, p)| p.playing())
                        .collect::<HashMap<&String, &mut PlayerClient>>()
                        .get_mut(&id)
                    {
                        Some(player) => {
                            player.update_position(position);
                            if last_updated_player.is_some_and(|p| p.name() == player.name()) {
                                self.publish_player_state(player, false);
                            }
                        }
                        None => warn!(
                        "PlayerTick event: tried to get player '{id}', but no such player exists"
                    ),
                    }
                }
            };
        }
    }

    fn handle_metadata_event(
        &self,
        players: &mut HashMap<String, PlayerClient>,
        mpris_metadata: MprisMetadata,
    ) {
        let player_id = mpris_metadata.player_id.clone();
        let active_players = self.get_active_player_ids(players);
        match players.entry(player_id.clone()) {
            Entry::Occupied(mut e) => {
                e.get_mut().update_metadata(mpris_metadata);
                match self.dbus_client.query_playback_status(&player_id) {
                    Ok(playback) => e.get_mut().update_playback_state(playback),
                    Err(err) => warn!("PlayerManager::handle_metadata_event: failed to query playback state, {err}"),
                }

                if !e.get().playing() && active_players.iter().any(|id| id != &e.get().get_id()) {
                } else {
                    self.publish_player_state(e.get_mut(), true);
                }
            }
            Entry::Vacant(e) => {
                let identity = self.dbus_client.query_mediaplayer_identity(&player_id);
                let playback = self.dbus_client.query_playback_status(&player_id);
                match identity {
                    Ok(identity) => {
                        let mut player_client = PlayerClient::new(identity, mpris_metadata);
                        if let Ok(playback) = playback {
                            player_client.update_playback_state(playback);
                        }
                        e.insert(player_client);
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

        let active_players = self.get_active_player_ids(players);
        if let Some(player) = players.get_mut(id) {
            match self.dbus_client.query_metadata(id) {
                Ok(metadata) => player.update_metadata(metadata),
                Err(err) => {
                    warn!("PlayerManager::handle_playback_event: failed to query metadata, {err}")
                }
            }
            player.update_playback_state(mpris_playback);

            // if the latest player is not playing, find the most recent one that is still playing and display that instead
            if !player.playing() && active_players.iter().any(|id| id != &player.get_id()) {
                self.set_most_recent_player_as_active(players);
            } else {
                self.publish_player_state(player, true);
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
            player.update_position(mpris_seeked.position);
            self.publish_player_state(player, true);
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
            self.publish_player_state(player, true);
        };
    }

    fn get_last_updated_player(
        &self,
        players: &HashMap<String, PlayerClient>,
    ) -> Option<PlayerClient> {
        players.values().cloned().max_by_key(|p| p.last_updated)
    }

    fn get_active_player_ids(&self, players: &mut HashMap<String, PlayerClient>) -> Vec<String> {
        players
            .iter()
            .filter(|(_, p)| p.playing())
            .map(|(id, _)| id)
            .cloned()
            .collect::<Vec<String>>()
    }

    pub fn publish_player_state(&self, player: &mut PlayerClient, set_last_updated: bool) {
        if set_last_updated {
            player.last_updated = Instant::now();
        }

        match PlayerState::from_mpris_data(
            player.name().to_owned(),
            player.metadata(),
            player.playback_state(),
            player.position(),
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
