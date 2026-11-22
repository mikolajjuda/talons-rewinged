mod score;
mod synth;

use std::sync::{Arc, Mutex};
use tinyaudio::{OutputDevice, OutputDeviceParameters, run_output_device};

use synth::SynthState;

const SAMPLE_RATE: usize = 48000;
const CHANNELS: usize = 2;
const BUFFER_FRAMES: usize = 4096;

const CATCH_ONA: i32 = 74;
const CATCH_SECONDS: f32 = 0.20;
const CRASH_ONA: i32 = 62;
const CRASH_SECONDS: f32 = 0.50;

pub struct AudioSystem {
    state: Arc<Mutex<SynthState>>,
    music_enabled: bool,
    muted: bool,
    _device: Option<OutputDevice>,
}

impl AudioSystem {
    pub fn new() -> Self {
        let state = Arc::new(Mutex::new(SynthState::new(score::talons_song())));
        let callback_state = Arc::clone(&state);

        let device = run_output_device(
            OutputDeviceParameters {
                channels_count: CHANNELS,
                sample_rate: SAMPLE_RATE,
                channel_sample_count: BUFFER_FRAMES,
            },
            move |data| {
                let Ok(mut synth) = callback_state.lock() else {
                    data.fill(0.0);
                    return;
                };

                for frame in data.chunks_exact_mut(CHANNELS) {
                    let sample = synth.next_sample();
                    frame[0] = sample;
                    frame[1] = sample;
                }
            },
        )
        .ok();

        Self {
            state,
            music_enabled: false,
            muted: false,
            _device: device,
        }
    }

    pub fn ensure_music_playing(&mut self) {
        if self.muted || self.music_enabled {
            return;
        }
        self.music_enabled = true;
        self.with_synth(SynthState::start_music);
    }

    pub fn stop_music(&mut self) {
        if !self.music_enabled {
            return;
        }
        self.music_enabled = false;
        self.with_synth(SynthState::stop_music);
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        if self.muted {
            self.music_enabled = false;
        }
        let muted = self.muted;
        self.with_synth(|synth| synth.set_muted(muted));
    }

    pub fn play_catch(&self) {
        self.play_effect(CATCH_ONA, CATCH_SECONDS);
    }

    pub fn play_crash(&self) {
        self.play_effect(CRASH_ONA, CRASH_SECONDS);
    }

    fn play_effect(&self, ona: i32, seconds: f32) {
        if self.muted {
            return;
        }
        let frequency = score::effect_frequency(ona);
        self.with_synth(|synth| synth.play_effect(frequency, seconds));
    }

    fn with_synth(&self, action: impl FnOnce(&mut SynthState)) {
        if let Ok(mut synth) = self.state.lock() {
            action(&mut synth);
        }
    }
}
