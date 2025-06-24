use bincode::{Decode, Encode};

use super::{mpris_metadata::MprisMetadata, mpris_playback::MprisPlayback};

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct PlayerState {
    pub artist: String,
    pub album: String,
    pub title: String,
    pub playing: Option<bool>,
}

impl PlayerState {
    pub fn new(artist: String, album: String, title: String, playing: Option<bool>) -> Self {
        Self {
            artist,
            album,
            title,
            playing,
        }
    }

    pub fn from_mpris_data(metadata: MprisMetadata, playback: Option<MprisPlayback>) -> Self {
        let artist = metadata.artist.first().unwrap().clone();
        let album = metadata.album.unwrap();
        let title = metadata.title.unwrap();
        let playing = playback
            .unwrap_or_default()
            .playing
            .map(|elem| elem == "Playing");

        PlayerState::new(artist, album, title, playing)
    }
}
