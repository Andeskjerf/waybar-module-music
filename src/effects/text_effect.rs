use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use log::warn;

use crate::effects::effect::Effect;

pub struct TextEffect {
    last_drawn: String,
    effects: Arc<Mutex<Vec<Box<dyn Effect>>>>,
    update_tick: Arc<Mutex<bool>>,
}

impl TextEffect {
    pub fn new(run_every_ms: u32) -> (Self, Receiver<bool>) {
        let update_tick = Arc::new(Mutex::new(false));
        let effects = Arc::new(Mutex::new(vec![]));
        let (tx, rx) = mpsc::channel();

        {
            let update_tick = Arc::clone(&update_tick);
            let effects = effects.clone();
            thread::spawn(move || {
                TextEffect::check_if_due_for_drawing(run_every_ms, update_tick, effects, tx)
            });
        }

        (
            Self {
                last_drawn: String::new(),
                effects,
                update_tick,
            },
            rx,
        )
    }

    fn check_if_due_for_drawing(
        run_every_ms: u32,
        update_tick: Arc<Mutex<bool>>,
        effects: Arc<Mutex<Vec<Box<dyn Effect>>>>,
        tx: Sender<bool>,
    ) {
        let mut time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        loop {
            // FIXME: temporary until i've implemented a proper event based system for the effects
            thread::sleep(Duration::from_millis(50));
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            // check if we're due for new draw call
            let elapsed = now - time;
            if (elapsed as u32) < run_every_ms {
                continue;
            }

            let should_update = effects
                .lock()
                .unwrap()
                .iter()
                .any(|effect| effect.is_active());

            if !should_update {
                continue;
            }

            // reset our timer if we're due for drawing
            time = now;
            if let Err(err) = tx.send(true) {
                warn!("failed to send TextEffect update tick over channel: {err}");
            }
            *update_tick.lock().unwrap() = true;
        }
    }

    pub fn with_effect(self, effect: Box<dyn Effect>) -> Self {
        self.effects.lock().unwrap().push(effect);
        self
    }

    pub fn override_last_drawn(&mut self, text: String) {
        self.last_drawn = text;
    }

    pub fn draw(&mut self, text: &str) -> String {
        if self.last_drawn.is_empty() {
            self.last_drawn = text.to_string();
        }

        for effect in self.effects.lock().unwrap().iter_mut() {
            effect.set_text(text.to_string());
        }

        let mut lock = self.update_tick.lock().unwrap();
        if !*lock {
            return self.last_drawn.clone();
        }

        *lock = false;
        drop(lock);

        let mut result = text.to_owned();
        for effect in self.effects.lock().unwrap().iter_mut() {
            result = effect.apply(result);
        }
        self.last_drawn = result.clone();
        result
    }
}
