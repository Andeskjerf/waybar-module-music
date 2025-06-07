use std::time::{SystemTime, UNIX_EPOCH};

use crate::effects::effect::Effect;

pub struct TextEffect {
    last_drawn: String,
    effects: Vec<Box<dyn Effect>>,
    time: u128,
    run_every_ms: u32,
}

impl TextEffect {
    pub fn new(run_every_ms: u32) -> Self {
        Self {
            last_drawn: String::new(),
            effects: vec![],
            time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            run_every_ms,
        }
    }

    pub fn with_effect(mut self, effect: Box<dyn Effect>) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn draw(&mut self, text: &str) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // check if we're due for new draw call
        let elapsed = now - self.time;
        if (elapsed as u32) < self.run_every_ms {
            return self.last_drawn.clone();
        }
        // reset our timer if we're due for drawing
        self.time = now;

        let mut result = text.to_owned();
        for effect in &mut self.effects {
            result = effect.apply(result);
        }
        self.last_drawn = result.clone();
        result
    }
}
