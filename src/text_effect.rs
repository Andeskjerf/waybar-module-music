use std::time::{SystemTime, UNIX_EPOCH};

use crate::effects::effect::Effect;

pub struct TextEffect {
    content: String,
    last_drawn: String,
    effects: Vec<Box<dyn Effect>>,
    time: u128,
    run_every_ms: u32,
}

impl TextEffect {
    pub fn new(run_every_ms: u32) -> Self {
        Self {
            content: String::new(),
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

    pub fn set_content(&mut self, content: &str) {
        self.content = String::from(content);
        if self.last_drawn.is_empty() {
            self.last_drawn = self.content.clone();
        }
    }

    pub fn draw(&mut self, now: u128) -> String {
        // check if we're due for new draw call
        let elapsed = now - self.time;
        if (elapsed as u32) < self.run_every_ms {
            return self.last_drawn.clone();
        }
        // reset our timer if we're due for drawing
        self.time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let mut text_with_effect = self.content.clone();
        for effect in &mut self.effects {
            text_with_effect = effect.apply(text_with_effect);
        }
        self.last_drawn = text_with_effect.clone();
        text_with_effect
    }
}
