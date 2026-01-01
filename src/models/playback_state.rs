use bincode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

impl PlaybackState {
    pub fn from_string(text: &str) -> Option<Self> {
        match text.to_lowercase().as_str() {
            "playing" => Some(PlaybackState::Playing),
            "paused" => Some(PlaybackState::Paused),
            "stopped" => Some(PlaybackState::Stopped),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            PlaybackState::Playing => String::from("playing"),
            PlaybackState::Paused => String::from("paused"),
            PlaybackState::Stopped => String::from("stopped"),
        }
    }
}
