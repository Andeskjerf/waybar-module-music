use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use dbus::{blocking::Connection, message::MatchRule, Message};

use crate::models::mpris_metadata::MprisMetadata;

use super::runnable::Runnable;

pub struct DBusMonitor {}

impl DBusMonitor {
    pub fn new() -> Self {
        Self {}
    }

    fn handle_on_match(msg: &Message) -> bool {
        let metadata = MprisMetadata::from_dbus_message(msg);
        println!("{:?}", metadata);
        true
    }

    pub fn begin_monitoring(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = Connection::new_session()?;

        let rule = MatchRule::new()
            .with_type(dbus::MessageType::Signal)
            .with_path("/org/mpris/MediaPlayer2")
            .with_interface("org.freedesktop.DBus.Properties")
            .with_member("PropertiesChanged");

        // TODO: could maybe do something smart with this token
        let token =
            match conn.add_match(rule, move |_: (), _, msg| DBusMonitor::handle_on_match(msg)) {
                Ok(token) => token,
                Err(err) => panic!("DBusMonitor was unable to monitor MPRIS players!\n{err}"),
            };

        loop {
            let result = conn.process(Duration::from_millis(1000)).unwrap();
        }

        Ok(())
    }
}

impl Runnable for DBusMonitor {
    fn run(self: Arc<Self>) -> JoinHandle<()> {
        thread::spawn(move || {
            let _ = self.begin_monitoring();
        })
    }
}
