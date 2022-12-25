use std::time::Instant;

use crate::math;

const SAMPLE_COUNT: usize = 5;

pub struct FrameTimer {
    last_time: Instant,
    delta_frame: f32,
    current_frame: usize,
    fps_samples: [f32; SAMPLE_COUNT],
}

impl FrameTimer {
    pub fn new() -> Self {
        FrameTimer {
            last_time: Instant::now(),
            delta_frame: 0f32,
            current_frame: 0,
            fps_samples: [0f32; SAMPLE_COUNT],
        }
    }

    /// Update the delta time, should be called just after frame update
    pub fn tick(&mut self) {
        let now = Instant::now();
        let delta_time = now - self.last_time;
        self.last_time = now;

        self.delta_frame = delta_time.as_secs_f32();
        self.fps_samples[self.current_frame] = self.delta_frame;
        self.current_frame = (self.current_frame + 1) % SAMPLE_COUNT;
    }

    /// Return current delta time in seconds
    pub fn delta_time(&self) -> f32 {
        self.delta_frame / 1_000_000.0_f32 // µs to s
    }

    /// Return the current framerate
    pub fn fps(&self) -> f32 {
        1f32 / math::mean(&self.fps_samples)
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::new()
    }
}
