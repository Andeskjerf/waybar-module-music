use std::time::Duration;

use dbus::{
    arg::RefArg,
    blocking::{stdintf::org_freedesktop_dbus::Properties, Connection, Proxy},
};

const INTERFACE_PATH: &str = "/org/mpris/MediaPlayer2";
pub const BASE_INTERFACE: &str = "org.mpris.MediaPlayer2";
const INTERFACE_PLAYER: &str = "org.mpris.MediaPlayer2.Player";

pub struct PlayerClient<'a> {
    proxy: Proxy<'a, &'a Connection>,
}

impl<'a> PlayerClient<'a> {
    pub fn new(conn: &'a Connection, player_name: &str) -> Self {
        let proxy = conn.with_proxy(
            format!("{BASE_INTERFACE}.{player_name}"),
            INTERFACE_PATH,
            Duration::from_millis(5000),
        );
        Self { proxy }
    }

    pub fn get_all_properties(
        &self,
    ) -> Result<std::collections::HashMap<String, dbus::arg::Variant<Box<dyn RefArg>>>, dbus::Error>
    {
        self.proxy.get_all(INTERFACE_PLAYER)
    }
}
