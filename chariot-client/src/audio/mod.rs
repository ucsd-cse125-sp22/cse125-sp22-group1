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

    let paths = fs::read_dir(format!("./{}", path)).unwrap_or_else(|err| {
      panic!("Problem reading the directory: {}", err);
    });

    for path in paths {
      let path_buf = match path {
        Ok(p) => p.path(),
        Err(err) => {
          println!("Problem obtaining the path: {}", err);
          continue;
        }
      };

      let path_str_dsp = path_buf.display().to_string();

      let file = fs::File::open(path_buf);
      let file = match file {
        Ok(f) => f,
        Err(err) => {
          println!("Problem opening the file: {}", err);
          continue;
        }
      };

      let buf = BufReader::new(file);
      let source = Decoder::new(buf);
      let source = match source {
        Ok(s) => s.buffered(),
        Err(err) => {
          println!("Problem decoding the file: {}", err);
          continue;
        }
      };


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
      let thread = self.threads.get_mut(i);
      match thread {
        Some(t) => {
          if t.is_empty() {
            self.threads.remove(i);
          } else {
            i += 1;
          }
        },
        None => {
          println!("Problem closing the thread: {}", i);
          i += 1;
        }
      };
    }
  }

  // Play Audio
  pub fn play(&mut self, track_id: usize, ctx: &AudioCtx, opt: SourceOptions) {
    // Clean Up All Stopped Threads
    self.clean();

    let source = self.tracks.get(track_id);
    let source = match source {
      Some(s) => s.clone(),
      None => {
        println!("Problem loading the source: {}", track_id);
        return;
      }
    };

    let mut thread = AudioThread::new(ctx, source, opt);
    thread.set_volume(self.volume);
    thread.set_pitch(self.pitch);

    thread.play();
    self.threads.push(thread);
  }

  // Play audio, & return a thread for self management
  pub fn play_alone(&mut self, track_id: usize, ctx: &AudioCtx, opt: SourceOptions) -> Option<AudioThread> {
    let source = self.tracks.get(track_id);
    let source = match source {
      Some(s) => s.clone(),
      None => {
        println!("Problem loading the source: {}", track_id);
        return None;
      }
    };

    let mut thread = AudioThread::new(ctx, source, opt);
    thread.set_volume(self.volume);
    thread.set_pitch(self.pitch);

    thread.play();
    return Some(thread);
  }

  // Play an Audio such that it will crossfade with all currently playing tracks
  pub fn play_cf(&mut self, track_id: usize, ctx: &AudioCtx, mut opt: SourceOptions, duration: Duration) {
    // Clean Up All Stopped Threads
    self.clean();

    // Enable Fade-In on newest thread
    if self.threads.len() > 0 {
      opt.set_fade_in(duration);
    }

    let source = self.tracks.get(track_id);
    let source = match source {
      Some(s) => s.clone(),
      None => {
        println!("Problem loading the source: {}", track_id);
        return;
      }
    };

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
      let active_thread = self.threads.pop();
      match active_thread {
        Some(t) => t.fade_out(duration),
        None => {
          println!("There was an issue fading out the thread");
        }
      }
    }
  }

  // Pause All Audio Threads
  pub fn pause(&mut self) {
    let mut i = 0;
    while i < self.threads.len() {
      let thread = self.threads.get_mut(i);
      match thread {
        Some(t) => t.pause(),
        None => {
          println!("There was an issue pausing the thread: {}", i);
        }
      };
      i += 1;
    }
  }

  // Resume All Audio Threads
  pub fn resume(&mut self) {
    let mut i = 0;
    while i < self.threads.len() {
      let thread = self.threads.get_mut(i);
      match thread {
        Some(t) => t.resume(),
        None => {
          println!("There was an issue resuming the thread: {}", i);
        }
      };
      i += 1;
    }
  }

  // Stop All Audio Threads
  pub fn stop(&mut self) {
    let mut i = 0;
    while i < self.threads.len() {
      let thread = self.threads.get_mut(i);
      match thread {
        Some(t) => t.stop(),
        None => {
          println!("There was an issue stopping the thread: {}", i);
        }
      };
      i += 1;
    }
  }

  // Set The Volume Of All Playing Threads
  pub fn set_volume(&mut self, vol: f32) {
    self.volume = vol;
    let mut i = 0;
    while i < self.threads.len() {
      let thread = self.threads.get_mut(i);
      match thread {
        Some(t) => t.set_volume(vol),
        None => {
          println!("There was an issue with setting the volume of thread: {}", i);
        }
      };
      i += 1;
    }
  }

  // Set The Pitch Of All Playing Threads
  pub fn set_pitch(&mut self, pitch: f32) {
    self.pitch = pitch;
    let mut i = 0;
    while i < self.threads.len() {
      let thread = self.threads.get_mut(i);
      match thread {
        Some(t) => t.set_pitch(pitch),
        None => {
          println!("There was an issue with setting the pitch of thread: {}", i);
        }
      };
      i += 1;
    }
  }
}