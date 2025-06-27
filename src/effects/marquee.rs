use std::time::Instant;

use super::effect::Effect;

const PADDING: &str = "     ";

pub struct Marquee {
    text: String,
    current_pos: u16,
    max_width: u16,
    active: bool,
    pause_on_loop_ms: u16,
    instant: Option<Instant>,
}

impl Marquee {
    pub fn new(max_width: u16, pause_on_loop_ms: u16) -> Self {
        Self {
            current_pos: 0,
            max_width,
            active: false,
            text: String::new(),
            pause_on_loop_ms,
            instant: None,
        }
    }
}

impl Effect for Marquee {
    fn apply(&mut self, text: String) -> String {
        if text.len() <= self.max_width as usize || self.max_width == 0 {
            return text;
        }

        let mut text = text.clone();
        text.push_str(PADDING);

        let mut result = String::new();
        for i in self.current_pos..self.current_pos + text.len() as u16 {
            let i = i % text.len() as u16;
            let c = text.chars().nth((i) as usize).unwrap_or(' ');
            result.push(c);
        }

        // FIXME: we want to pause the effect, which we do here
        // somewhat unfinished. we still draw even when there's nothing new to draw while this pause is ongoing
        // since the logic to apply or remove the pause happens in here, we need to process events like normal
        // need to think of something smart so we can avoid this, though it's not a huge deal
        if self.instant.is_some()
            && self.instant.unwrap().elapsed().as_millis() >= self.pause_on_loop_ms as u128
        {
            self.instant = None;
        }

        if self.instant.is_none() {
            self.current_pos += 1;
            self.current_pos %= text.len() as u16;
        }

        if self.instant.is_none() && self.pause_on_loop_ms != 0 && self.current_pos == 0 {
            self.instant = Some(Instant::now());
        }

        if result.len() > self.max_width as usize {
            result.split_at(self.max_width as usize).0.to_owned()
        } else {
            result
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn update_active(&mut self) {
        self.active = self.text.len() > self.max_width as usize && self.max_width > 0
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
        self.update_active();
    }
}
