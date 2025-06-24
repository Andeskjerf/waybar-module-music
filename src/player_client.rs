use bincode::config;

use crate::{
    event_bus::{EventBusHandle, EventType},
    models::{
        mpris_metadata::MprisMetadata, mpris_playback::MprisPlayback, player_state::PlayerState,
    },
};

const INTERFACE_PATH: &str = "/org/mpris/MediaPlayer2";
pub const BASE_INTERFACE: &str = "org.mpris.MediaPlayer2";
const INTERFACE_PLAYER: &str = "org.mpris.MediaPlayer2.Player";

#[derive(Debug)]
pub struct PlayerClient {
    player_name: String,
    metadata: MprisMetadata,
    playback_state: Option<MprisPlayback>,
    // does this make sense?
    // to let the player object itself report its state, or should the manager do that?
    event_bus: EventBusHandle,
}

impl PlayerClient {
    pub fn new(event_bus: EventBusHandle, player_name: &str, metadata: MprisMetadata) -> Self {
        Self {
            event_bus,
            player_name: player_name.to_owned(),
            metadata,
            playback_state: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.player_name
    }

    fn publish_state(&self) {
        match bincode::encode_to_vec(
            PlayerState::from_mpris_data(self.metadata.clone(), self.playback_state.clone()),
            config::standard(),
        ) {
            Ok(encoded) => self
                .event_bus
                .publish(EventType::PlayerStateChanged, encoded),
            Err(err) => {
                println!("failed to encode player state, skipping publish\n\n{err}");
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
}
