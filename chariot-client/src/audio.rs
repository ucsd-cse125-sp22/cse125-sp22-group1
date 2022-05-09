use std::fs;
use std::sync::{ Arc, Mutex };
use std::thread;

use std::path::PathBuf;
use std::io::BufReader;
use std::time::{SystemTime, Duration};
use rodio::{Decoder, OutputStream, OutputStreamHandle, SpatialSink};
use rodio::source::{Source};

// Audio Context
pub struct AudioCtx {
  _stream: OutputStream,
  stream_handle: OutputStreamHandle
}

impl AudioCtx {
  pub fn new() -> Self {
    // Get a output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    Self {
      _stream,
      stream_handle
    }
  }
}

pub struct SourceOptions {
  fade_in: Duration,
  repeat: bool,
  skip_duration: Duration,
  take_duration: Duration,
  pitch: f32,
  emitter_pos: [f32; 3],
  left_ear: [f32; 3],
  right_ear: [f32; 3],
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
      left_ear: [-0.000001, 0.0, 0.0],
      right_ear: [0.000001, 0.0, 0.0]
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

pub struct AudioThread {
  time_start: SystemTime,
  volume: f32,
  pitch: f32,
  file: fs::File,
  sink: SpatialSink,
  src_opt: SourceOptions,
}

impl AudioThread {
  pub fn new(ctx: &AudioCtx, path_buf: PathBuf, src_opt: SourceOptions) -> Self {
    let sink = SpatialSink::try_new(&ctx.stream_handle, 
      src_opt.emitter_pos, src_opt.left_ear, src_opt.right_ear).unwrap();
    let file = fs::File::open(path_buf.clone()).unwrap();

    Self {
      time_start: SystemTime::now(),
      volume: 1.0,
      pitch: 1.0,
      file: file,
      sink: sink,
      src_opt: src_opt
    }
  }

  pub fn time_alive(&mut self) -> Duration {
    return self.time_start.elapsed().unwrap();
  }

  pub fn play(&mut self) {
    let buf = BufReader::new(self.file.try_clone().unwrap());
    let source = Decoder::new(buf).unwrap();
    
    // Apply Skip Duration
    let skp_src = source.skip_duration(self.src_opt.skip_duration);

    // Apply Pitch Warp
    let pitch_src = skp_src.speed(self.src_opt.pitch);

    // Apply Fade
    let fade_src = pitch_src.fade_in(self.src_opt.fade_in);

    // Apply Take Duration
    let tk_src = fade_src.take_duration(self.src_opt.take_duration);

    // Apply Repeat
    if self.src_opt.repeat {
      let rpt_src = tk_src.repeat_infinite();
      self.sink.append(rpt_src);
    } else {
      self.sink.append(tk_src);
    }

    self.time_start = SystemTime::now();
  }

  // Fade out the sink
  pub fn fade_out(&mut self, duration: Duration) {
    let delta_time = duration.as_millis() / 100;
    
    for n in 1..100 {
      let volume = (self.volume / 100.0) * (100 - n) as f32;
      self.sink.set_volume(volume);
      thread::sleep(Duration::from_millis(delta_time as u64));
      // println!("current volume is {}", volume);
    }

    self.sink.stop();
  }

  // Pause playback
  pub fn pause(&mut self) {
    self.sink.pause();
  }

  // Resume playback
  pub fn resume(&mut self) {
    self.sink.play();
  }

  // Stop all playback
  pub fn stop(&mut self) {
    self.sink.stop();
  }

  pub fn set_volume(&mut self, volume: f32) {
    self.volume = volume;
    self.sink.set_volume(volume);
  }

  pub fn set_pitch(&mut self, pitch: f32) {
    self.pitch = pitch;
    self.sink.set_speed(pitch);
  }
}

// For Playing Track Audio (Music / Ambient / SFX)
pub struct AudioSource {
  tracks: Vec<PathBuf>,
  threads: Vec<AudioThread>,
  volume: f32,
  pitch: f32,
}

impl AudioSource {
  pub fn new(path: &str) -> Self {
    let threads = Vec::new();
    let mut tracks = Vec::new();

    let paths = fs::read_dir(format!("./{}", path)).unwrap();

    for path in paths {
      let path_buf = path.unwrap().path();
      let path_str_dsp = path_buf.display().to_string();

      let file = fs::File::open(path_buf.clone()).unwrap();

      let buf = BufReader::new(file);
      let decode = Decoder::new(buf);

      if decode.is_ok() {
        tracks.push(path_buf);
        println!("Loaded Track {}: [{}]", tracks.len(), path_str_dsp);
      }
    }

    Self {
      threads: threads,
      tracks: tracks,
      volume: 1.0,
      pitch: 1.0
    }
  }

  // Clean Up Sink w/o Audio
  pub fn clean(&mut self) {
    let mut i = 0;
    while i < self.threads.len() {
      let thread = self.threads.get_mut(i).unwrap();
      if thread.sink.empty() {
        self.threads.remove(i);
      } else {
        i += 1;
      }
    }
  }

  // Play Audio
  pub fn play(&mut self, track_id: usize, ctx: &AudioCtx, opt: SourceOptions) {
    // Clean Up All Stopped Threads
    self.clean();

    let path_buf = self.tracks.get(track_id).unwrap();
    let mut thread = AudioThread::new(ctx, path_buf.to_path_buf(), opt);
    thread.set_volume(self.volume);
    thread.set_pitch(self.pitch);

    thread.play();
    self.threads.push(thread);
  }

  // Play an Audio such that it will crossfade with all currently playing tracks
  pub fn play_cf(&mut self, track_id: usize, ctx: &AudioCtx, mut opt: SourceOptions, duration: Duration) {
    // Clean Up All Stopped Threads
    self.clean();

    // Enable Fade-In on newest thread
    if self.threads.len() > 0 {
      opt.set_fade_in(duration);
    }

    let path_buf = self.tracks.get(track_id).unwrap();
    let mut thread = AudioThread::new(ctx, path_buf.to_path_buf(), opt);
    thread.set_volume(self.volume);
    thread.set_pitch(self.pitch);

    self.fade_all_threads(duration);

    thread.play();
    self.threads.push(thread);
  }

  pub fn fade_all_threads(&mut self, duration: Duration) {
    // Fade Out All Currently Active Threads
    while self.threads.len() > 0 {
      let active_thread = self.threads.pop().unwrap();
      self.fade_out_thread(active_thread, duration);
    }
  }

  // Fade Out An Audio Thread
  pub fn fade_out_thread(&mut self, thread: AudioThread, duration: Duration) {
    let x = Arc::new(Mutex::new(thread));
    let alias = x.clone();

    thread::spawn(move || {
      let mut mutref = alias.lock().unwrap(); 
      mutref.fade_out(duration);
    });
  }

  // Pause All Audio Threads
  pub fn pause(&mut self) {
    let mut i = 0;
    while i < self.threads.len() {
      self.threads.get_mut(i).unwrap().pause();
      i += 1;
    }
  }

  // Resume All Audio Threads
  pub fn resume(&mut self) {
    let mut i = 0;
    while i < self.threads.len() {
      self.threads.get_mut(i).unwrap().resume();
      i += 1;
    }
  }

  // Stop All Audio Threads
  pub fn stop(&mut self) {
    let mut i = 0;
    while i < self.threads.len() {
      self.threads.get_mut(i).unwrap().stop();
      i += 1;
    }
  }

  // Set The Volume Of All Playing Threads
  pub fn set_volume(&mut self, vol: f32) {
    self.volume = vol;
    let mut i = 0;
    while i < self.threads.len() {
      self.threads.get_mut(i).unwrap().set_volume(vol);
      i += 1;
    }
  }

  // Set The Pitch Of All Playing Threads
  pub fn set_pitch(&mut self, pitch: f32) {
    self.pitch = pitch;
    let mut i = 0;
    while i < self.threads.len() {
      self.threads.get_mut(i).unwrap().set_pitch(pitch);
      i += 1;
    }
  }
}