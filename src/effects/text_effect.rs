use std::sync::{Arc, Mutex};

use crate::effects::effect::Effect;

#[derive(Default)]
pub struct TextEffect {
    last_drawn: String,
    effects: Arc<Mutex<Vec<Box<dyn Effect>>>>,
    update_tick: Arc<Mutex<bool>>,
}

impl TextEffect {
    pub fn new() -> Self {
        let update_tick = Arc::new(Mutex::new(false));
        let effects = Arc::new(Mutex::new(vec![]));

        Self {
            last_drawn: String::new(),
            effects,
            update_tick,
        }
    }

    pub fn set_effect_text(&mut self, text: String) {
        self.effects.lock().unwrap().iter_mut().for_each(|effect| {
            effect.set_text(text.clone());
        });
    }

    pub fn has_active_effects(&self) -> bool {
        self.effects
            .lock()
            .unwrap()
            .iter()
            .any(|elem| elem.is_active())
    }

    pub fn should_redraw(&mut self) {
        *self.update_tick.lock().unwrap() = true;
    }

    pub fn with_effect(self, effect: Box<dyn Effect>) -> Self {
        self.effects.lock().unwrap().push(effect);
        self
    }

    pub fn override_last_drawn(&mut self, text: String) {
        self.last_drawn = text;
    }

    pub fn draw(&mut self, text: &str) -> String {
        let text = self.sanitize_output(text);

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
