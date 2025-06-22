use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use bincode::config;
use dbus::{blocking::Connection, message::MatchRule, Message};

use crate::{
    event_bus::{EventBus, EventType},
    models::mpris_metadata::MprisMetadata,
};

use super::runnable::Runnable;

pub struct DBusMonitor {
    event_bus: Arc<Mutex<EventBus>>,
}

impl DBusMonitor {
    pub fn new(event_bus: Arc<Mutex<EventBus>>) -> Self {
        Self { event_bus }
    }

    fn handle_on_match(msg: &Message, event_bus: &Arc<Mutex<EventBus>>) -> bool {
        let metadata = MprisMetadata::from_dbus_message(msg);
        let encoded = bincode::encode_to_vec(&metadata, config::standard());

        match encoded {
            Ok(encoded) => {
                event_bus
                    .lock()
                    .unwrap()
                    .publish(EventType::PlayerSongChanged, encoded);
                println!("{:?}", metadata);
            }
            Err(err) => panic!("failed to encode MPRIS metadata!\n----\n{err}"),
        }
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
        let event_bus = Arc::clone(&self.event_bus);
        let token = match conn.add_match(rule, move |_: (), _, msg| {
            DBusMonitor::handle_on_match(msg, &event_bus)
        }) {
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
