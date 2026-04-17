use unicode_segmentation::UnicodeSegmentation;

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
        let text_graphemes = text.graphemes(true).collect::<Vec<&str>>();
        if text_graphemes.len() <= self.max_width as usize || self.max_width == 0 {
            return text;
        }

        self.active = false;
        format!(
            "{}...",
            // we gotta join here, since we have a Vec<&str>, not a string.
            // and join just looks nicer than .into_iter().collect<String>() but is functionally identical
            text_graphemes.split_at(self.max_width as usize).0.join("")
        )
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn update_active(&mut self) {
        self.active = self.text.len() > self.max_width as usize && self.max_width > 0;
    }

    fn set_text(&mut self, text: String) {
        if self.text != text {
            self.text = text;
            self.update_active();
        }
    }
}
