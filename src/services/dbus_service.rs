use std::time::Duration;

use dbus::blocking::Connection;

use crate::utils::strip_until_match;

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

    fn get_unique_name(&self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let proxy = self
            .conn
            .with_proxy("org.freedesktop.DBus", "/", Duration::from_millis(5000));

        let (unique_name,): (String,) = proxy.method_call(
            "org.freedesktop.DBus",
            "GetNameOwner",
            (format!("s {name}"),),
        )?;

        Ok(unique_name)
    }
}
