use std::{
    fs::{self, File},
    sync::Arc,
    thread,
};

use event_bus::EventBus;
use interfaces::dbus_client::DBusClient;
use services::{
    dbus_monitor::DBusMonitor, display::Display, player_manager::PlayerManager, runnable::Runnable,
};
use simplelog::{CombinedLogger, Config, WriteLogger};

mod effects;
mod event_bus;
mod interfaces;
mod models;
mod services;
mod utils;

fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    let app_cache = dirs::cache_dir()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "could not get user's cache directory",
            )
        })?
        .join("waybar-module-music");

    match fs::create_dir(&app_cache) {
        Ok(_) => (),
        Err(err) => eprintln!("{err}"),
    };

    let log_path = app_cache.join("app.log");

    CombinedLogger::init(vec![WriteLogger::new(
        log::LevelFilter::Debug,
        Config::default(),
        File::create(log_path)?,
    )])?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: arg handling with clap
    // TODO: events, like sending signal to play/pause active player
    // TODO: logging
    init_logger()?;

    let (event_bus, event_bus_handle) = EventBus::new();
    thread::spawn(move || {
        event_bus.run();
    });

    let dbus_client = Arc::new(DBusClient::new());

    let services: Vec<Arc<dyn Runnable>> = vec![
        Arc::new(DBusMonitor::new(
            event_bus_handle.clone(),
            dbus_client.clone(),
        )),
        Arc::new(PlayerManager::new(
            event_bus_handle.clone(),
            dbus_client.clone(),
        )),
        Arc::new(Display::new(event_bus_handle.clone())),
    ];

    let mut handles = vec![];
    for actor in services {
        handles.push(actor.run());
    }

    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}
