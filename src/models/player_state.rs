use bincode::{Decode, Encode};

use super::{mpris_metadata::MprisMetadata, mpris_playback::MprisPlayback};

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct PlayerState {
    artist: String,
    album: String,
    title: String,
    playing: bool,
}

impl PlayerState {
    pub fn new(artist: String, album: String, title: String, playing: bool) -> Self {
        Self {
            artist,
            album,
            title,
            playing,
        }
    }

    pub fn from_mpris_data(metadata: MprisMetadata, playback: MprisPlayback) -> Self {
        let artist = metadata.artist.first().unwrap().clone();
        let album = metadata.album.unwrap();
        let title = metadata.title.unwrap();
        let playing = playback.playing.unwrap() == "Playing";

        PlayerState::new(artist, album, title, playing)
    }
}
