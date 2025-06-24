use std::{sync::Arc, thread};

use actors::{dbus_monitor::DBusMonitor, display::Display, runnable::Runnable};
use event_bus::EventBus;
use player_manager::PlayerManager;
use services::dbus_service::DBusService;

mod actors;
mod effects;
mod event_bus;
mod models;
mod player_client;
mod player_manager;
mod services;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: arg handling with clap
    // TODO: events, like sending signal to play/pause active player
    // TODO: logging

    let (event_bus, event_bus_handle) = EventBus::new();
    thread::spawn(move || {
        event_bus.run();
    });

    let dbus_service = Arc::new(DBusService::new());

    let actors: Vec<Arc<dyn Runnable>> = vec![
        Arc::new(DBusMonitor::new(
            event_bus_handle.clone(),
            dbus_service.clone(),
        )),
        Arc::new(PlayerManager::new(
            event_bus_handle.clone(),
            dbus_service.clone(),
        )),
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
