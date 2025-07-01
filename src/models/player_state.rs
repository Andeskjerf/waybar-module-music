use std::fmt::Display;

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
    ) -> Option<Self> {
        let player_id = metadata.player_id;
        let artist = metadata.artist.first()?.clone();
        let album = metadata.album?;
        let title = metadata.title?;
        let playing = playback
            .unwrap_or_default()
            .playing
            .map(|elem| elem == "Playing");

        Some(PlayerState::new(
            player_id,
            player_name,
            artist,
            album,
            title,
            playing,
        ))
    }
}

impl Display for PlayerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "player_id: {}\nplayer_name: {}\nartist: {}\nalbum: {}\ntitle: {}\nplaying: {:?}",
            self.player_id, self.player_name, self.artist, self.album, self.title, self.playing
        )
    }
}
