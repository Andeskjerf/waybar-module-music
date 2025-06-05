use super::effect::Effect;

pub struct Marquee {
    current_pos: u16,
    max_width: u16,
    /// If the marquee effect should only apply if the length exceeds the max width
    on_exceeded: bool,
}

impl Marquee {
    pub fn new(max_width: u16, on_exceeded: bool) -> Self {
        Self {
            current_pos: 0,
            max_width,
            on_exceeded,
        }
    }
}

impl Effect for Marquee {
    fn apply(&mut self, text: String) -> String {
        if self.on_exceeded && text.len() <= self.max_width as usize {
            return text;
        }

        let mut result = String::new();
        for i in self.current_pos..self.current_pos + text.len() as u16 {
            let index = i % text.len() as u16;
            let c = text
                .chars()
                .nth((i % text.len() as u16) as usize)
                .unwrap_or_else(|| panic!("no char at index {index}"));
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
}
