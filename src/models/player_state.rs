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
}

impl PlayerState {
    pub fn new(
        player_id: String,
        player_name: String,
        artist: String,
        album: String,
        title: String,
        playing: Option<bool>,
    ) -> Self {
        Self {
            player_id,
            player_name,
            artist,
            album,
            title,
            playing,
        }
    }

    pub fn from_mpris_data(
        player_name: String,
        metadata: MprisMetadata,
        playback: Option<MprisPlayback>,
    ) -> Self {
        let player_id = metadata.player_id;
        let artist = metadata.artist.first().unwrap().clone();
        let album = metadata.album.unwrap();
        let title = metadata.title.unwrap();
        let playing = playback
            .unwrap_or_default()
            .playing
            .map(|elem| elem == "Playing");

        PlayerState::new(player_id, player_name, artist, album, title, playing)
    }
}
