use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use bincode::config;
use dbus::{blocking::Connection, message::MatchRule, Message};
use log::{error, warn};

use crate::{
    event_bus::{EventBusHandle, EventType},
    interfaces::dbus_client::DBusClient,
    models::{mpris_metadata::MprisMetadata, mpris_playback::MprisPlayback},
};

use super::runnable::Runnable;

pub struct DBusMonitor {
    event_bus: EventBusHandle,
    dbus_client: Arc<DBusClient>,
}

// TODO: we also need to discover players when we run the program initially
// who should handle that? the monitor, or a new actor?

impl DBusMonitor {
    pub fn new(event_bus: EventBusHandle, dbus_client: Arc<DBusClient>) -> Self {
        Self {
            event_bus,
            dbus_client,
        }
    }

    fn determine_event_type(msg: &Message) -> EventType {
        for elem in msg.iter_init() {
            if let Some(mut args) = elem.as_iter() {
                if let Some(arg_type) = args.next() {
                    match arg_type.as_str() {
                        Some(arg_type) => match arg_type {
                            "Metadata" => return EventType::PlayerSongChanged,
                            "PlaybackStatus" => return EventType::PlaybackChanged,
                            _ => return EventType::Unknown(arg_type.to_string()),
                        },
                        None => return EventType::ParseError,
                    };
                };
            };
        }

        error!("got to end of message iteration without finding event type and without error, this should not happen");
        EventType::ParseError
    }

    fn handle_on_match(msg: &Message, event_bus: EventBusHandle) -> bool {
        let event_type = DBusMonitor::determine_event_type(msg);
        let encoded = match event_type {
            EventType::PlayerSongChanged => {
                bincode::encode_to_vec(MprisMetadata::from_dbus_message(msg), config::standard())
            }
            EventType::PlaybackChanged => {
                bincode::encode_to_vec(MprisPlayback::from_dbus_message(msg), config::standard())
            }
            EventType::ParseError => {
                warn!("failed to parse message. skipping");
                return false;
            }
            EventType::Unknown(found_arg) => {
                warn!("got unknown event with name '{found_arg}'. skipping");
                return false;
            }
            _ => return false, // ignore other messages
        };

        match encoded {
            Ok(encoded) => event_bus.publish(event_type, encoded),
            Err(err) => error!("failed to encode MPRIS data!\n----\n{err}"),
        }
        true
    }

    // TODO: some of this should be handled by DBusClient
    pub fn begin_monitoring(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = Connection::new_session()?;

        let rule = MatchRule::new()
            .with_type(dbus::MessageType::Signal)
            .with_path("/org/mpris/MediaPlayer2")
            .with_interface("org.freedesktop.DBus.Properties")
            .with_member("PropertiesChanged");

        // TODO: could maybe do something smart with this token
        let event_bus = self.event_bus.clone();
        let token = match conn.add_match(rule, move |_: (), _, msg| {
            DBusMonitor::handle_on_match(msg, event_bus.clone())
        }) {
            Ok(token) => token,
            Err(err) => {
                error!("DBusMonitor was unable to monitor MPRIS players!\n{err}");
                return Err(err.into());
            }
        };

        loop {
            match conn.process(Duration::from_millis(1000)) {
                Ok(res) => (),
                Err(err) => warn!("failed to process DBus connection\n{err}"),
            }
        }

        Ok(())
    }
}

impl Runnable for DBusMonitor {
    fn run(self: Arc<Self>) -> JoinHandle<()> {
        thread::spawn(move || {
            let result = self.begin_monitoring();
        })
    }
}
