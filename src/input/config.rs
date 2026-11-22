use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AxisBinding {
    pub pad: String,
    pub axis: Option<String>,
    pub neg: Option<String>,
    pub pos: Option<String>,
    pub raw: [f32; 2],
    pub out: [f32; 2],
    pub deadzone: f32,
}

impl Default for AxisBinding {
    fn default() -> Self {
        Self {
            pad: String::new(),
            axis: None,
            neg: None,
            pos: None,
            raw: [-1.0, 1.0],
            out: [-1.0, 1.0],
            deadzone: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SwitchBinding {
    pub pad: String,
    pub button: Option<String>,
    pub axis: Option<String>,
    pub threshold: f32,
    pub invert: bool,
}

impl Default for SwitchBinding {
    fn default() -> Self {
        Self {
            pad: String::new(),
            button: None,
            axis: None,
            threshold: 0.25,
            invert: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct InputConfig {
    pub yaw: AxisBinding,
    pub throttle: AxisBinding,
    pub roll: AxisBinding,
    pub pitch: AxisBinding,
    pub switch: SwitchBinding,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            yaw: AxisBinding {
                pad: "Keyboard".into(),
                neg: Some("Q".into()),
                pos: Some("E".into()),
                raw: [-1.0, 1.0],
                out: [-1.0, 1.0],
                deadzone: 0.0,
                ..Default::default()
            },
            throttle: AxisBinding {
                pad: "Keyboard".into(),
                pos: Some("LeftShift".into()),
                raw: [0.0, 1.0],
                out: [0.0, 1.0],
                deadzone: 0.0,
                ..Default::default()
            },
            roll: AxisBinding {
                pad: "Keyboard".into(),
                neg: Some("A".into()),
                pos: Some("D".into()),
                raw: [-1.0, 1.0],
                out: [-1.0, 1.0],
                deadzone: 0.0,
                ..Default::default()
            },
            pitch: AxisBinding {
                pad: "Keyboard".into(),
                neg: Some("S".into()),
                pos: Some("W".into()),
                raw: [-1.0, 1.0],
                out: [-1.0, 1.0],
                deadzone: 0.0,
                ..Default::default()
            },
            switch: SwitchBinding {
                pad: "Keyboard".into(),
                button: Some("Space".into()),
                ..Default::default()
            },
        }
    }
}

impl InputConfig {
    pub fn load_or_default(path: &str) -> (Self, String) {
        match std::fs::read_to_string(path) {
            Ok(txt) => match serde_json::from_str(&txt) {
                Ok(cfg) => (cfg, format!("{path}: loaded")),
                Err(e) => (Self::default(), format!("{path}: parse error ({e})")),
            },
            Err(_) => (
                Self::default(),
                format!("{path}: not found (default keyboard)"),
            ),
        }
    }
}
