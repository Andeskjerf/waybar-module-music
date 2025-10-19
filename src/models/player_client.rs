use std::time::{SystemTime, UNIX_EPOCH};

use bincode::config;
use log::warn;

use crate::{
    event_bus::{EventBusHandle, EventType},
    models::{
        mpris_metadata::MprisMetadata, mpris_playback::MprisPlayback, player_state::PlayerState,
    },
};

use super::mpris_seeked::MprisSeeked;

#[derive(Debug)]
pub struct PlayerClient {
    player_name: String,
    metadata: MprisMetadata,
    playback_state: Option<MprisPlayback>,
    current_position: i64,
    /// Timestamp for metadata or playback updates
    pub last_updated: u64,
    /// Timestamp for last timer event, like song progressing in time
    last_tick: i64,
    // does this make sense?
    // to let the player object itself report its state, or should the manager do that?
    event_bus: EventBusHandle,
}

impl PlayerClient {
    pub fn new(player_name: String, event_bus: EventBusHandle, metadata: MprisMetadata) -> Self {
        Self {
            player_name,
            event_bus,
            metadata,
            current_position: -1,
            last_updated: 0,
            last_tick: -1,
            playback_state: None,
        }
    }

    pub fn playing(&self) -> bool {
        self.playback_state
            .as_ref()
            .map(|elem| elem.is_playing())
            .unwrap_or(false)
    }

    pub fn publish_state(&mut self) {
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("failed to get current timestamp")
            .as_secs();

        match PlayerState::from_mpris_data(
            self.player_name.clone(),
            self.metadata.clone(),
            self.playback_state.clone(),
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
                warn!("failed to construct PlayerState. did we get empty metadata? skipping publish: {:?}", self.metadata);
            }
        }
    }

    pub fn update_metadata(&mut self, metadata: MprisMetadata) {
        self.metadata = metadata;
        self.publish_state();
    }

    pub fn update_playback_state(&mut self, playback_state: MprisPlayback) {
        self.playback_state = Some(playback_state);
        self.publish_state();
    }

    pub fn update_position(&mut self, mpris_seeked: MprisSeeked) {
        self.current_position = mpris_seeked.timestamp;
    }
}
