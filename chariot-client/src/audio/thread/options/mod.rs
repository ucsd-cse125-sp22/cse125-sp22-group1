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
      right_ear: [0.0; 3]
    }
  }

  pub fn set_fade_in(&mut self, duration: Duration) {
    self.fade_in = duration;
  }

  pub fn set_repeat(&mut self, repeat: bool) {
    self.repeat = repeat;
  }

  pub fn set_skip_dur(&mut self, duration: Duration) {
    self.skip_duration = duration;
  }

  pub fn set_take_dur(&mut self, duration: Duration) {
    self.take_duration = duration
  }

  pub fn set_pitch(&mut self, pitch: f32) {
    self.pitch = pitch;
  }

  pub fn set_emitter_pos(&mut self, pos: [f32; 3]) {
    self.emitter_pos = pos;
  }

  pub fn set_left_ear_pos(&mut self, pos: [f32; 3]) {
    self.left_ear = pos;
  }

  pub fn set_right_ear_pos(&mut self, pos: [f32; 3]) {
    self.right_ear = pos;
  }
}