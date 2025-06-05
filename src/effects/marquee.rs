use super::effect::Effect;

pub struct Marquee {
    current_pos: u16,
    max_width: u16,
}

impl Marquee {
    pub fn new(max_width: u16) -> Self {
        Self {
            current_pos: 0,
            max_width,
        }
    }
}

impl Effect for Marquee {
    fn apply(&mut self, text: String) -> String {
        let mut result = String::new();
        for i in self.current_pos..self.current_pos + text.len() as u16 {
            let index = i % text.len() as u16;
            let c = text
                .chars()
                .nth((i % text.len() as u16) as usize)
                .unwrap_or_else(|| panic!("no char at index {index}\n{text}"));
            result.push(c);
        }

        self.current_pos += 1;
        self.current_pos %= text.len() as u16;

        result
    }
}
