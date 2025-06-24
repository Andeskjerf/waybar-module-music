use std::{sync::Arc, thread, time::Duration};

use actors::{dbus_monitor::DBusMonitor, display::Display, runnable::Runnable};
use dbus::blocking::Connection;
use event_bus::EventBus;
use player_client::BASE_INTERFACE;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: arg handling with clap
    // TODO: events, like sending signal to play/pause active player
    // TODO: logging

    // let players =  get_players(conn);

    let (event_bus, event_bus_handle) = EventBus::new();
    thread::spawn(move || {
        event_bus.run();
    });

    let actors: Vec<Arc<dyn Runnable>> = vec![
        Arc::new(DBusMonitor::new(event_bus_handle.clone())),
        Arc::new(PlayerManager::new(event_bus_handle.clone())),
        Arc::new(Display::new(event_bus_handle.clone())),
    ];

    let mut handles = vec![];
    for actor in actors {
        handles.push(actor.run());
    }

    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}
