use std::sync::{Arc, Mutex};
use tinyaudio::{OutputDevice, OutputDeviceParameters, run_output_device};

const SAMPLE_RATE: usize = 44100;
const CHANNELS: usize = 2;
const BUFFER_FRAMES: usize = 512;
const PIT_CLOCK_HZ: f32 = 1_193_182.0;

const TEMPLE_TEMPO: f32 = 2.5;
const STACCATO_FACTOR: f32 = 0.9;

const MUSIC_AMPLITUDE: f32 = 0.12;
const SFX_AMPLITUDE: f32 = 0.26;

const TALONS_SONG: [&str; 4] = [
    "5eCGFsD4A5e.C4sG5eGDqCDeGsGG4qG",
    "5eCGFsD4A5e.C4sG5eGDqCDeGsGG4qG",
    "5eGECGC4A5FCsC4B5C4B5e.GsG4qGB",
    "5eGECGC4A5FCsC4B5C4B5e.GsG4qGB",
];

#[derive(Clone, Copy)]
struct NoteEvent {
    freq: f32,
    active_samples: usize,
    total_samples: usize,
}

struct SynthState {
    song: Vec<NoteEvent>,
    music_enabled: bool,
    muted: bool,
    note_idx: usize,
    note_sample: usize,
    sfx_freq: f32,
    sfx_remaining: usize,
    sfx_gap_remaining: usize,
    phase: f32,
}

impl SynthState {
    fn new(song: Vec<NoteEvent>) -> Self {
        Self {
            song,
            music_enabled: false,
            muted: false,
            note_idx: 0,
            note_sample: 0,
            sfx_freq: 0.0,
            sfx_remaining: 0,
            sfx_gap_remaining: 0,
            phase: 0.0,
        }
    }

    fn next_sample(&mut self) -> f32 {
        if self.muted {
            self.sfx_remaining = 0;
            self.sfx_gap_remaining = 0;
            return 0.0;
        }

        let (freq, amp) = if self.sfx_remaining > 0 {
            self.sfx_remaining -= 1;
            (self.sfx_freq, SFX_AMPLITUDE)
        } else if self.sfx_gap_remaining > 0 {
            self.sfx_gap_remaining -= 1;
            (0.0, MUSIC_AMPLITUDE)
        } else if self.music_enabled && !self.song.is_empty() {
            let note = self.song[self.note_idx];
            let f = if self.note_sample < note.active_samples {
                note.freq
            } else {
                0.0
            };

            self.note_sample += 1;
            if self.note_sample >= note.total_samples {
                self.note_sample = 0;
                self.note_idx = (self.note_idx + 1) % self.song.len();
            }
            (f, MUSIC_AMPLITUDE)
        } else {
            (0.0, MUSIC_AMPLITUDE)
        };

        if freq <= 0.0 {
            return 0.0;
        }

        self.phase = (self.phase + freq / SAMPLE_RATE as f32).fract();
        if self.phase < 0.5 { amp } else { -amp }
    }
}

pub struct AudioSystem {
    state: Arc<Mutex<SynthState>>,
    _device: Option<OutputDevice>,
}

impl AudioSystem {
    pub fn new() -> Self {
        let mut song = Vec::new();
        for line in TALONS_SONG {
            parse_play_string(line, &mut song);
        }

        let state = Arc::new(Mutex::new(SynthState::new(song)));
        let cb_state = Arc::clone(&state);

        let device = run_output_device(
            OutputDeviceParameters {
                channels_count: CHANNELS,
                sample_rate: SAMPLE_RATE,
                channel_sample_count: BUFFER_FRAMES,
            },
            move |data| {
                let Ok(mut synth) = cb_state.lock() else {
                    data.fill(0.0);
                    return;
                };

                for frame in data.chunks_exact_mut(CHANNELS) {
                    let s = synth.next_sample();
                    frame[0] = s;
                    frame[1] = s;
                }
            },
        )
        .ok();

        Self {
            state,
            _device: device,
        }
    }

    pub fn ensure_music_playing(&mut self) {
        if let Ok(mut synth) = self.state.lock() {
            if !synth.muted && !synth.music_enabled {
                synth.note_idx = 0;
                synth.note_sample = 0;
                synth.music_enabled = true;
            }
        }
    }

    pub fn stop_music(&mut self) {
        if let Ok(mut synth) = self.state.lock() {
            synth.music_enabled = false;
            synth.note_idx = 0;
            synth.note_sample = 0;
        }
    }

    pub fn toggle_mute(&mut self) {
        if let Ok(mut synth) = self.state.lock() {
            synth.muted = !synth.muted;
            if synth.muted {
                synth.music_enabled = false;
            }
        }
    }

    pub fn play_catch(&self) {
        if let Ok(mut synth) = self.state.lock() {
            if !synth.muted {
                synth.sfx_freq = pit_quantize_freq(ona_to_freq(74));
                synth.sfx_remaining = (0.20 * SAMPLE_RATE as f32).round() as usize;
                synth.sfx_gap_remaining = (0.06 * SAMPLE_RATE as f32).round() as usize;
            }
        }
    }

    pub fn play_crash(&self) {
        if let Ok(mut synth) = self.state.lock() {
            if !synth.muted {
                synth.sfx_freq = pit_quantize_freq(ona_to_freq(62));
                synth.sfx_remaining = (0.50 * SAMPLE_RATE as f32).round() as usize;
                synth.sfx_gap_remaining = 0;
            }
        }
    }
}

fn ona_to_freq(ona: i32) -> f32 {
    if ona <= 0 {
        0.0
    } else {
        (440.0 / 32.0) * 2.0f32.powf(ona as f32 / 12.0)
    }
}

fn pit_quantize_freq(freq: f32) -> f32 {
    if freq <= 0.0 {
        return 0.0;
    }
    let divisor = (PIT_CLOCK_HZ / freq).round().clamp(1.0, 65535.0);
    PIT_CLOCK_HZ / divisor
}

fn parse_play_string(script: &str, out: &mut Vec<NoteEvent>) {
    let mut octave: i32 = 4;
    let mut note_len: f32 = 1.0;

    let bytes = script.as_bytes();
    let mut idx = 0;

    while idx < bytes.len() {
        let ch = bytes[idx] as char;
        idx += 1;

        match ch {
            '0'..='9' => octave = (ch as u8 - b'0') as i32,
            'w' => note_len = 4.0,
            'h' => note_len = 2.0,
            'q' => note_len = 1.0,
            'e' => note_len = 0.5,
            's' => note_len = 0.25,
            't' => note_len *= 2.0 / 3.0,
            '.' => note_len *= 1.5,
            'R' => {
                let total_secs = note_len / TEMPLE_TEMPO;
                let total_samples = (total_secs * SAMPLE_RATE as f32).round() as usize;
                out.push(NoteEvent {
                    freq: 0.0,
                    active_samples: 0,
                    total_samples: total_samples.max(1),
                });
            }
            'A'..='G' => {
                let base_semitone = match ch {
                    'A' => 0,
                    'B' => 2,
                    'C' => 3,
                    'D' => 5,
                    'E' => 7,
                    'F' => 8,
                    'G' => 10,
                    _ => 0,
                };

                let mut accidental = 0;
                if idx < bytes.len() {
                    match bytes[idx] as char {
                        '#' => {
                            accidental = 1;
                            idx += 1;
                        }
                        'b' => {
                            accidental = -1;
                            idx += 1;
                        }
                        _ => {}
                    }
                }

                let ona = (octave + 1) * 12 + base_semitone + accidental;
                let total_secs = note_len / TEMPLE_TEMPO;
                let active_secs = total_secs * STACCATO_FACTOR;

                let total_samples = (total_secs * SAMPLE_RATE as f32).round() as usize;
                let active_samples =
                    ((active_secs * SAMPLE_RATE as f32).round() as usize).min(total_samples);

                out.push(NoteEvent {
                    freq: pit_quantize_freq(ona_to_freq(ona)),
                    active_samples,
                    total_samples: total_samples.max(1),
                });
            }
            _ => {}
        }
    }
}
