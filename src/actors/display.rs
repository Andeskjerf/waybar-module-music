use bincode::config;

use crate::{
    effects::{marquee::Marquee, text_effect::TextEffect},
    event_bus::{EventBusHandle, EventType},
    models::player_state::PlayerState,
    utils::strip_until_match,
};

use super::runnable::Runnable;
use std::{
    sync::{Arc, Mutex},
    thread::{self},
    time::Duration,
};

pub struct Display {
    player_state: Arc<Mutex<Option<PlayerState>>>,
    event_bus: EventBusHandle,
}

impl Display {
    pub fn new(event_bus: EventBusHandle) -> Self {
        Self {
            player_state: Arc::new(Mutex::new(None)),
            event_bus,
        }
    }

    fn init_threads(self: Arc<Self>) {
        {
            let display = Arc::clone(&self);
            let player_state = Arc::clone(&self.player_state);
            thread::spawn(move || {
                display.listen_player_state(player_state);
            });
        }
        {
            let display = Arc::clone(&self);
            let player_state = Arc::clone(&self.player_state);
            thread::spawn(move || {
                display.display(player_state);
            });
        }
    }

    fn listen_player_state(&self, player_state: Arc<Mutex<Option<PlayerState>>>) {
        let rx = self
            .event_bus
            .subscribe(EventType::PlayerStateChanged)
            .expect("failed to subscribe to PlayerStateChanged");

        loop {
            let msg = rx.recv();
            let (state, _): (PlayerState, usize) = match msg {
                Ok(encoded) => {
                    bincode::decode_from_slice(&encoded[..], config::standard()).unwrap()
                }
                Err(err) => {
                    println!("failed to decode message in Display!\n----\n{err}");
                    continue;
                }
            };

            *player_state.lock().unwrap() = Some(state);
        }
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

    fn get_class(&self) -> &str {
        let lock = self.player_state.lock().unwrap();
        if let Some(state) = &(*lock) {
            if let Some(playing) = state.playing {
                if playing {
                    return "playing";
                } else {
                    return "paused";
                }
            }
        }

        "n/a"
    }

    /// Create the final output JSON, in the format that Waybar expects
    fn format_json_output(&self, text: &str) -> String {
        let class = self.get_class();
        format!(
            "{{\"text\": \"{}\", \"tooltip\": \"{}\", \"class\": \"{}\", \"alt\": \"{}\"}}",
            text, "", class, ""
        )
    }

    fn display(&self, player_state: Arc<Mutex<Option<PlayerState>>>) {
        let max_width = 20;
        let apply_effects_ms = 200;
        let mut marquee =
            TextEffect::new(apply_effects_ms).with_effect(Box::new(Marquee::new(max_width, true)));

        // TODO: only update display if there's a state change or time to run an effect
        loop {
            std::thread::sleep(Duration::from_millis(apply_effects_ms as u64));
            let lock = player_state.lock().unwrap();

            if lock.is_none() {
                // FIXME: not nice to have to drop the lock since we lock again in format_json_output
                drop(lock);
                println!("{}", self.format_json_output("Nothing happening"));
                continue;
            }

            let player_state = lock.as_ref().unwrap();

            let icon = match player_state.playing.unwrap_or(false) {
                true => "",
                false => "",
            };

            let artist = &player_state.artist;
            let title = &player_state.title;

            println!("{artist}");
            let formatted = if title.is_empty() && artist.is_empty() {
                "No data".to_owned()
            } else {
                format!(
                    "{}{}",
                    if artist.is_empty() {
                        String::new()
                    } else {
                        format!("{} - ", artist)
                    },
                    marquee.draw(&Display::sanitize_title(title.clone(), artist))
                )
            };

            drop(lock);

            println!(
                "{}",
                self.format_json_output(&format!("[ {icon} ] {formatted}"))
            )
        }
    }
}

impl Runnable for Display {
    fn run(self: std::sync::Arc<Self>) -> std::thread::JoinHandle<()> {
        thread::spawn(move || {
            self.init_threads();
        })
    }
}
