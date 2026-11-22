const TARGET_FRAME_TIME: f32 = 1.0 / 60.0;
const SPIKE_LIMIT: f32 = 0.25;
const SMOOTHING: f32 = 0.05;
const SLOW_FRAME: f32 = 1.05;
const SHRINK_RATE: f32 = 0.998;
const MIN_SCALE: f32 = 0.1;

pub struct Governor {
    scale: f32,
    frame_time: f32,
}

impl Governor {
    pub fn new() -> Self {
        Self {
            scale: 1.0,
            frame_time: TARGET_FRAME_TIME,
        }
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn observe(&mut self, frame_time: f32) {
        if frame_time > SPIKE_LIMIT {
            return;
        }

        self.frame_time += (frame_time - self.frame_time) * SMOOTHING;

        if self.frame_time > TARGET_FRAME_TIME * SLOW_FRAME {
            self.scale = (self.scale * SHRINK_RATE).max(MIN_SCALE);
        }
    }
}
