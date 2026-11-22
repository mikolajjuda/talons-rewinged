use super::SAMPLE_RATE;
use super::score::NoteEvent;

const MUSIC_AMPLITUDE: f32 = 0.12;
const SFX_AMPLITUDE: f32 = 0.26;

pub struct SynthState {
    song: Vec<NoteEvent>,
    music_enabled: bool,
    muted: bool,
    note_index: usize,
    note_sample: usize,
    effect_frequency: f32,
    effect_remaining: usize,
    phase: f32,
}

impl SynthState {
    pub fn new(song: Vec<NoteEvent>) -> Self {
        Self {
            song,
            music_enabled: false,
            muted: false,
            note_index: 0,
            note_sample: 0,
            effect_frequency: 0.0,
            effect_remaining: 0,
            phase: 0.0,
        }
    }

    pub fn start_music(&mut self) {
        if self.muted || self.music_enabled {
            return;
        }
        self.note_index = 0;
        self.note_sample = 0;
        self.music_enabled = true;
    }

    pub fn stop_music(&mut self) {
        self.music_enabled = false;
        self.note_index = 0;
        self.note_sample = 0;
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        if muted {
            self.music_enabled = false;
            self.effect_remaining = 0;
        }
    }

    pub fn play_effect(&mut self, frequency: f32, seconds: f32) {
        if self.muted {
            return;
        }
        self.effect_frequency = frequency;
        self.effect_remaining = (seconds * SAMPLE_RATE as f32).round() as usize;
    }

    pub fn next_sample(&mut self) -> f32 {
        if self.muted {
            return 0.0;
        }

        let music_frequency = self.advance_music();
        let (frequency, amplitude) = if self.effect_remaining > 0 {
            self.effect_remaining -= 1;
            (self.effect_frequency, SFX_AMPLITUDE)
        } else {
            (music_frequency, MUSIC_AMPLITUDE)
        };

        if frequency <= 0.0 {
            return 0.0;
        }

        self.phase = (self.phase + frequency / SAMPLE_RATE as f32).fract();
        if self.phase < 0.5 { amplitude } else { -amplitude }
    }

    fn advance_music(&mut self) -> f32 {
        if !self.music_enabled || self.song.is_empty() {
            return 0.0;
        }

        let note = self.song[self.note_index];
        let frequency = if self.note_sample < note.active_samples {
            note.frequency
        } else {
            0.0
        };

        self.note_sample += 1;
        if self.note_sample >= note.total_samples {
            self.note_sample = 0;
            self.note_index = (self.note_index + 1) % self.song.len();
        }
        frequency
    }
}
