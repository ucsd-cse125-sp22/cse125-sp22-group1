use std::time::Duration;

pub struct SourceOptions {
    pub fade_in: Duration,
    pub repeat: bool,
    pub skip_duration: Duration,
    pub take_duration: Duration,
    pub pitch: f32,
    pub emitter_pos: [f32; 3],
    pub left_ear: [f32; 3],
    pub right_ear: [f32; 3],
}

impl SourceOptions {
    pub fn new() -> Self {
        Self {
            fade_in: Duration::ZERO,
            repeat: false,
            skip_duration: Duration::ZERO,
            take_duration: Duration::from_secs(3600),
            pitch: 1.0,
            emitter_pos: [0.0; 3],
            left_ear: [0.0; 3],
            right_ear: [0.0; 3],
        }
    }

    pub fn set_repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }
}
