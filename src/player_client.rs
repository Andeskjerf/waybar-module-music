use std::{error::Error, time::Duration};

use dbus::{
    arg::{PropMap, RefArg},
    blocking::{stdintf::org_freedesktop_dbus::Properties, Connection, Proxy},
};

const INTERFACE_PATH: &str = "/org/mpris/MediaPlayer2";
pub const BASE_INTERFACE: &str = "org.mpris.MediaPlayer2";
const INTERFACE_PLAYER: &str = "org.mpris.MediaPlayer2.Player";

pub struct PlayerClient<'a> {
    player_name: String,
    proxy: Proxy<'a, &'a Connection>,
}

impl<'a> PlayerClient<'a> {
    pub fn new(conn: &'a Connection, player_name: &str) -> Self {
        let proxy = conn.with_proxy(
            format!("{BASE_INTERFACE}.{player_name}"),
            INTERFACE_PATH,
            Duration::from_millis(5000),
        );
        Self {
            proxy,
            player_name: player_name.to_owned(),
        }
    }

    pub fn get_all_properties(
        &self,
    ) -> Result<std::collections::HashMap<String, dbus::arg::Variant<Box<dyn RefArg>>>, dbus::Error>
    {
        self.proxy.get_all(INTERFACE_PLAYER)
    }

    pub fn name(&self) -> &str {
        &self.player_name
    }

    pub fn playing(&self) -> Result<bool, dbus::Error> {
        let result: String = self.proxy.get(INTERFACE_PLAYER, "PlaybackStatus")?;
        Ok(result == "Playing")
    }

    fn get_metadata_value(&self, key: &str) -> Result<Box<dyn dbus::arg::RefArg>, Box<dyn Error>> {
        let result: PropMap = self.proxy.get(INTERFACE_PLAYER, "Metadata")?;
        let result = match result.get(key) {
            Some(value) => value,
            None => return Err("no key found".into()),
        };

        Ok(result.0.box_clone())
    }

    pub fn title(&self) -> Result<String, Box<dyn Error>> {
        Ok(self
            .get_metadata_value("xesam:title")?
            .as_str()
            .expect("failed to convert str to String")
            .to_owned())
    }

    pub fn artist(&self) -> Result<String, Box<dyn Error>> {
        let value = self.get_metadata_value("xesam:artist")?;
        let x = Ok(value
            .as_iter()
            .unwrap()
            .next()
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned());
        x
    }
}
