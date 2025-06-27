use bincode::config;
use log::{error, info, warn};

use crate::{
    effects::{marquee::Marquee, text_effect::TextEffect},
    event_bus::{EventBusHandle, EventType},
    models::{args::Args, player_state::PlayerState},
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

        fields.insert("album", TextEffect::new());
        fields.insert("player", TextEffect::new());

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

    fn populate_using_placeholders(
        &self,
        player_state: &PlayerState,
        fields: &mut HashMap<&str, TextEffect>,
    ) -> String {
        let icon = match player_state.playing.unwrap_or(false) {
            true => &self.args.pause_icon,
            false => &self.args.play_icon,
        };

        // FIXME: the fields shouldn't be missing, but I should still try to avoid unwrapping
        let artist = fields.get_mut("artist").unwrap().draw(&player_state.artist);
        let title = fields.get_mut("title").unwrap().draw(&player_state.title);
        let album = fields.get_mut("album").unwrap().draw(&player_state.album);
        let player = fields
            .get_mut("player")
            .unwrap()
            .draw(player_state.player_id.as_ref().unwrap());

        let mut result = String::new();
        let mut placeholder = String::new();
        let mut add_next = false;

        for c in self.args.format.chars() {
            if add_next {
                placeholder.push(c);
                add_next = false;
            } else if c != '%' && !placeholder.is_empty() {
                placeholder.push(c);
            } else if c != '%' {
                result.push(c);
            } else if c == '%' && !placeholder.is_empty() {
                match placeholder.to_lowercase().as_str() {
                    "icon" => result.push_str(icon),
                    "title" => result.push_str(&title),
                    "artist" => result.push_str(&artist),
                    "album" => result.push_str(&album),
                    "player" => result.push_str(&player),
                    _ => error!("failed to parse placeholder: {placeholder}"),
                }
                placeholder.clear();
            } else if c == '%' && placeholder.is_empty() {
                add_next = true;
            }
        }

        result
    }

    fn draw(&self, player_state: &Option<PlayerState>, fields: &mut HashMap<&str, TextEffect>) {
        let player_state = match player_state {
            Some(state) => state,
            None => {
                println!("{}", self.format_json_output("Nothing playing", "stopped"));
                return;
            }
        };

        println!(
            "{}",
            self.format_json_output(
                &self.populate_using_placeholders(player_state, fields),
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
