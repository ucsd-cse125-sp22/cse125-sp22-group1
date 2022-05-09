# Audio Engine
This is the audio engine for chariots, programmed in Rust using Rodio.

## Audio Context
Define & Set up the audio output devices used to playback audio

Usage: `AudioCtx::new();`

### Struct
* _stream: `OutputStream`
* stream_handle: `OutputStreamHandle`

## Source Options
Options that modulate the source of audio that is being played

Usage: `SourceOptions::new();`

### Struct
* fade_in: `Duration`,
* repeat: `bool`,
* skip_duration: `Duration`,
* take_duration: `Duration`,
* pitch: `f32`,

## Audio Thread
A multipurpose class that is designed to wrap the main Audio Sink (thread that plays audio in Rodio) and provide additional control over playback.

Usage: `AudioThread::new(&AudioCtx, PathBuf, SourceOptions);`

### Struct
* time_start: `SystemTime`,
* volume: `f32`,
* pitch: `f32`,
* file: `File`,
* sink: `Sink`,
* src_opt: `SourceOptions`

## Audio Source
A class designed to maintain and control all Audio Threads.

Usage: `AudioSource::new(Path);`

### Struct
* tracks: `Vec<PathBuf>`,
* threads: `Vec<AudioThread>`,
* volume: `f32`,
* pitch: `f32`,
* emitter_pos: `[f32; 3]`,
* left_ear: `[f32; 3]`,
* right_ear: `[f32; 3]`,

## Instructions
1. Begin with the relevant imports

```
use audio::{AudioCtx, SourceOptions, AudioSource};
mod audio;
```

2. Set up an audio context (this is what determines what output we are using on the device)
By default, it is automatically configured to identify the standard output the device is using.

`let audio_ctx = AudioCtx::new();`

3. Create an AudioSource. This manager is a collection of audio threads that is able to be controlled as a group. It's a good idea to separate music, ambient, and SFX tracks via their own audio managers.

```
let mut music_manager = AudioSource::new("audio/music");
let mut amb_manager = AudioSource::new("audio/ambient");
let mut sfx_manager = AudioSource::new("audio/sfx");
```

4. Define a sound source options and play sounds

Normal playback, will play alongside all other current sounds. Good for SFX.
`track_id` is the usize index of the track to play in the source directory.

```
let src_opt = SourceOptions::new();
sfx_manager.play(track_id, &audio_ctx, src_opt);
```

Crossfade playback, will play a new sound fading in with all other current sounds fading out. Good for Music & Ambient where only 1 thing should be playing at a time.

```
// 1000 ms fade in & fade out
let src_opt = SourceOptions::new();
music_manager.play_cf(track_id, &audio_ctx, src_opt, Duration::from_millis(1000));
```

Use additional control settings if necessary:

```
// Volume control
music_manager.set_volume(1.0);
```

```
// Pitch control (affects playback speed)
music_manager.set_pitch(1.0);
```
