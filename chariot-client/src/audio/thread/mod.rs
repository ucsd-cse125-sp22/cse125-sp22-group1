#![allow(dead_code)]

use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use rodio::{Decoder, Sink, Source, SpatialSink};

use crate::audio::thread::fade_source::FadeSource;
use context::AudioCtx;
use options::SourceOptions;

// Buffered Audio Source
pub type AudioBuffer = &'static [u8];

pub mod context;
mod fade_source;
pub mod options;

enum AudioSinkType {
    Spatial(SpatialSink),
    Standard(Sink),
}

pub struct AudioThread {
    time_start: SystemTime,
    volume: f32,
    pitch: f32,
    source: AudioBuffer,
    sink: AudioSinkType,
    // basically, a thread safe duration to set as the fade out
    fade_out: Arc<Mutex<Option<Duration>>>,
    src_opt: SourceOptions,
}

impl AudioThread {
    pub fn new(ctx: &AudioCtx, source: AudioBuffer, src_opt: SourceOptions) -> Self {
        return if src_opt.emitter_pos != [0.0; 3]
            || src_opt.left_ear != [0.0; 3]
            || src_opt.right_ear != [0.0; 3]
        {
            // Spatial Sink
            let spatial_sink = SpatialSink::try_new(
                &ctx.stream_handle,
                src_opt.emitter_pos,
                src_opt.left_ear,
                src_opt.right_ear,
            );
            let sink = match spatial_sink {
                Ok(s) => AudioSinkType::Spatial(s),
                Err(err) => {
                    println!(
                        "There was an error in creating the spatial sink instance: {}",
                        err
                    );
                    AudioSinkType::Standard(Sink::new_idle().0)
                }
            };

            Self {
                time_start: SystemTime::now(),
                volume: 1.0,
                pitch: 1.0,
                source,
                sink,
                fade_out: Arc::new(Mutex::new(None)),
                src_opt,
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

            Self {
                time_start: SystemTime::now(),
                volume: 1.0,
                pitch: 1.0,
                source,
                sink,
                fade_out: Arc::new(Mutex::new(None)),
                src_opt,
            }
        };
    }

    pub fn time_alive(&mut self) -> Option<Duration> {
        let time_elapsed = self.time_start.elapsed();
        return match time_elapsed {
            Ok(t) => Some(t),
            Err(err) => {
                println!("There was an error at retrieving thread lifetime, {}", err);
                None
            }
        };
    }

    pub fn play(&mut self) {
        let fade_out = self.fade_out.clone();

        let source = Decoder::new(Cursor::new(self.source))
            .expect("failed to decode track")
            .skip_duration(self.src_opt.skip_duration)
            .speed(self.src_opt.pitch)
            .fade_in(self.src_opt.fade_in)
            .take_duration(self.src_opt.take_duration);

        // Apply Repeat and fadesource filters
        // I really shouldn't have to do it this way but when it compiles it'll unroll like this anyways so
        if self.src_opt.repeat {
            let source = FadeSource::new(source.repeat_infinite().buffered()).periodic_access(
                Duration::from_millis(5),
                move |src| {
                    if !src.is_fadeout() {
                        let fade_out = fade_out.lock().unwrap();
                        if let Some(requested_duration) = *fade_out {
                            src.set_fadeout(requested_duration);
                        }
                    }
                },
            );

            match &self.sink {
                AudioSinkType::Spatial(sink) => sink.append(source),
                AudioSinkType::Standard(sink) => sink.append(source),
            }
        } else {
            let source = FadeSource::new(source.buffered()).periodic_access(
                Duration::from_millis(5),
                move |src| {
                    if !src.is_fadeout() {
                        let fade_out = fade_out.lock().unwrap();
                        if let Some(requested_duration) = *fade_out {
                            src.set_fadeout(requested_duration);
                        }
                    }
                },
            );

            match &self.sink {
                AudioSinkType::Spatial(sink) => sink.append(source),
                AudioSinkType::Standard(sink) => sink.append(source),
            }
        }

        self.time_start = SystemTime::now();
    }

    // Fade out the sink
    pub fn fade_out(&mut self, duration: Duration) {
        let mut fade_out = self.fade_out.lock().unwrap();
        *fade_out = Some(duration);
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

    pub fn is_empty(&self) -> bool {
        return match &self.sink {
            AudioSinkType::Spatial(sink) => sink.empty(),
            AudioSinkType::Standard(sink) => sink.empty(),
        };
    }
}
