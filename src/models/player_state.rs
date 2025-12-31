use bincode::{Decode, Encode};

use super::{mpris_metadata::MprisMetadata, mpris_playback::MprisPlayback};

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct PlayerState {
    pub player_id: String,
    pub player_name: String,
    pub artist: String,
    pub album: String,
    pub title: String,
    pub playing: Option<bool>,
    pub length: u64,
    pub position: u128,
}

impl PlayerState {
    pub fn new(
        player_id: String,
        player_name: String,
        artist: String,
        album: String,
        title: String,
        playing: Option<bool>,
        length: u64,
        position: u128,
    ) -> Self {
        Self {
            player_id,
            player_name,
            artist,
            album,
            title,
            playing,
            length,
            position,
        }
    }

    pub fn from_mpris_data(
        player_name: String,
        metadata: MprisMetadata,
        playback: Option<MprisPlayback>,
        position: u128,
    ) -> Option<Self> {
        let player_id = metadata.player_id;
        let artist = metadata.artist.first()?.clone();
        let album = metadata.album?;
        let title = metadata.title?;
        let playing = playback
            .unwrap_or_default()
            .playing
            .map(|elem| elem == "Playing");
        let length = metadata.length.unwrap_or(0);

        Some(PlayerState::new(
            player_id,
            player_name,
            artist,
            album,
            title,
            playing,
            length,
            position,
        ))
    }
}
