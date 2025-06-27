use super::effect::Effect;

const PADDING: &str = "     ";

pub struct Marquee {
    text: String,
    current_pos: u16,
    max_width: u16,
    active: bool,
}

impl Marquee {
    pub fn new(max_width: u16) -> Self {
        Self {
            current_pos: 0,
            max_width,
            active: false,
            text: String::new(),
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

        self.current_pos += 1;
        self.current_pos %= text.len() as u16;

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
