use std::sync::{Arc, Mutex};

use bincode::config;

use crate::{
    event_bus::{EventBus, EventType},
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
    event_bus: Arc<Mutex<EventBus>>,
}

impl PlayerClient {
    pub fn new(
        event_bus: Arc<Mutex<EventBus>>,
        player_name: &str,
        metadata: MprisMetadata,
    ) -> Self {
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

    fn encode_player_state(&self) -> Option<Vec<u8>> {
        match bincode::encode_to_vec(
            PlayerState::from_mpris_data(self.metadata.clone(), self.playback_state.clone()),
            config::standard(),
        ) {
            Ok(encoded) => Some(encoded),
            Err(err) => {
                println!("failed to encode player state\n\n{err}");
                None
            }
        }
    }

    fn publish_state(&self) {
        if let Some(state) = self.encode_player_state() {
            self.event_bus
                .lock()
                .unwrap()
                .publish(EventType::PlayerStateChanged, state);
        }
    }

    pub fn update_metadata(&mut self, metadata: MprisMetadata) {
        self.metadata = metadata;
        self.publish_state();
    }

    pub fn update_plaback_state(&mut self, playback_state: MprisPlayback) {
        self.playback_state = Some(playback_state);
        self.publish_state();
    }
}
