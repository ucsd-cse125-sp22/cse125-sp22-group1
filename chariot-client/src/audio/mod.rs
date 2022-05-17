use std::fs;
use rodio::{Source, Decoder, source::{Buffered}};
use std::time::Duration;
use std::path::PathBuf;
use std::io::BufReader;
use std::collections::HashMap;

pub mod thread;

use thread::AudioThread;
use self::thread::context::AudioCtx;
use self::thread::options::SourceOptions;

// Buffered Audio Source
type AudioBuffer = Buffered<Decoder<BufReader<fs::File>>>;

// For Playing Track Audio (Music / Ambient / SFX)
pub struct AudioSource {
  thread_ct: u64,
  tracks: HashMap<String, AudioBuffer>,
  threads: HashMap<u64, AudioThread>,
  volume: f32,
  pitch: f32,
}

impl AudioSource {
  pub fn new(path: &str) -> Self {
    let mut tracks = HashMap::new();
    let threads = HashMap::new();

    let paths = fs::read_dir(format!("./{}", path)).unwrap_or_else(|err| {
      panic!("Problem reading the directory: {}", err);
    });

    // obtains buffers for each audio file in the provided paths
    for path in paths {
      let path_buf = match path {
        Ok(p) => p.path(),
        Err(err) => {
          println!("Problem obtaining the path: {}", err);
          continue;
        }
      };

      let file_name = path_buf.file_name();
      let file_path = path_buf.display().to_string();

      // Identify the file name of the loaded file
      let file_name = match file_name {
        Some(n) => match n.to_str() {
          Some(s) => s.to_owned(),
          None => {
            println!("There was a problem with creating a string from a filename in: {}", file_path);
            continue; 
          }
        },
        None => {
          println!("There was a problem with identifying the filename in: {}", file_path);
          continue;
        }
      };

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


      tracks.insert(file_name, source);
      println!("Loaded Track [{}] from: {}", tracks.len(), file_path);
    }

    Self {
      thread_ct: 0,
      threads: threads,
      tracks: tracks,
      volume: 1.0,
      pitch: 1.0
    }
  }

  // Clean Up Sink w/o Audio
  pub fn clean(&mut self) {
    self.threads.retain(|&_id, thread| !thread.is_empty());
    println!("Threads alive: {}", self.threads.len());
  }

  // Finds a track in the tracklist (or returns None if it doesn't exist)
  pub fn getTrack(&mut self, track_name: &str) -> Option<AudioBuffer> {
    if !self.tracks.contains_key(track_name) {
      println!("Error finding the track named: {}", track_name);
      return None;
    }

    let source = self.tracks.get(track_name);
    let source = match source {
      Some(s) => s.clone(),
      None => {
        println!("Problem loading the source: {}", track_name);
        return None;
      }
    };

    return Some(source);
  }

  // Spawns a new thread with preapplied volume & pitch for use
  pub fn spawnThread(&mut self, ctx: &AudioCtx, source: AudioBuffer, opt: SourceOptions) -> AudioThread {
    let mut thread = AudioThread::new(self.thread_ct, ctx, source, opt);
    thread.set_volume(self.volume);
    thread.set_pitch(self.pitch);

    // Increment our track counter
    self.thread_ct = self.thread_ct + 1;

    return thread;
  }

  // Play Audio
  pub fn play(&mut self, track_name: &str, ctx: &AudioCtx, opt: SourceOptions) {
    // Clean Up All Stopped Threads
    self.clean();

    let source = match self.getTrack(track_name) {
      Some(s) => s,
      None => {
        return;
      }
    };

    let mut thread = self.spawnThread(ctx, source, opt);
    thread.play();
    self.threads.insert(thread.getId(), thread);
  }

  // Play an Audio and crossfade out all currently playing tracks
  pub fn play_cf(&mut self, track_name: &str, ctx: &AudioCtx, mut opt: SourceOptions, duration: Duration) {
    // Clean Up All Stopped Threads
    self.clean();

    // Enable Fade-In on newest thread
    if self.threads.len() > 0 {
      opt.set_fade_in(duration);
    }

    let source = match self.getTrack(track_name) {
      Some(s) => s,
      None => {
        return;
      }
    };

    let mut thread = self.spawnThread(ctx, source, opt);
    self.fade_all_threads(duration);

    thread.play();
    self.threads.insert(thread.getId(), thread);
  }

  // Do a fade-out on all currently playing threads in this manager
  pub fn fade_all_threads(&mut self, duration: Duration) {
    // Fade Out All Currently Active Threads
    self.threads.drain().for_each(|(_id, thread)| {
      thread.fade_out(duration);
    });
  }

  // Pause All Audio Threads
  pub fn pause(&mut self) {
    self.threads.iter_mut().for_each(|(_id, thread)| {
      thread.pause();
    });
  }

  // Resume All Audio Threads
  pub fn resume(&mut self) {
    self.threads.iter_mut().for_each(|(_id, thread)| {
      thread.resume();
    });
  }

  // Stop All Audio Threads
  pub fn stop(&mut self) {
    self.threads.iter_mut().for_each(|(_id, thread)| {
      thread.stop();
    });
  }

  // Set The Volume Of All Playing Threads
  pub fn set_volume(&mut self, vol: f32) {
    self.volume = vol;

    self.threads.iter_mut().for_each(|(_id, thread)| {
      thread.set_volume(vol);
    });
  }

  // Set The Pitch Of All Playing Threads
  pub fn set_pitch(&mut self, pitch: f32) {
    self.pitch = pitch;

    self.threads.iter_mut().for_each(|(_id, thread)| {
      thread.set_pitch(pitch);
    });
  }
}