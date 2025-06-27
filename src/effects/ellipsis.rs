use super::effect::Effect;

pub struct Ellipsis {
    text: String,
    max_width: u16,
    active: bool,
}

impl Ellipsis {
    pub fn new(max_width: u16) -> Self {
        Self {
            max_width,
            active: false,
            text: String::new(),
        }
    }
}

impl Effect for Ellipsis {
    fn apply(&mut self, text: String) -> String {
        if text.len() <= self.max_width as usize || self.max_width == 0{
            return text;
        }

        format!("{}...", text.split_at(self.max_width as usize).0)
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn update_active(&mut self) {
        self.active = true
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
        self.update_active();
    }
}
