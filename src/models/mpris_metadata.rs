use bincode::{Decode, Encode};
use dbus::{
    arg::{RefArg, Variant},
    Message,
};

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct MprisMetadata {
    pub player_id: String,
    album_artist: Vec<String>,
    content_created: Option<String>,
    last_used: Option<String>,
    genre: Vec<String>,
    pub artist: Vec<String>,
    pub title: Option<String>,
    use_count: Option<u32>,
    pub album: Option<String>,
    disc_number: Option<u8>,
    track_number: Option<u8>,
    length: Option<u64>,
    comment: Vec<String>,
    track_id: Option<String>,
    art_url: Option<String>,
}

impl MprisMetadata {
    pub fn new(sender: String) -> Self {
        Self {
            player_id: sender.to_string(),
            album_artist: vec![],
            content_created: None,
            last_used: None,
            genre: vec![],
            artist: vec![],
            title: None,
            use_count: None,
            album: None,
            disc_number: None,
            track_number: None,
            length: None,
            comment: vec![],
            track_id: None,
            art_url: None,
        }
    }

    fn refarg_to_vec_string(value: Variant<Box<dyn RefArg>>) -> Vec<String> {
        let mut result = vec![];

        // FIXME: not pretty!! must be a nicer way to unpack these values
        for e in value.as_iter().unwrap().collect::<Vec<_>>() {
            for e in e.as_iter().unwrap().collect::<Vec<_>>() {
                for e in e.as_iter().unwrap().collect::<Vec<_>>() {
                    result.push(e.as_str().unwrap().to_string());
                }
            }
        }

        result
    }

    fn refarg_to_string(value: Variant<Box<dyn RefArg>>) -> Option<String> {
        value.as_str().map(|elem| elem.to_string())
    }

    fn set_field(&mut self, key: &str, value: Variant<Box<dyn RefArg>>) {
        match key {
            "xesam:comment" => self.comment = MprisMetadata::refarg_to_vec_string(value),
            "xesam:contentCreated" => self.content_created = MprisMetadata::refarg_to_string(value),
            "xesam:album" => self.album = MprisMetadata::refarg_to_string(value),
            "xesam:albumArtist" => self.album_artist = MprisMetadata::refarg_to_vec_string(value),
            "xesam:artist" => self.artist = MprisMetadata::refarg_to_vec_string(value),
            "xesam:discNumber" => self.disc_number = value.as_f64().map(|elem| elem as u8),
            "xesam:lastUsed" => self.last_used = MprisMetadata::refarg_to_string(value),
            "xesam:useCount" => self.use_count = value.as_f64().map(|elem| elem as u32),
            "xesam:trackNumber" => self.track_number = value.as_f64().map(|elem| elem as u8),
            "xesam:title" => self.title = MprisMetadata::refarg_to_string(value),
            "xesam:genre" => self.genre = MprisMetadata::refarg_to_vec_string(value),
            "mpris:length" => self.length = value.as_i64().map(|elem| elem as u64),
            "mpris:trackid" => self.track_id = MprisMetadata::refarg_to_string(value),
            "mpris:artUrl" => self.art_url = MprisMetadata::refarg_to_string(value),
            _ => (),
        }
    }

    pub fn from_dbus_message(msg: &Message) -> Self {
        let mut result = MprisMetadata::new(msg.sender().unwrap().to_string());

        // FIXME: this is ugly...
        for msg in msg.iter_init() {
            if let Some(dict) = msg.as_iter() {
                for chunk in dict.collect::<Vec<_>>().chunks(2) {
                    // only handle key-value pairs
                    if chunk.len() != 2 {
                        continue;
                    }

                    if let (Some(key), value) = (chunk[0].as_str(), &chunk[1]) {
                        if key != "Metadata" {
                            continue;
                        }

                        if let Some(metadata_dict) = value.as_iter() {
                            for m in metadata_dict.collect::<Vec<_>>().iter() {
                                if let Some(metadata_item) = m.as_iter() {
                                    for metadata_item_chunk in
                                        metadata_item.collect::<Vec<_>>().chunks(2)
                                    {
                                        if metadata_item_chunk.len() != 2 {
                                            continue;
                                        }

                                        if let Some(meta_key) = metadata_item_chunk[0].as_str() {
                                            result.set_field(
                                                meta_key,
                                                Variant(metadata_item_chunk[1].box_clone()),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        result
    }
}
