use std::time::Instant;

pub struct PlayerTimer {
    name: String,
    playing: bool,
    length: u64,
    position: u128,
    rate: u64,
    last_update: Instant,
}

impl PlayerTimer {
    pub fn new(name: String, length: u64) -> Self {
        Self {
            name: name.to_string(),
            playing: false,
            length,
            position: 0,
            rate: 1,
            last_update: Instant::now(),
        }
    }

    pub fn tick(&mut self, increment_ms: u128) {
        // position is in microseconds
        // 1000 == 1 millisecond
        self.position += 1000 * increment_ms;
        self.last_update = Instant::now();
    }

    pub fn position(&self) -> u128 {
        self.position
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn time_ms_since_last_update(&self) -> u128 {
        self.last_update.duration_since(Instant::now()).as_millis()
    }
}
