use bincode::config;

use crate::{
    effects::{marquee::Marquee, text_effect::TextEffect},
    event_bus::{EventBusHandle, EventType},
    models::player_state::PlayerState,
    utils::strip_until_match,
};

use super::runnable::Runnable;
use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread::{self},
};

enum DisplayMessages {
    PlayerStateChanged(PlayerState),
    AnimationDue,
}

pub struct Display {
    event_bus: EventBusHandle,
}

impl Display {
    pub fn new(event_bus: EventBusHandle) -> Self {
        Self { event_bus }
    }

    fn init_threads(self: Arc<Self>) {
        let (tx, rx) = mpsc::channel();
        {
            let tx = tx.clone();
            let event_rx = self
                .event_bus
                .subscribe(EventType::PlayerStateChanged)
                .expect("failed to subscribe to PlayerStateChanged");
            thread::spawn(move || {
                Display::listen_player_state(event_rx, tx);
            });
        }

        let max_width = 20;
        let apply_effects_ms = 200;

        let (text_effect, effect_rx) = TextEffect::new(apply_effects_ms);
        let mut text_effect = text_effect.with_effect(Box::new(Marquee::new(max_width, true)));

        {
            let tx = tx.clone();
            thread::spawn(move || {
                Display::listen_text_effect(tx, effect_rx);
            });
        }

        self.listen_for_updates(rx, &mut text_effect);
    }

    fn listen_player_state(rx: Receiver<Vec<u8>>, tx: Sender<DisplayMessages>) {
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

            if let Err(err) = tx.send(DisplayMessages::PlayerStateChanged(state)) {
                eprintln!("failed to send DisplayMessages\n{err}");
            }
        }
    }

    fn listen_text_effect(tx: Sender<DisplayMessages>, effect_rx: Receiver<bool>) {
        loop {
            let msg = match effect_rx.recv() {
                Ok(msg) => msg,
                Err(err) => {
                    eprintln!("failed to recieve message from TextEffect\n{err}");
                    continue;
                }
            };

            if msg {
                if let Err(err) = tx.send(DisplayMessages::AnimationDue) {
                    eprintln!("failed to send DisplayMessage AnimationDue update\n{err}");
                }
            }
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

    fn get_class(&self, state: &PlayerState) -> &str {
        if let Some(playing) = state.playing {
            if playing {
                return "playing";
            } else {
                return "paused";
            }
        }

        "n/a"
    }

    /// Create the final output JSON, in the format that Waybar expects
    fn format_json_output(&self, text: &str, class: &str) -> String {
        format!(
            "{{\"text\": \"{}\", \"tooltip\": \"{}\", \"class\": \"{}\", \"alt\": \"{}\"}}",
            text, "", class, ""
        )
    }

    fn listen_for_updates(&self, rx: Receiver<DisplayMessages>, text_effect: &mut TextEffect) {
        let mut player_state: Option<PlayerState> = None;

        // TODO: only update display if there's a state change or time to run an effect
        loop {
            let msg = match rx.recv() {
                Ok(msg) => msg,
                Err(err) => {
                    eprintln!("failed to recieve message\n{err}");
                    continue;
                }
            };

            match msg {
                DisplayMessages::PlayerStateChanged(state) => {
                    if let Some(player_state) = player_state {
                        if player_state.title != state.title {
                            text_effect.override_last_drawn(state.title.clone());
                        }
                    }
                    player_state = Some(state);
                    self.draw(&player_state, text_effect)
                }
                DisplayMessages::AnimationDue => self.draw(&player_state, text_effect),
            }
        }
    }

    fn draw(&self, player_state: &Option<PlayerState>, text_effect: &mut TextEffect) {
        let player_state = match player_state {
            Some(state) => state,
            None => {
                println!("{}", self.format_json_output("Nothing playing", "stopped"));
                return;
            }
        };

        let icon = match player_state.playing.unwrap_or(false) {
            true => "",
            false => "",
        };

        let artist = &player_state.artist;
        let title = &player_state.title;

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
                text_effect.draw(&Display::sanitize_title(title.clone(), artist))
            )
        };

        println!(
            "{}",
            self.format_json_output(
                &format!("[ {icon} ] {formatted}"),
                self.get_class(player_state)
            )
        )
    }
}

impl Runnable for Display {
    fn run(self: Arc<Self>) -> std::thread::JoinHandle<()> {
        thread::spawn(move || {
            self.init_threads();
        })
    }
}
