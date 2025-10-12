use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use bincode::config;
use dbus::{blocking::Connection, message::MatchRule, Message};
use log::{debug, error, info, warn};

use crate::{
    event_bus::{EventBusHandle, EventType},
    interfaces::dbus_client::DBusClient,
    models::{
        args::Args, mpris_metadata::MprisMetadata, mpris_playback::MprisPlayback,
        mpris_seeked::MprisSeeked,
    },
};

use super::runnable::Runnable;

pub struct DBusMonitor {
    args: Arc<Args>,
    event_bus: EventBusHandle,
    dbus_client: Arc<DBusClient>,
}

// TODO: we also need to discover players when we run the program initially
// who should handle that? the monitor, or a new service?

impl DBusMonitor {
    pub fn new(args: Arc<Args>, event_bus: EventBusHandle, dbus_client: Arc<DBusClient>) -> Self {
        Self {
            args,
            event_bus,
            dbus_client,
        }
    }

    // TODO: the dbus client should handle parsing like this
    fn determine_event_type(msg: &Message) -> EventType {
        // some events can be determined by simply checking their member
        // while others may require more manual parsing
        if let Some(member) = msg.member() {
            if member.to_lowercase().as_str() == "seeked" {
                return EventType::Seeked;
            }
        }

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

    fn should_handle_sender(args: Arc<Args>, dbus_client: Arc<DBusClient>, msg: &Message) -> bool {
        if args.whitelist.is_empty() {
            return true;
        }

        let sender = match msg.sender() {
            Some(sender) => sender.to_string(),
            None => {
                error!("failed to determine sender, handling it anyway");
                return true;
            }
        };

        match dbus_client.query_mediaplayer_identity(&sender) {
            Ok(identity) => args
                .whitelist
                .iter()
                .any(|w| identity.to_lowercase().contains(&w.to_lowercase())),
            Err(err) => {
                error!("failed to query media player identity, handling it anyway: {err}");
                true
            }
        }
    }

    fn handle_on_match(
        args: Arc<Args>,
        dbus_client: Arc<DBusClient>,
        msg: &Message,
        event_bus: EventBusHandle,
    ) -> bool {
        if !DBusMonitor::should_handle_sender(args, dbus_client, msg) {
            debug!("ignoring sender, not in whitelist");
            return true;
        }

        let event_type = DBusMonitor::determine_event_type(msg);
        // TODO: the MPRIS objects could potentially use a common interface to make this cleaner
        let encoded = match event_type {
            EventType::PlayerSongChanged => {
                bincode::encode_to_vec(MprisMetadata::from_dbus_message(msg), config::standard())
            }
            EventType::PlaybackChanged => {
                bincode::encode_to_vec(MprisPlayback::from_dbus_message(msg), config::standard())
            }
            EventType::Seeked => {
                bincode::encode_to_vec(MprisSeeked::from_dbus_message(msg), config::standard())
            }
            EventType::ParseError => {
                warn!("failed to parse message. skipping");
                return true;
            }
            EventType::Unknown(found_arg) => {
                warn!("got unknown event with name '{found_arg}'. skipping");
                return true;
            }
            _ => return true, // ignore other messages
        };

        match encoded {
            Ok(encoded) => event_bus.publish(event_type, encoded),
            Err(err) => error!("failed to encode MPRIS data: {err}"),
        }
        true
    }

    // TODO: some of this should be handled by DBusClient
    pub fn begin_monitoring(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = Connection::new_session()?;

        let rules: Vec<MatchRule> = vec![
            MatchRule::new()
                .with_type(dbus::MessageType::Signal)
                .with_path("/org/mpris/MediaPlayer2")
                .with_interface("org.freedesktop.DBus.Properties")
                .with_member("PropertiesChanged"),
            MatchRule::new()
                .with_type(dbus::MessageType::Signal)
                .with_path("/org/mpris/MediaPlayer2")
                .with_interface("org.mpris.MediaPlayer2.Player")
                .with_member("Seeked"),
        ];

        for rule in rules {
            let event_bus = self.event_bus.clone();
            let dbus_client = self.dbus_client.clone();
            let args = self.args.clone();
            match conn.add_match(rule, move |_: (), _, msg| {
                DBusMonitor::handle_on_match(
                    args.clone(),
                    dbus_client.clone(),
                    msg,
                    event_bus.clone(),
                )
            }) {
                Ok(token) => token,
                Err(err) => {
                    error!("DBusMonitor was unable to monitor MPRIS players: {err}");
                    return Err(err.into());
                }
            };
        }

        loop {
            match conn.process(Duration::from_millis(1000)) {
                Ok(res) => (),
                Err(err) => warn!("failed to process DBus connection: {err}"),
            }
        }

        Ok(())
    }
}

impl Runnable for DBusMonitor {
    fn run(self: Arc<Self>) -> JoinHandle<()> {
        thread::spawn(move || {
            info!("starting DBusMonitor thread");
            let result = self.begin_monitoring();
            info!("DBusMonitor thread is stopping");
        })
    }
}
