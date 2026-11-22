use super::SAMPLE_RATE;

const PIT_CLOCK_HZ: f32 = 1_193_182.0;
const TEMPLE_TEMPO: f32 = 2.5;
const STACCATO_FACTOR: f32 = 0.9;
const DEFAULT_OCTAVE: i32 = 4;

const TALONS_SONG: [&str; 4] = [
    "5eCGFsD4A5e.C4sG5eGDqCDeGsGG4qG",
    "5eCGFsD4A5e.C4sG5eGDqCDeGsGG4qG",
    "5eGECGC4A5FCsC4B5C4B5e.GsG4qGB",
    "5eGECGC4A5FCsC4B5C4B5e.GsG4qGB",
];

#[derive(Clone, Copy)]
pub struct NoteEvent {
    pub frequency: f32,
    pub active_samples: usize,
    pub total_samples: usize,
}

pub fn talons_song() -> Vec<NoteEvent> {
    let mut song = Vec::new();
    for line in TALONS_SONG {
        parse_play_string(line, &mut song);
    }
    song
}

pub fn effect_frequency(ona: i32) -> f32 {
    pit_quantize_frequency(ona_to_frequency(ona))
}

fn ona_to_frequency(ona: i32) -> f32 {
    if ona <= 0 {
        0.0
    } else {
        (440.0 / 32.0) * 2.0f32.powf(ona as f32 / 12.0)
    }
}

fn pit_quantize_frequency(frequency: f32) -> f32 {
    if frequency <= 0.0 {
        return 0.0;
    }
    let divisor = (PIT_CLOCK_HZ / frequency).round().clamp(1.0, 65535.0);
    PIT_CLOCK_HZ / divisor
}

fn semitone_of(letter: char) -> i32 {
    match letter {
        'A' => 0,
        'B' => 2,
        'C' => 3,
        'D' => 5,
        'E' => 7,
        'F' => 8,
        _ => 10,
    }
}

fn parse_play_string(script: &str, out: &mut Vec<NoteEvent>) {
    let mut octave = DEFAULT_OCTAVE;
    let mut note_length: f32 = 1.0;

    let bytes = script.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let symbol = bytes[index] as char;
        index += 1;

        match symbol {
            '0'..='9' => octave = (symbol as u8 - b'0') as i32,
            'w' => note_length = 4.0,
            'h' => note_length = 2.0,
            'q' => note_length = 1.0,
            'e' => note_length = 0.5,
            's' => note_length = 0.25,
            't' => note_length *= 2.0 / 3.0,
            '.' => note_length *= 1.5,
            'R' => {
                let total = samples_for(note_length);
                out.push(NoteEvent {
                    frequency: 0.0,
                    active_samples: 0,
                    total_samples: total.max(1),
                });
            }
            'A'..='G' => {
                let mut accidental = 0;
                if index < bytes.len() {
                    match bytes[index] as char {
                        '#' => {
                            accidental = 1;
                            index += 1;
                        }
                        'b' => {
                            accidental = -1;
                            index += 1;
                        }
                        _ => {}
                    }
                }

                let ona = (octave + 1) * 12 + semitone_of(symbol) + accidental;
                let total = samples_for(note_length);
                let active = ((total as f32 * STACCATO_FACTOR).round() as usize).min(total);

                out.push(NoteEvent {
                    frequency: effect_frequency(ona),
                    active_samples: active,
                    total_samples: total.max(1),
                });
            }
            _ => {}
        }
    }
}

fn samples_for(note_length: f32) -> usize {
    ((note_length / TEMPLE_TEMPO) * SAMPLE_RATE as f32).round() as usize
}
