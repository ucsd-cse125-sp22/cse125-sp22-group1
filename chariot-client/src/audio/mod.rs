use std::fs;
use rodio::{Source, Decoder, source::{Buffered}};
use std::time::Duration;
use std::path::PathBuf;
use std::io::BufReader;

pub mod thread;

use thread::AudioThread;
use self::thread::context::AudioCtx;
use self::thread::options::SourceOptions;

// For Playing Track Audio (Music / Ambient / SFX)
pub struct AudioSource {
  tracks: Vec<Buffered<Decoder<BufReader<fs::File>>>>,
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

      let file = fs::File::open(path_buf).unwrap();

      let buf = BufReader::new(file);
      let source = Decoder::new(buf).unwrap().buffered();

      tracks.push(source);
      println!("Loaded Track {}: [{}]", tracks.len(), path_str_dsp);
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
      if thread.is_empty() {
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

    let source = self.tracks.get(track_id).unwrap().clone();
    let mut thread = AudioThread::new(ctx, source, opt);
    thread.set_volume(self.volume);
    thread.set_pitch(self.pitch);

    thread.play();
    self.threads.push(thread);
  }

  // Play audio, & return a thread for self management
  pub fn play_alone(&mut self, track_id: usize, ctx: &AudioCtx, opt: SourceOptions) -> AudioThread {
    let source = self.tracks.get(track_id).unwrap().clone();
    let mut thread = AudioThread::new(ctx, source, opt);
    thread.set_volume(self.volume);
    thread.set_pitch(self.pitch);

    thread.play();
    return thread;
  }

  // Play an Audio such that it will crossfade with all currently playing tracks
  pub fn play_cf(&mut self, track_id: usize, ctx: &AudioCtx, mut opt: SourceOptions, duration: Duration) {
    // Clean Up All Stopped Threads
    self.clean();

    // Enable Fade-In on newest thread
    if self.threads.len() > 0 {
      opt.set_fade_in(duration);
    }

    let source = self.tracks.get(track_id).unwrap().clone();
    let mut thread = AudioThread::new(ctx, source, opt);
    thread.set_volume(self.volume);
    thread.set_pitch(self.pitch);

    self.fade_all_threads(duration);

    thread.play();
    self.threads.push(thread);
  }

  // Do a fade-out on all currently playing threads in this manager
  pub fn fade_all_threads(&mut self, duration: Duration) {
    // Fade Out All Currently Active Threads
    while self.threads.len() > 0 {
      let active_thread = self.threads.pop().unwrap();
      active_thread.fade_out(duration);
    }
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