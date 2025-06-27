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
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread::{self},
    time::Duration,
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

        let mut fields = HashMap::new();

        fields.insert(
            "title",
            TextEffect::new().with_effect(Box::new(Marquee::new(
                self.args.title_width,
                true,
                self.args.marquee,
            ))),
        );

        fields.insert(
            "artist",
            TextEffect::new().with_effect(Box::new(Marquee::new(
                self.args.artist_width,
                true,
                self.args.marquee,
            ))),
        );

        {
            let tx = tx.clone();
            let effect_speed = self.args.effect_speed as u64;
            thread::spawn(move || {
                Display::text_effect_timer(effect_speed, tx);
            });
        }

        self.listen_for_updates(rx, fields);
    }

    fn text_effect_timer(interval_ms: u64, tx: Sender<DisplayMessages>) {
        loop {
            thread::sleep(Duration::from_millis(interval_ms));
            if let Err(err) = tx.send(DisplayMessages::AnimationDue) {
                warn!("failed to send AnimationDue message: {err}");
            }
        }
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

    fn listen_for_updates(
        &self,
        rx: Receiver<DisplayMessages>,
        mut fields: HashMap<&str, TextEffect>,
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
                            match fields.get_mut("title") {
                                Some(field) => field.override_last_drawn(state.title.clone()),
                                None => error!("failed to get 'title' field"),
                            }
                        }
                        if player_state.artist != state.artist {
                            match fields.get_mut("artist") {
                                Some(field) => field.override_last_drawn(state.artist.clone()),
                                None => error!("failed to get 'artist' field"),
                            }
                        }
                    }
                    player_state = Some(state);
                    self.draw(&player_state, &mut fields)
                }
                DisplayMessages::AnimationDue => {
                    if fields.iter().any(|(_, v)| v.has_active_effects()) {
                        fields.iter_mut().for_each(|(_, v)| {
                            v.should_redraw();
                        });
                        self.draw(&player_state, &mut fields)
                    }
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

    fn draw(&self, player_state: &Option<PlayerState>, fields: &mut HashMap<&str, TextEffect>) {
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
                    format!("{} - ", fields.get_mut("artist").unwrap().draw(artist))
                },
                fields
                    .get_mut("title")
                    .unwrap()
                    .draw(&Display::sanitize_title(title.clone(), artist))
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
