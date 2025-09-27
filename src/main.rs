use std::{fs::File, sync::Arc, thread};

use clap::Parser;
use event_bus::EventBus;
use interfaces::dbus_client::DBusClient;
use log::info;
use models::args::Args;
use services::{
    dbus_monitor::DBusMonitor, display::Display, player_manager::PlayerManager, runnable::Runnable,
};
use simplelog::{CombinedLogger, Config as LogConfig, WriteLogger};

mod effects;
mod event_bus;
mod helpers;
mod interfaces;
mod models;
mod services;

fn init_logger(debug: bool) -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = helpers::dir::get_and_create_dir(dirs::cache_dir)?;
    let log_path = cache_dir.join("app.log");

    CombinedLogger::init(vec![WriteLogger::new(
        if debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        },
        LogConfig::default(),
        File::create(log_path)?,
    )])?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arc::new(Args::parse());

    // TODO: events, like sending signal to play/pause active player
    init_logger(args.debug)?;

    let (event_bus, event_bus_handle) = EventBus::new();
    thread::spawn(move || {
        event_bus.run();
    });

    let dbus_client = Arc::new(DBusClient::new());

    let services: Vec<Arc<dyn Runnable>> = vec![
        Arc::new(DBusMonitor::new(
            args.clone(),
            event_bus_handle.clone(),
            dbus_client.clone(),
        )),
        Arc::new(PlayerManager::new(
            event_bus_handle.clone(),
            dbus_client.clone(),
        )),
        Arc::new(Display::new(args.clone(), event_bus_handle.clone())),
    ];

    let mut handles = vec![];
    for service in services {
        handles.push(service.run());
    }

    for handle in handles {
        let _ = handle.join();
    }

    info!("all threads stopped, stopping...");

    Ok(())
}
