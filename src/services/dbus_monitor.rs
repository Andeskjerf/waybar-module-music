use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use bincode::config;
use dbus::{arg::RefArg, blocking::Connection, message::MatchRule, Message};
use log::{debug, error, info, warn};

use crate::{
    event_bus::{EventBusHandle, EventType},
    interfaces::dbus_client::DBusClient,
    models::{
        args::Args, mpris_identity::MprisIdentity, mpris_metadata::MprisMetadata,
        mpris_playback::MprisPlayback, mpris_rate::MprisRate, mpris_seeked::MprisSeeked,
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
    fn determine_event_type(property: String) -> EventType {
        match property.to_lowercase().as_str() {
            "metadata" => return EventType::PlayerSongChanged,
            "playbackstatus" => return EventType::PlaybackChanged,
            "seeked" => return EventType::Seeked,
            "rate" => return EventType::Rate,
            _ => (),
        }

        error!("got to end of message iteration without finding event type and without error, this should not happen");
        EventType::ParseError
    }

    // FIXME: very nested...
    fn get_signal_property_keys(msg: &Message) -> Vec<String> {
        let mut result = vec![];
        for elem in msg.iter_init() {
            if let Some(args) = elem.as_iter() {
                for arg in args {
                    if let Some(arg_str) = arg.as_str() {
                        result.push(String::from(arg_str));
                    }
                }
            };
        }
        result
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

        debug!("dbus_monitor msg: {:?}", msg);
        let mut property_keys = DBusMonitor::get_signal_property_keys(msg);
        if let Some(member) = msg.member() {
            property_keys.push(member.to_string());
        }

        for key in property_keys {
            let event_type = DBusMonitor::determine_event_type(key);
            let encoded = match event_type {
                EventType::PlayerSongChanged => bincode::encode_to_vec(
                    MprisMetadata::from_dbus_message(msg),
                    config::standard(),
                ),
                EventType::PlaybackChanged => bincode::encode_to_vec(
                    MprisPlayback::from_dbus_message(msg),
                    config::standard(),
                ),
                EventType::Seeked => {
                    bincode::encode_to_vec(MprisSeeked::from_dbus_message(msg), config::standard())
                }
                EventType::Rate => {
                    bincode::encode_to_vec(MprisRate::from_dbus_message(msg), config::standard())
                }
                EventType::Identity => bincode::encode_to_vec(
                    MprisIdentity::from_dbus_message(msg),
                    config::standard(),
                ),
                EventType::ParseError => {
                    warn!("failed to parse message. skipping");
                    continue;
                }
                EventType::Unknown(found_arg) => {
                    warn!("got unknown event with name '{found_arg}'. skipping");
                    continue;
                }
                _ => continue, // ignore other messages
            };

            match encoded {
                Ok(encoded) => event_bus.publish(event_type, encoded),
                Err(err) => error!("failed to encode MPRIS data: {err}"),
            }
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
