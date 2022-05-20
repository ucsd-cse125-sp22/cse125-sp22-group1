use std::fs::File;
use std::sync::{ Arc, Mutex };
use std::thread;
use rodio::{Source, Decoder, Sink, SpatialSink, source::{Buffered}};

use std::path::PathBuf;
use std::io::BufReader;

use std::time::{SystemTime, Duration};

// Buffered Audio Source
pub type AudioBuffer = Buffered<Decoder<BufReader<File>>>;

pub mod context;
pub mod options;

use context::AudioCtx;
use options::SourceOptions;

enum AudioSinkType {
  Spatial(SpatialSink),
  Standard(Sink)
}

pub struct AudioThread {
  pub thread_id: u64,
  time_start: SystemTime,
  volume: f32,
  pitch: f32,
  source: AudioBuffer,
  sink: AudioSinkType,
  src_opt: SourceOptions,
}

impl AudioThread {
  pub fn new(thread_id: u64, ctx: &AudioCtx, source: Buffered<Decoder<BufReader<File>>>, 
    src_opt: SourceOptions) -> Self {
    if src_opt.emitter_pos != [0.0; 3] || 
      src_opt.left_ear != [0.0; 3] || 
      src_opt.right_ear != [0.0; 3] {
      // Spatial Sink
      let spatial_sink = SpatialSink::try_new(&ctx.stream_handle, 
        src_opt.emitter_pos, src_opt.left_ear, src_opt.right_ear);
      let sink = match spatial_sink {
        Ok(s) => AudioSinkType::Spatial(s),
        Err(err) => {
          println!("There was an error in creating the spatial sink instance: {}", err);
          AudioSinkType::Standard(Sink::new_idle().0)
        }
      };

      return Self {
        thread_id: thread_id,
        time_start: SystemTime::now(),
        volume: 1.0,
        pitch: 1.0,
        source: source,
        sink: sink,
        src_opt: src_opt
      }
    } else {
      // Standard Sink
      let sink = Sink::try_new(&ctx.stream_handle);
      let sink = match sink {
        Ok(s) => AudioSinkType::Standard(s),
        Err(err) => {
          println!("There was an error in creating the sink instance: {}", err);
          AudioSinkType::Standard(Sink::new_idle().0)
        }
      };

      return Self {
        thread_id: thread_id,
        time_start: SystemTime::now(),
        volume: 1.0,
        pitch: 1.0,
        source: source,
        sink: sink,
        src_opt: src_opt
      }
    }
  }

  pub fn time_alive(&mut self) -> Option<Duration> {
    let time_elapsed = self.time_start.elapsed();
    match time_elapsed {
      Ok(t) => return Some(t),
      Err(err) => {
        println!("There was an error at retrieving thread lifetime, {}", err);
        return None;
      }
    }
  }

  pub fn play(&mut self) {
    let source = self.source.clone();

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
      match &self.sink {
        AudioSinkType::Spatial(sink) => sink.append(rpt_src),
        AudioSinkType::Standard(sink) => sink.append(rpt_src),
      }
    } else {
      match &self.sink {
        AudioSinkType::Spatial(sink) => sink.append(tk_src),
        AudioSinkType::Standard(sink) => sink.append(tk_src),
      }
    }

    self.time_start = SystemTime::now();
  }

  // Fade out the sink
  pub fn fade_out(self, duration: Duration) {
    // 1 step every 10 ms
    let steps = duration.as_millis() / 10;
    let delta_time = duration.as_millis() / steps;
    
    let x = Arc::new(Mutex::new(self));
    let alias = x.clone();

    // Spawn a new thread to automate the fade out
    thread::spawn(move || {
      let mutref = alias.lock();
      let mutref = match mutref {
        Ok(m) => m,
        Err(err) => {
          println!("There was an error while creating the mutref: {}", err);
          return;
        }
      };

      for n in 1..steps {
        let volume = (mutref.volume / steps as f32) * (steps - n) as f32;

        match &mutref.sink {
          AudioSinkType::Spatial(sink) => sink.set_volume(volume),
          AudioSinkType::Standard(sink) => sink.set_volume(volume),
        }
        
        thread::sleep(Duration::from_millis(delta_time as u64));
      }

      match &mutref.sink {
        AudioSinkType::Spatial(sink) => sink.stop(),
        AudioSinkType::Standard(sink) => sink.stop(),
      }
    });
  }

  // Pause playback
  pub fn pause(&mut self) {
    match &self.sink {
      AudioSinkType::Spatial(sink) => sink.pause(),
      AudioSinkType::Standard(sink) => sink.pause(),
    }
  }

  // Resume playback
  pub fn resume(&mut self) {
    match &self.sink {
      AudioSinkType::Spatial(sink) => sink.play(),
      AudioSinkType::Standard(sink) => sink.play(),
    }
  }

  // Stop all playback
  pub fn stop(&mut self) {
    match &self.sink {
      AudioSinkType::Spatial(sink) => sink.stop(),
      AudioSinkType::Standard(sink) => sink.stop(),
    }
  }

  pub fn set_volume(&mut self, volume: f32) {
    self.volume = volume;
    match &self.sink {
      AudioSinkType::Spatial(sink) => sink.set_volume(volume),
      AudioSinkType::Standard(sink) => sink.set_volume(volume),
    }
  }

  pub fn set_pitch(&mut self, pitch: f32) {
    self.pitch = pitch;
    match &self.sink {
      AudioSinkType::Spatial(sink) => sink.set_speed(pitch),
      AudioSinkType::Standard(sink) => sink.set_speed(pitch),
    }
  }

  pub fn is_empty(&mut self) -> bool {
    match &self.sink {
      AudioSinkType::Spatial(sink) => return sink.empty(),
      AudioSinkType::Standard(sink) => return sink.empty(),
    }
  }
}
