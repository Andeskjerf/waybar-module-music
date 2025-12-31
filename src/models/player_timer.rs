use std::time::Instant;

#[derive(Debug)]
pub struct PlayerTimer {
    playing: bool,
    position: u128,
    rate: f64,
    last_update: Instant,
}

impl PlayerTimer {
    pub fn new() -> Self {
        Self {
            playing: false,
            position: 0,
            rate: 1.0,
            last_update: Instant::now(),
        }
    }

    pub fn tick(&mut self, increment_ms: u128) {
        // position is in microseconds
        // 1000 == 1 millisecond
        self.position += 1000 * (increment_ms as f64 * self.rate()) as u128;
        self.last_update = Instant::now();
    }

    pub fn set_position(&mut self, position: u128) {
        self.position = position;
    }

    pub fn set_rate(&mut self, rate: f64) {
        self.rate = rate;
    }

    pub fn rate(&self) -> f64 {
        if self.rate == 0.0 {
            1.0
        } else {
            self.rate
        }
    }

    pub fn position(&self) -> u128 {
        self.position
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.playing = playing;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn time_ms_since_last_update(&self) -> u128 {
        self.last_update.duration_since(Instant::now()).as_millis()
    }
}
