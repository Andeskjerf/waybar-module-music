use bincode::{Decode, Encode};
use dbus::Message;
use log::error;

#[derive(Debug, Default, Clone, Encode, Decode, PartialEq)]
pub struct MprisSeeked {
    pub player_id: String,
    pub timestamp: i64,
}

impl MprisSeeked {
    pub fn new(player_id: String) -> Self {
        Self {
            player_id,
            timestamp: -1,
        }
    }

    pub fn from_dbus_message(msg: &Message) -> Self {
        let mut result = MprisSeeked::new(msg.sender().unwrap().to_string());

        if let Some(position) = msg.get1::<i64>() {
            result.timestamp = position;
            return result;
        }

        error!("got to end of MprisSeeked constructor without returning during construction, this should not happen");
        result
    }
}
