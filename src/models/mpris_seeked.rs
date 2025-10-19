use bincode::{Decode, Encode};
use dbus::Message;
use log::error;

#[derive(Debug, Default, Clone, Encode, Decode, PartialEq)]
pub struct MprisSeeked {
    pub player_id: String,
    pub position: u128,
}

impl MprisSeeked {
    pub fn new(player_id: String) -> Self {
        Self {
            player_id,
            position: 0,
        }
    }

    pub fn from_dbus_message(msg: &Message) -> Self {
        let mut result = MprisSeeked::new(msg.sender().unwrap().to_string());

        if let Some(position) = msg.get1::<i64>() {
            result.position = position as u128;
            return result;
        }

        error!("got to end of MprisSeeked constructor without returning during construction, this should not happen");
        result
    }
}
