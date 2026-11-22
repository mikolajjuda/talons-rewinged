use crate::game::{Mode, NO_BEST_TIME};
use std::collections::HashMap;
use std::fs;

pub struct BestTimes {
    path: &'static str,
    times: HashMap<&'static str, f32>,
}

impl BestTimes {
    pub fn load(path: &'static str) -> Self {
        let mut times = HashMap::new();

        if let Ok(text) = fs::read_to_string(path) {
            for line in text.lines() {
                let mut parts = line.split_whitespace();
                let (Some(key), Some(value)) = (parts.next(), parts.next()) else {
                    continue;
                };
                let Ok(time) = value.parse::<f32>() else {
                    continue;
                };
                if let Some(mode) = Mode::ALL.iter().find(|mode| mode.key() == key) {
                    times.insert(mode.key(), time);
                }
            }
        }

        Self { path, times }
    }

    pub fn best(&self, mode: Mode) -> f32 {
        self.times.get(mode.key()).copied().unwrap_or(NO_BEST_TIME)
    }

    pub fn record(&mut self, mode: Mode, time: f32) {
        if time >= self.best(mode) {
            return;
        }
        self.times.insert(mode.key(), time);
        self.save();
    }

    fn save(&self) {
        let text: String = Mode::ALL
            .iter()
            .filter_map(|mode| {
                self.times
                    .get(mode.key())
                    .map(|time| format!("{} {:.2}\n", mode.key(), time))
            })
            .collect();
        let _ = fs::write(self.path, text);
    }
}
