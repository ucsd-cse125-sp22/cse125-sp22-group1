use std::io::Cursor;
use std::time::{Duration, SystemTime};

use rodio::{Decoder, Sink, Source, SpatialSink};

use context::AudioCtx;
use options::SourceOptions;

// Buffered Audio Source
pub type AudioBuffer = &'static [u8];

pub mod context;
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
        let source = Decoder::new(Cursor::new(self.source)).expect("failed to decode track");

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
        // TODO
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
