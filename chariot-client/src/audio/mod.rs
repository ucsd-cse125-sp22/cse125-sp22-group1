use rodio::{source::Buffered, Decoder, Source};
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::time::Duration;

pub mod thread;

use self::thread::context::AudioCtx;
use self::thread::options::SourceOptions;
use thread::AudioBuffer;
use thread::AudioThread;

// For Playing Track Audio (Music / Ambient / SFX)
pub struct AudioSource {
    pub name: String,
    max_thread_id: u64,
    available_threads: Vec<u64>,
    tracks: HashMap<String, AudioBuffer>,
    threads: HashMap<u64, AudioThread>,
    volume: f32,
    pitch: f32,
}

impl AudioSource {
    pub fn new(path: &str) -> Self {
        let available_threads = Vec::new();
        let mut tracks = HashMap::new();
        let threads = HashMap::new();

        let paths = fs::read_dir(format!("./{}", path)).expect("Problem reading the directory: ");

        // Obtains buffers for each audio file in the provided paths
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
                        println!(
                            "There was a problem with creating a string from a filename in: {}",
                            file_path
                        );
                        continue;
                    }
                },
                None => {
                    println!(
                        "There was a problem with identifying the filename in: {}",
                        file_path
                    );
                    continue;
                }
            };

            // Try to open the file
            let file = fs::File::open(path_buf);
            let file = match file {
                Ok(f) => f,
                Err(err) => {
                    println!("Problem opening the file: {}", err);
                    continue;
                }
            };

            // Generate our buffered audio source
            let buf = BufReader::new(file);
            let source = Decoder::new(buf);
            let source = match source {
                Ok(s) => s.buffered(),
                Err(err) => {
                    println!("Problem decoding the file: {}", err);
                    continue;
                }
            };

            // Store the audio sources into our HashMap
            tracks.insert(file_name, source);
            println!("Loaded Track [{}] from: {}", tracks.len(), file_path);
        }

        Self {
            name: path.to_owned(),
            max_thread_id: 0,
            available_threads,
            threads,
            tracks,
            volume: 1.0,
            pitch: 1.0,
        }
    }

    // Clean Up Sink w/o Audio
    pub fn clean(&mut self) {
        self.threads.retain(|&_id, thread| {
            if !thread.is_empty() {
                true
            } else {
                self.available_threads.push(thread.thread_id);
                false
            }
        });

        println!(
            "[{}] {} Threads alive: {:?}",
            self.name,
            self.threads.len(),
            self.threads.keys()
        );
        println!(
            "[{}] Available Threads {:?}",
            self.name, self.available_threads
        );
    }

    // Finds a track in the tracklist (or returns None if it doesn't exist)
    pub fn get_track(&mut self, track_name: &str) -> Option<AudioBuffer> {
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
    pub fn spawn_thread(
        &mut self,
        ctx: &AudioCtx,
        source: AudioBuffer,
        opt: SourceOptions,
    ) -> AudioThread {
        // Default to a new maximum
        let mut thread_id = self.max_thread_id;

        // If we have an accessible thread_id that has already been cleaned
        if !self.available_threads.is_empty() {
            thread_id = match self.available_threads.pop() {
                Some(id) => id,
                None => {
                    // Increment our maximum track id
                    self.max_thread_id = self.max_thread_id + 1;

                    thread_id
                }
            }
        } else {
            // Increment our maximum track id
            self.max_thread_id = self.max_thread_id + 1;
        }

        println!(
            "[{}] Spawned an audio thread with ID {}",
            self.name, thread_id
        );

        // Create the new instance of an audio thread, and set the volume and pitch to the levels defined in source
        let mut thread = AudioThread::new(thread_id, ctx, source.clone(), opt);
        thread.set_volume(self.volume);
        thread.set_pitch(self.pitch);

        return thread;
    }

    // Get a specific audio thread from an ID
    pub fn get_thread(&mut self, id: u64) -> Option<&AudioThread> {
        return self.threads.get(&id);
    }

    // Get a specific audio thread from an ID
    pub fn get_mut_thread(&mut self, id: u64) -> Option<&mut AudioThread> {
        return self.threads.get_mut(&id);
    }

    // Play Audio
    pub fn play(&mut self, track_name: &str, ctx: &AudioCtx, opt: SourceOptions) -> Option<u64> {
        // Clean Up All Stopped Threads
        self.clean();

        let source = match self.get_track(track_name) {
            Some(s) => s,
            None => {
                return None;
            }
        };

        let mut thread = self.spawn_thread(ctx, source, opt);
        let thread_id = thread.thread_id;

        thread.play();
        self.threads.insert(thread_id, thread);

        return Some(thread_id);
    }

    // Play an Audio and crossfade out all currently playing tracks
    pub fn play_cf(
        &mut self,
        track_name: &str,
        ctx: &AudioCtx,
        mut opt: SourceOptions,
        duration: Duration,
    ) -> Option<u64> {
        // Clean Up All Stopped Threads
        self.clean();

        // Enable Fade-In on newest thread
        if self.threads.len() > 0 {
            opt.fade_in = duration;
        }

        let source = match self.get_track(track_name) {
            Some(s) => s,
            None => {
                return None;
            }
        };

        let mut thread = self.spawn_thread(ctx, source, opt);
        let thread_id = thread.thread_id;

        self.fade_all_threads(duration);

        thread.play();
        self.threads.insert(thread_id, thread);

        return Some(thread_id);
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
