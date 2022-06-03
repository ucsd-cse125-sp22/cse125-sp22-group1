#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use thread::AudioBuffer;
use thread::AudioThread;

use self::thread::context::AudioCtx;
use self::thread::options::SourceOptions;

pub mod thread;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct AudioThreadHandle(usize);

impl AudioThreadHandle {
    fn unique() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

// For Playing Track Audio (Music / Ambient / SFX)
pub struct AudioManager {
    threads: HashMap<AudioThreadHandle, AudioThread>,
    volume: f32,
    pitch: f32,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            threads: HashMap::new(),
            volume: 1.0,
            pitch: 1.0,
        }
    }

    // Clean Up Sink w/o Audio
    pub fn clean(&mut self) {
        self.threads.retain(|&_id, thread| !thread.is_empty());
    }

    // Spawns a new thread with preapplied volume & pitch for use
    fn spawn_thread(
        &mut self,
        ctx: &AudioCtx,
        track_audio: AudioBuffer,
        opt: SourceOptions,
    ) -> AudioThread {
        // Create the new instance of an audio thread, and set the volume and pitch to the levels defined in source
        let mut thread = AudioThread::new(ctx, track_audio, opt);
        thread.set_volume(self.volume);
        thread.set_pitch(self.pitch);

        thread
    }

    // Get a specific audio thread from a handle
    pub fn get_thread(&mut self, handle: &AudioThreadHandle) -> Option<&AudioThread> {
        self.threads.get(handle)
    }

    // Get a specific audio thread, MUTABLY, from a handle
    pub fn get_mut_thread(&mut self, handle: &AudioThreadHandle) -> Option<&mut AudioThread> {
        self.threads.get_mut(handle)
    }

    // Play Audio
    pub fn play(
        &mut self,
        track_audio: AudioBuffer,
        ctx: &AudioCtx,
        opt: SourceOptions,
    ) -> AudioThreadHandle {
        // Clean Up All Stopped Threads
        self.clean();

        let mut thread = self.spawn_thread(ctx, track_audio, opt);
        let thread_id = AudioThreadHandle::unique();

        thread.play();
        self.threads.insert(thread_id, thread);

        thread_id
    }

    // Play an Audio and crossfade out all currently playing tracks
    pub fn play_cf(
        &mut self,
        track_audio: AudioBuffer,
        ctx: &AudioCtx,
        mut opt: SourceOptions,
        duration: Duration,
    ) -> AudioThreadHandle {
        // Clean Up All Stopped Threads
        self.clean();

        // Enable Fade-In on newest thread
        if self.threads.len() > 0 {
            opt.fade_in = duration;
        }

        let mut thread = self.spawn_thread(ctx, track_audio, opt);
        let thread_id = AudioThreadHandle::unique();

        self.fade_all_threads(duration);

        thread.play();
        self.threads.insert(thread_id, thread);

        thread_id
    }

    // Do a fade-out on all currently playing threads in this manager
    pub fn fade_all_threads(&mut self, duration: Duration) {
        // Fade Out All Currently Active Threads
        self.threads.iter_mut().for_each(|(_id, thread)| {
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
