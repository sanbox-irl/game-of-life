use std::time::Instant;

pub struct Time {
    pub time: Instant,
    pub delta_time: f32,
    pub tick_count: u64,
}

impl Time {
    pub fn new() -> Self {
        Time {
            time: Instant::now(),
            delta_time: 0.0,
            tick_count: 0,
        }
    }

    pub fn game_start(&mut self) {
        self.time = Instant::now();
    }

    pub fn end_frame(&mut self) {
        let new_time = Instant::now();
        let difference = new_time.duration_since(self.time);
        let delta_time = difference.as_secs() as f64 + (f64::from(difference.subsec_nanos()) / 1.0e9);

        self.delta_time = delta_time as f32;
        self.time = new_time;
        self.tick_count += 1;
    }
}
