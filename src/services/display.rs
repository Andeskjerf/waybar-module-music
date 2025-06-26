use bincode::config;
use log::{error, info, warn};

use crate::{
    effects::{marquee::Marquee, text_effect::TextEffect},
    event_bus::{EventBusHandle, EventType},
    models::{args::Args, player_state::PlayerState},
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
    args: Arc<Args>,
    event_bus: EventBusHandle,
}

impl Display {
    pub fn new(args: Arc<Args>, event_bus: EventBusHandle) -> Self {
        Self { args, event_bus }
    }

    fn init_worker(self: Arc<Self>) {
        println!("{}", self.format_json_output("No activity", "stopped"));

        let (tx, rx) = mpsc::channel();

        if let Some(rx) = self.event_bus.subscribe(EventType::PlayerStateChanged) {
            let tx = tx.clone();
            thread::spawn(move || {
                Display::listen_player_state(rx, tx);
            });
        } else {
            error!("failed to subscribe to PlayerStateChanged listener");
        }

        // TODO: figure out something smart for enabling the text effect on the artist too
        let (title_effect, effect_rx) = TextEffect::new(self.args.effect_speed);
        let title_effect =
            title_effect.with_effect(Box::new(Marquee::new(self.args.title_width, true)));

        {
            let tx = tx.clone();
            thread::spawn(move || {
                Display::listen_text_effect(tx, effect_rx);
            });
        }

        let (artist_effect, effect_rx) = TextEffect::new(self.args.effect_speed);
        let artist_effect =
            artist_effect.with_effect(Box::new(Marquee::new(self.args.artist_width, true)));

        {
            let tx = tx.clone();
            thread::spawn(move || {
                Display::listen_text_effect(tx, effect_rx);
            });
        }

        self.listen_for_updates(rx, artist_effect, title_effect);
    }

    fn listen_player_state(rx: Receiver<Vec<u8>>, tx: Sender<DisplayMessages>) {
        loop {
            let msg = rx.recv();
            let (state, _): (PlayerState, usize) = match msg {
                Ok(encoded) => {
                    bincode::decode_from_slice(&encoded[..], config::standard()).unwrap()
                }
                Err(err) => {
                    warn!("failed to decode message in Display: {err}");
                    continue;
                }
            };

            if let Err(err) = tx.send(DisplayMessages::PlayerStateChanged(state)) {
                warn!("failed to send DisplayMessages: {err}");
            }
        }
    }

    fn listen_text_effect(tx: Sender<DisplayMessages>, effect_rx: Receiver<bool>) {
        loop {
            let msg = match effect_rx.recv() {
                Ok(msg) => msg,
                Err(err) => {
                    warn!("failed to recieve message from TextEffect: {err}");
                    continue;
                }
            };

            if msg {
                if let Err(err) = tx.send(DisplayMessages::AnimationDue) {
                    warn!("failed to send DisplayMessage AnimationDue update: {err}");
                }
            }
        }
    }

    fn listen_for_updates(
        &self,
        rx: Receiver<DisplayMessages>,
        mut artist_effect: TextEffect,
        mut title_effect: TextEffect,
    ) {
        let mut player_state: Option<PlayerState> = None;

        loop {
            let msg = match rx.recv() {
                Ok(msg) => msg,
                Err(err) => {
                    warn!("failed to recieve message: {err}");
                    continue;
                }
            };

            match msg {
                DisplayMessages::PlayerStateChanged(state) => {
                    if let Some(player_state) = player_state {
                        if player_state.title != state.title {
                            title_effect.override_last_drawn(state.title.clone());
                        }
                    }
                    player_state = Some(state);
                    self.draw(&player_state, &mut artist_effect, &mut title_effect)
                }
                DisplayMessages::AnimationDue => {
                    self.draw(&player_state, &mut artist_effect, &mut title_effect)
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

        "stopped"
    }

    /// Create the final output JSON, in the format that Waybar expects
    fn format_json_output(&self, text: &str, class: &str) -> String {
        format!(
            "{{\"text\": \"{}\", \"tooltip\": \"{}\", \"class\": \"{}\", \"alt\": \"{}\"}}",
            text, "", class, ""
        )
    }

    fn draw(
        &self,
        player_state: &Option<PlayerState>,
        artist_effect: &mut TextEffect,
        title_effect: &mut TextEffect,
    ) {
        let player_state = match player_state {
            Some(state) => state,
            None => {
                println!("{}", self.format_json_output("Nothing playing", "stopped"));
                return;
            }
        };

        let icon = match player_state.playing.unwrap_or(false) {
            true => &self.args.pause_icon,
            false => &self.args.play_icon,
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
                    format!("{} - ", artist_effect.draw(artist))
                },
                title_effect.draw(&Display::sanitize_title(title.clone(), artist))
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
            info!("starting Display worker");
            self.init_worker();
        })
    }
}
