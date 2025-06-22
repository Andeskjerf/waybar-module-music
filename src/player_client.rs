use crate::models::mpris_metadata::MprisMetadata;

const INTERFACE_PATH: &str = "/org/mpris/MediaPlayer2";
pub const BASE_INTERFACE: &str = "org.mpris.MediaPlayer2";
const INTERFACE_PLAYER: &str = "org.mpris.MediaPlayer2.Player";

pub struct PlayerClient {
    player_name: String,
    metadata: MprisMetadata,
    // proxy: Proxy<'a, &'a Connection>,
}

impl PlayerClient {
    pub fn new(player_name: &str, metadata: MprisMetadata) -> Self {
        Self {
            player_name: player_name.to_owned(),
            metadata,
        }
    }

    pub fn name(&self) -> &str {
        &self.player_name
    }

    pub fn update_metadata(&mut self, metadata: MprisMetadata) {
        self.metadata = metadata;
    }
}
