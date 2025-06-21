use crate::player_client::PlayerClient;
use dbus::{blocking::Connection, message::MatchRule};
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

pub struct PlayerManager<'a> {
    players: Vec<PlayerClient<'a>>,
    monitor_handle: Option<JoinHandle<()>>,
}

impl<'a> PlayerManager<'a> {
    // pub fn new(conn: &Connection) -> Self {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let mut manager = Self {
            players: vec![],
            monitor_handle: None,
        };
        manager.monitor_handle = Some(manager.begin_monitoring(conn));
        manager
    }

    fn begin_monitoring(&mut self, conn: Arc<Mutex<Connection>>) -> JoinHandle<()> {
        let rule = MatchRule::new()
            .with_type(dbus::MessageType::Signal)
            .with_path("/org/mpris/MediaPlayer2")
            .with_interface("org.freedesktop.DBus.Properties")
            .with_member("PropertiesChanged");

        // TODO: could maybe do something smart with this token
        let token = match conn
            .lock()
            .expect("failed to lock Connection mutex on begin_monitoring")
            .add_match(rule, |_: (), _, msg| {
                println!("{:?}", msg);
                true
            }) {
            Ok(token) => token,
            Err(err) => panic!("PlayerManager was unable to monitor MPRIS players!\n{err}"),
        };

        // let conn = Arc::clone(&conn);
        thread::spawn(move || loop {
            conn.lock()
                .expect("failed to lock Connection mutex in PlayerManager")
                .process(Duration::from_millis(1000))
                .unwrap();
        })
    }
}
