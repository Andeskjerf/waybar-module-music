use bincode::{Decode, Encode};
use dbus::Message;
use log::error;

#[derive(Debug, Default, Clone, Encode, Decode, PartialEq)]
pub struct MprisIdentity {
    pub player_id: String,
    pub identity: String,
}

impl MprisIdentity {
    pub fn new(player_id: String) -> Self {
        Self {
            player_id,
            identity: String::new(),
        }
    }

    pub fn from_dbus_message(msg: &Message) -> Self {
        let mut result = MprisIdentity::new(msg.sender().unwrap().to_string());

        for elem in msg.iter_init() {
            if let Some(args) = elem.as_iter() {
                if let Some(kv) = args.collect::<Vec<_>>().chunks(2).next() {
                    if let (Some(key), Some(value)) = (kv[0].as_str(), kv[1].as_str()) {
                        if key != "Rate" {
                            error!("tried to create MprisRate but message does not conform to expected format");
                            return result;
                        }
                        result.identity = String::from(value);
                        return result;
                    } else {
                        error!("MprisIdentity: got unexpected key-value pair, types do not conform to expected format: {:?}", kv);
                        return result;
                    }
                }
            };
        }

        error!("got to end of MprisIdentity constructor without returning during construction, this should not happen");
        result
    }
}
