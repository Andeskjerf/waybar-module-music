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
    fn apply(&mut self, text: &str) -> String {
        let mut result = String::new();
        for i in self.current_pos..self.current_pos + text.len() as u16 {
            let c = text.chars().nth(i as usize).unwrap();
            result.push(c);
        }

        self.current_pos += 1;
        self.current_pos %= self.max_width;

        String::new()
    }
}
