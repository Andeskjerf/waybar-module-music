use crate::{
    models::{mpris_metadata::MprisMetadata, mpris_playback::MprisPlayback},
    utils::time::get_current_timestamp,
};

#[derive(Debug, Clone)]
pub struct PlayerClient {
    player_name: String,
    metadata: MprisMetadata,
    playback_state: Option<MprisPlayback>,
    current_position: u128,
    /// Timestamp for metadata or playback updates
    pub last_updated: u64,
    /// Timestamp for last timer event, like song progressing in time
    last_tick: Option<u64>,
}

impl PlayerClient {
    pub fn new(player_name: String, metadata: MprisMetadata) -> Self {
        Self {
            player_name,
            metadata,
            current_position: 0,
            last_updated: 0,
            last_tick: None,
            playback_state: None,
        }
    }

    pub fn name(&self) -> &String {
        &self.player_name
    }

    pub fn get_id(&self) -> String {
        self.metadata().player_id
    }

    pub fn metadata(&self) -> MprisMetadata {
        self.metadata.clone()
    }

    pub fn playback_state(&self) -> Option<MprisPlayback> {
        self.playback_state.clone()
    }

    pub fn position(&self) -> u128 {
        self.current_position
    }

    pub fn playing(&self) -> bool {
        self.playback_state
            .as_ref()
            .map(|elem| elem.is_playing())
            .unwrap_or(false)
    }

    pub fn update_metadata(&mut self, metadata: MprisMetadata) {
        self.metadata = metadata;
    }

    pub fn update_playback_state(&mut self, playback_state: MprisPlayback) {
        self.playback_state = Some(playback_state);
    }

    pub fn update_position(&mut self, position: u128) {
        self.current_position = position;
        self.last_tick = Some(get_current_timestamp());
    }
}
