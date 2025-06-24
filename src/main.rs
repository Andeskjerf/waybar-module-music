use std::{
    error::Error,
    sync::{Arc, Mutex},
    time::Duration,
};

use actors::{dbus_monitor::DBusMonitor, display::Display, runnable::Runnable};
use dbus::blocking::Connection;
use effects::marquee::Marquee;
use effects::text_effect::TextEffect;
use event_bus::EventBus;
use player_client::{PlayerClient, BASE_INTERFACE};
use player_manager::PlayerManager;
use utils::strip_until_match;

mod actors;
mod effects;
mod event_bus;
mod models;
mod player_client;
mod player_manager;
mod utils;

fn get_players(conn: &Connection) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let proxy = conn.with_proxy("org.freedesktop.DBus", "/", Duration::from_millis(5000));

    let (names,): (Vec<String>,) = proxy.method_call("org.freedesktop.DBus", "ListNames", ())?;

    let players: Vec<String> = names
        .iter()
        .filter(|name| name.contains(BASE_INTERFACE))
        .fold(vec![], |mut a, elem| {
            a.push(strip_until_match(BASE_INTERFACE, elem));
            a
        });

    Ok(players)
}

/// If the artist name is leading the title, we remove the artist from the title
fn sanitize_title(title: String, artist: &str) -> String {
    if title
        .to_lowercase()
        .contains(&format!("{} -", &artist.to_lowercase()))
    {
        return strip_until_match(&format!("{} -", artist), &title).to_owned();
    }
    title
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_bus: Arc<Mutex<EventBus>> = Arc::new(Mutex::new(EventBus::new()));
    let actors: Vec<Arc<dyn Runnable>> = vec![
        Arc::new(DBusMonitor::new(Arc::clone(&event_bus))),
        Arc::new(PlayerManager::new(Arc::clone(&event_bus))),
        Arc::new(Display::new(Arc::clone(&event_bus))),
    ];

    let mut handles = vec![];
    for actor in actors {
        handles.push(actor.run());
    }

    for handle in handles {
        let _ = handle.join();
    }

    Ok(())

    // TODO: arg handling with clap
    // TODO: events, like sending signal to play/pause active player
    // TODO: logging

    // TODO: thread to monitor players being opened or exited
    // let players = get_players(&conn)?
    //     .iter()
    //     .map(|p| PlayerClient::new(&conn, p))
    //     .collect::<Vec<PlayerClient>>();
}
