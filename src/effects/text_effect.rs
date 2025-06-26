use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::effects::effect::Effect;

pub struct TextEffect {
    last_drawn: String,
    effects: Vec<Box<dyn Effect>>,
    update_tick: Arc<Mutex<bool>>,
}

impl TextEffect {
    pub fn new(run_every_ms: u32) -> (Self, Receiver<bool>) {
        let update_tick = Arc::new(Mutex::new(false));
        let (tx, rx) = mpsc::channel();

        {
            let update_tick = Arc::clone(&update_tick);
            thread::spawn(move || {
                TextEffect::check_if_due_for_drawing(run_every_ms, update_tick, tx)
            });
        }

        (
            Self {
                last_drawn: String::new(),
                effects: vec![],
                update_tick,
            },
            rx,
        )
    }

    fn check_if_due_for_drawing(
        run_every_ms: u32,
        update_tick: Arc<Mutex<bool>>,
        tx: Sender<bool>,
    ) {
        let mut time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        loop {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            // check if we're due for new draw call
            let elapsed = now - time;
            if (elapsed as u32) < run_every_ms {
                continue;
            }

            // reset our timer if we're due for drawing
            time = now;
            if let Err(err) = tx.send(true) {
                eprintln!("failed to send TextEffect update tick over channel\n{err}");
            }
            *update_tick.lock().unwrap() = true;
        }
    }

    pub fn with_effect(mut self, effect: Box<dyn Effect>) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn draw(&mut self, text: &str) -> String {
        let mut lock = self.update_tick.lock().unwrap();
        if !*lock {
            return self.last_drawn.clone();
        }

        *lock = false;
        drop(lock);

        let mut result = text.to_owned();
        for effect in &mut self.effects {
            result = effect.apply(result);
        }
        self.last_drawn = result.clone();
        result
    }
}
