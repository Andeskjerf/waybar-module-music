use std::error::Error;
use std::time::Duration;

use dbus::{
    arg::PropMap,
    blocking::{stdintf::org_freedesktop_dbus::Properties, Connection, Proxy},
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

    pub fn get_media_player_proxy<'a>(&'a self, player_id: &'a str) -> Proxy<'a, &'a Connection> {
        self.conn.with_proxy(
            player_id,
            "/org/mpris/MediaPlayer2",
            Duration::from_millis(5000),
        )
    }

    fn call_player_method(
        &self,
        player_id: &str,
        method: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.get_media_player_proxy(player_id);
        proxy.method_call::<(), _, _, _>("org.mpris.MediaPlayer2.Player", method, ())?;
        Ok(())
    }

    pub fn play_mpris_player(&self, player_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.call_player_method(player_id, "Play")?;
        Ok(())
    }

    pub fn pause_mpris_player(&self, player_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.call_player_method(player_id, "Pause")?;
        Ok(())
    }

    pub fn play_pause_mpris_player(
        &self,
        player_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.call_player_method(player_id, "PlayPause")?;
        Ok(())
    }

    pub fn next_mpris_player(&self, player_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.call_player_method(player_id, "Next")?;
        Ok(())
    }

    pub fn previous_mpris_player(&self, player_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.call_player_method(player_id, "Previous")?;
        Ok(())
    }
}

unsafe impl Send for DBusService {}

unsafe impl Sync for DBusService {}
