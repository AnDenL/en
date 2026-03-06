use std::time::Instant;
use bevy_ecs::prelude::Resource;

#[derive(Resource)]
pub struct Time {
    start_time: Instant,
    last_frame: Instant,
    pub delta_time: f32,
    pub elapsed_time: f32,
}

impl Default for Time {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame: now,
            delta_time: 0.0,
            elapsed_time: 0.0,
        }
    }
}

impl Time {
    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta_time = now.duration_since(self.last_frame).as_secs_f32();
        self.elapsed_time = now.duration_since(self.start_time).as_secs_f32();
        self.last_frame = now;
    }
}