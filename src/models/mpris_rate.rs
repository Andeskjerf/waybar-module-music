use bincode::{Decode, Encode};
use dbus::Message;
use log::error;

#[derive(Debug, Default, Clone, Encode, Decode, PartialEq)]
pub struct MprisRate {
    pub player_id: String,
    pub rate: f64,
}

impl MprisRate {
    pub fn new(player_id: String) -> Self {
        Self {
            player_id,
            rate: 0.0,
        }
    }

    pub fn from_dbus_message(msg: &Message) -> Self {
        let mut result = MprisRate::new(msg.sender().unwrap().to_string());

        if let Some(rate) = msg.get1::<f64>() {
            result.rate = rate;
            return result;
        }

        error!("got to end of MprisRate constructor without returning during construction, this should not happen");
        result
    }
}
