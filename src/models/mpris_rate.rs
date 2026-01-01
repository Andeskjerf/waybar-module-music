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

        for elem in msg.iter_init() {
            if let Some(args) = elem.as_iter() {
                if let Some(kv) = args.collect::<Vec<_>>().chunks(2).next() {
                    if let (Some(key), Some(value)) = (kv[0].as_str(), kv[1].as_f64()) {
                        if key != "Rate" {
                            error!("tried to create MprisRate but message does not conform to expected format");
                            return result;
                        }
                        result.rate = value;
                        return result;
                    } else {
                        error!("got unexpected key-value pair, types do not conform to expected format");
                        return result;
                    }
                }
            };
        }

        result
    }
}
