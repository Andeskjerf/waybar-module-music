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

        let mut text = text.clone();
        text.push_str(&" ".repeat(5));

        let mut result = String::new();
        for i in self.current_pos..self.current_pos + text.len() as u16 {
            let i = i % text.len() as u16;
            let c = text
                .chars()
                .nth((i) as usize)
                // .unwrap_or_else(|| panic!("no char at index {i}. len == {}\n{text}", text.len()));
                .unwrap_or(' ');
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
