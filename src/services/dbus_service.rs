use std::error::Error;
use std::time::Duration;

use dbus::{
    arg::PropMap,
    blocking::{stdintf::org_freedesktop_dbus::Properties, Connection},
    message::MatchRule,
};

use crate::models::{mpris_metadata::MprisMetadata, mpris_playback::MprisPlayback};

pub struct DBusService {
    conn: Connection,
}

impl DBusService {
    pub fn new() -> Self {
        Self {
            conn: Connection::new_session().expect("failed to create DBus connection"),
        }
    }

    fn get_players(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let proxy = self
            .conn
            .with_proxy("org.freedesktop.DBus", "/", Duration::from_millis(5000));

        let (names,): (Vec<String>,) =
            proxy.method_call("org.freedesktop.DBus", "ListNames", ())?;

        let players: Vec<String> = names
            .iter()
            .filter(|name| name.contains("org.mpris.MediaPlayer2"))
            .cloned()
            .collect();

        Ok(players)
    }

    pub fn query_playback_status(&self, player_id: &str) -> Result<MprisPlayback, dbus::Error> {
        let proxy = self.conn.with_proxy(
            player_id,
            "/org/mpris/MediaPlayer2",
            Duration::from_millis(5000),
        );
        let result: String = proxy.get("org.mpris.MediaPlayer2.Player", "PlaybackStatus")?;
        Ok(MprisPlayback::new_with_playing(
            player_id.to_string(),
            result,
        ))
    }

    pub fn query_metadata(&self, player_id: &str) -> Result<MprisMetadata, Box<dyn Error>> {
        let proxy = self.conn.with_proxy(
            player_id,
            "/org/mpris/MediaPlayer2",
            Duration::from_millis(5000),
        );
        let result: PropMap = proxy.get("org.mpris.MediaPlayer2.Player", "Metadata")?;

        Ok(MprisMetadata::from_dbus_propmap(
            player_id.to_string(),
            result,
        ))
    }
}

unsafe impl Send for DBusService {}

unsafe impl Sync for DBusService {}
