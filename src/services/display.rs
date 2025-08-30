use bincode::config;
use log::{debug, error, info, warn};

use crate::{
    effects::{ellipsis::Ellipsis, marquee::Marquee, text_effect::TextEffect},
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

#[derive(Debug)]
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

    fn escape_pango(&self, text_to_escape: &str) -> String {
        text_to_escape
            .chars()
            .map(|x| match x {
                // find and replace all special characters with escaped pango sequences
                '&' => "&amp;".to_string(),
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '\'' => "&#39;".to_string(),
                '"' => "&quot;".to_string(),
                _ => x.to_string(),
            })
            .collect()
    }

    fn init_worker(self: Arc<Self>) {
        println!("{}", self.format_json_output("No activity", "stopped"));

        let (tx, rx) = mpsc::channel();
        let (effect_tx, effect_rx) = mpsc::channel();

        if let Some(rx) = self.event_bus.subscribe(EventType::PlayerStateChanged) {
            let tx = tx.clone();
            thread::spawn(move || {
                Display::listen_player_state(rx, tx);
            });
        } else {
            error!("failed to subscribe to PlayerStateChanged listener");
        }

        {
            let tx = tx.clone();
            let effect_speed = self.args.effect_speed as u64;
            thread::spawn(move || {
                Display::text_effect_timer(effect_speed, effect_rx, tx);
            });
        }

        self.listen_for_updates(rx, effect_tx, self.init_fields());
    }

    fn init_fields(&self) -> HashMap<&str, TextEffect> {
        let mut fields = HashMap::new();

        // FIXME: I'm sure this could be done better
        if self.args.marquee {
            fields.insert(
                "title",
                TextEffect::new().with_effect(Box::new(Marquee::new(
                    self.args.title_width,
                    self.args.delay_marquee as u16,
                ))),
            );

            fields.insert(
                "artist",
                TextEffect::new().with_effect(Box::new(Marquee::new(
                    self.args.artist_width,
                    self.args.delay_marquee as u16,
                ))),
            );
        } else if self.args.ellipsis {
            fields.insert(
                "title",
                TextEffect::new().with_effect(Box::new(Ellipsis::new(self.args.title_width))),
            );

            fields.insert(
                "artist",
                TextEffect::new().with_effect(Box::new(Ellipsis::new(self.args.artist_width))),
            );
        } else {
            fields.insert("title", TextEffect::new());
            fields.insert("artist", TextEffect::new());
        }

        fields.insert("album", TextEffect::new());
        fields.insert("player", TextEffect::new());

        fields
    }

    fn text_effect_timer(interval_ms: u64, effect_rx: Receiver<bool>, tx: Sender<DisplayMessages>) {
        let mut active_effects = false;
        loop {
            if active_effects {
                thread::sleep(Duration::from_millis(interval_ms));
                if let Err(err) = tx.send(DisplayMessages::AnimationDue) {
                    warn!("failed to send AnimationDue message: {err}");
                }
                active_effects = match effect_rx.try_recv() {
                    Ok(msg) => msg,
                    Err(_) => active_effects,
                };
            } else {
                debug!("waiting for effect trigger to continue effect timer");
                active_effects = match effect_rx.recv() {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!("failed to receieve effect message: {err}");
                        false
                    }
                };
                debug!("got effect trigger message: {active_effects}");
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

    fn set_text_effect_field(
        fields: &mut HashMap<&str, TextEffect>,
        old_value: &str,
        new_value: &str,
        field: &str,
    ) {
        if old_value != new_value {
            match fields.get_mut(field) {
                Some(field) => {
                    field.set_effect_text(new_value.to_string());
                    field.override_last_drawn(new_value.to_string());
                }
                None => error!("failed to get '{field}' field"),
            }
        }
    }

    fn should_effects_be_redrawn(&self, fields: &HashMap<&str, TextEffect>) -> bool {
        fields.iter().any(|(_, v)| v.has_active_effects())
    }

    fn listen_for_updates(
        &self,
        rx: Receiver<DisplayMessages>,
        effect_tx: Sender<bool>,
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

            debug!("msg receieved: {:?}", msg);

            match msg {
                DisplayMessages::PlayerStateChanged(state) => {
                    if let Some(player_state) = player_state {
                        Display::set_text_effect_field(
                            &mut fields,
                            &player_state.title,
                            &state.title,
                            "title",
                        );
                        Display::set_text_effect_field(
                            &mut fields,
                            &player_state.artist,
                            &state.artist,
                            "artist",
                        );
                        Display::set_text_effect_field(
                            &mut fields,
                            &player_state.album,
                            &state.album,
                            "album",
                        );
                        Display::set_text_effect_field(
                            &mut fields,
                            &player_state.player_name,
                            &state.player_name,
                            "player",
                        );
                    }
                    player_state = Some(state);
                    fields.iter_mut().for_each(|(_, v)| {
                        v.should_redraw();
                    });
                    self.draw(&player_state, &mut fields);
                    if let Err(err) = effect_tx.send(self.should_effects_be_redrawn(&fields)) {
                        error!("failed to notify effects thread: {err}");
                    }
                }
                DisplayMessages::AnimationDue => {
                    if self.should_effects_be_redrawn(&fields) {
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
        let text = self.escape_pango(text);
        let class = self.escape_pango(class);
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
        let replacements: HashMap<&str, String> = [
            (
                "icon",
                match player_state.playing.unwrap_or(false) {
                    true => self.args.pause_icon.clone(),
                    false => self.args.play_icon.clone(),
                },
            ),
            (
                "title",
                fields.get_mut("title").unwrap().draw(&player_state.title),
            ),
            (
                "artist",
                fields.get_mut("artist").unwrap().draw(&player_state.artist),
            ),
            (
                "album",
                fields.get_mut("album").unwrap().draw(&player_state.album),
            ),
            (
                "player",
                fields
                    .get_mut("player")
                    .unwrap()
                    .draw(&player_state.player_name),
            ),
        ]
        .into_iter()
        .collect();

        replacements
            .iter()
            .fold(self.args.format.clone(), |acc, (key, value)| {
                acc.replace(&format!("%{key}%"), value)
            })
    }

    fn draw(&self, player_state: &Option<PlayerState>, fields: &mut HashMap<&str, TextEffect>) {
        let player_state = match player_state {
            Some(state) => state,
            None => {
                println!("{}", self.format_json_output("Nothing playing", "stopped"));
                return;
            }
        };

        let output = self.format_json_output(
            &self.populate_using_placeholders(player_state, fields),
            self.get_class(player_state),
        );

        // debug!("{output}");
        println!("{output}")
    }
}

impl Runnable for Display {
    fn run(self: Arc<Self>) -> std::thread::JoinHandle<()> {
        thread::spawn(move || {
            info!("starting Display thread");
            self.init_worker();
            info!("Display thread is stopping");
        })
    }
}
