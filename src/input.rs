use gilrs::{Axis, Button, Event, EventType, GamepadId, Gilrs};
use macroquad::input::get_keys_down;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

pub struct DeviceState {
    pub name: String,
    pub connected: bool,
    pub axes: BTreeMap<String, f32>,
    pub buttons: BTreeMap<String, bool>,
}

pub struct GameInputs {
    pub yaw: f32,
    pub throttle: f32,
    pub roll: f32,
    pub pitch: f32,
    pub switch: bool,
}

pub struct InputSystem {
    pub gilrs: Gilrs,
    pub keyboard: DeviceState,
    pub pads: Vec<(GamepadId, DeviceState)>,
    pub cfg: InputConfig,
    pub config_path: String,
    pub status: String,
    pub selected: usize,
}

impl InputSystem {
    pub fn new(config_path: &str) -> Self {
        let gilrs = Gilrs::new().expect("gilrs init failed");
        let mut pads = Vec::new();
        for (id, pad) in gilrs.gamepads() {
            if pad.is_connected() {
                pads.push((
                    id,
                    DeviceState {
                        name: pad.name().to_string(),
                        connected: true,
                        axes: BTreeMap::new(),
                        buttons: BTreeMap::new(),
                    },
                ));
            }
        }
        let (cfg, status) = InputConfig::load_or_default(config_path);
        Self {
            gilrs,
            keyboard: DeviceState {
                name: "Keyboard".into(),
                connected: true,
                axes: BTreeMap::new(),
                buttons: BTreeMap::new(),
            },
            pads,
            cfg,
            config_path: config_path.to_string(),
            status,
            selected: 0,
        }
    }

    pub fn reload(&mut self) {
        (self.cfg, self.status) = InputConfig::load_or_default(&self.config_path);
    }

    pub fn devices(&self) -> Vec<&DeviceState> {
        let mut devs = vec![&self.keyboard];
        devs.extend(self.pads.iter().map(|(_, d)| d).filter(|d| d.connected));
        devs
    }

    pub fn poll(&mut self) -> GameInputs {
        for v in self.keyboard.buttons.values_mut() {
            *v = false;
        }
        for k in get_keys_down() {
            self.keyboard.buttons.insert(format!("{k:?}"), true);
        }

        while let Some(Event { id, event, .. }) = self.gilrs.next_event() {
            let pad_name = self.gilrs.gamepad(id).name().to_string();
            let dev = if let Some((_, d)) = self.pads.iter_mut().find(|(pid, _)| *pid == id) {
                d
            } else {
                self.pads.push((
                    id,
                    DeviceState {
                        name: pad_name.clone(),
                        connected: true,
                        axes: BTreeMap::new(),
                        buttons: BTreeMap::new(),
                    },
                ));
                &mut self.pads.last_mut().unwrap().1
            };
            dev.name = pad_name;

            match event {
                EventType::Connected => dev.connected = true,
                EventType::Disconnected => dev.connected = false,
                EventType::AxisChanged(axis, val, code) => {
                    let key = if axis != Axis::Unknown {
                        format!("{axis:?}")
                    } else {
                        format!("{code:?}")
                    };
                    dev.axes.insert(key, val);
                }
                EventType::ButtonPressed(btn, code) => {
                    let key = if btn != Button::Unknown {
                        format!("{btn:?}")
                    } else {
                        format!("{code:?}")
                    };
                    dev.buttons.insert(key, true);
                }
                EventType::ButtonReleased(btn, code) => {
                    let key = if btn != Button::Unknown {
                        format!("{btn:?}")
                    } else {
                        format!("{code:?}")
                    };
                    dev.buttons.insert(key, false);
                }
                _ => {}
            }
        }

        let def = InputConfig::default();
        GameInputs {
            yaw: self.eval_axis(&self.cfg.yaw, &def.yaw, false),
            throttle: self.eval_axis(&self.cfg.throttle, &def.throttle, true),
            roll: self.eval_axis(&self.cfg.roll, &def.roll, false),
            pitch: self.eval_axis(&self.cfg.pitch, &def.pitch, false),
            switch: self.eval_switch(&self.cfg.switch, &def.switch),
        }
    }

    fn matching_devices<'a>(&'a self, pat: &str) -> Vec<&'a DeviceState> {
        let p = pat.to_ascii_lowercase();
        if p == "keyboard" {
            return vec![&self.keyboard];
        }
        self.pads
            .iter()
            .map(|(_, d)| d)
            .filter(|d| {
                d.connected
                    && (p == "*" || p.is_empty() || d.name.to_ascii_lowercase().contains(&p))
            })
            .collect()
    }

    fn remap(val: f32, raw: [f32; 2], out: [f32; 2], deadzone: f32, unipolar: bool) -> f32 {
        let span = raw[1] - raw[0];
        let t = if span.abs() < 1e-6 {
            0.0
        } else {
            ((val - raw[0]) / span).clamp(0.0, 1.0)
        };
        let mut v = out[0] + t * (out[1] - out[0]);
        if unipolar {
            v = v.clamp(0.0, 1.0);
            if deadzone > 0.0 && v <= deadzone {
                v = 0.0;
            }
        } else {
            v = v.clamp(-1.0, 1.0);
            if deadzone > 0.0 && v.abs() <= deadzone {
                v = 0.0;
            }
        }
        v
    }

    fn eval_binding_on_dev(
        &self,
        b: &AxisBinding,
        dev: &DeviceState,
        unipolar: bool,
    ) -> Option<f32> {
        let neg = b
            .neg
            .as_ref()
            .and_then(|k| dev.buttons.get(k))
            .copied()
            .unwrap_or(false);
        let pos = b
            .pos
            .as_ref()
            .and_then(|k| dev.buttons.get(k))
            .copied()
            .unwrap_or(false);

        if neg || pos {
            let val = match (neg, pos) {
                (true, false) => b.raw[0],
                (false, true) => b.raw[1],
                _ => {
                    if unipolar {
                        b.raw[0]
                    } else {
                        (b.raw[0] + b.raw[1]) * 0.5
                    }
                }
            };
            Some(Self::remap(val, b.raw, b.out, b.deadzone, unipolar))
        } else {
            None
        }
    }

    fn eval_axis(&self, b: &AxisBinding, default_b: &AxisBinding, unipolar: bool) -> f32 {
        // 1. Keyboard button presses ALWAYS override controller axes:
        // Check active binding buttons on keyboard
        if let Some(val) = self.eval_binding_on_dev(b, &self.keyboard, unipolar) {
            return val;
        }
        // Check default keyboard buttons on keyboard (seamless fallback if controller disconnected)
        if let Some(val) = self.eval_binding_on_dev(default_b, &self.keyboard, unipolar) {
            return val;
        }

        // 2. If no keyboard button is pressed, read matching device (gamepad controller)
        for dev in self.matching_devices(&b.pad) {
            if let Some(id) = &b.axis {
                if let Some(&raw_val) = dev.axes.get(id) {
                    return Self::remap(raw_val, b.raw, b.out, b.deadzone, unipolar);
                }
            }
            if let Some(val) = self.eval_binding_on_dev(b, dev, unipolar) {
                return val;
            }
        }

        0.0
    }

    fn eval_switch(&self, b: &SwitchBinding, default_b: &SwitchBinding) -> bool {
        // 1. Check keyboard first (configured button on keyboard OR default Space on keyboard)
        if let Some(btn) = &b.button {
            if self.keyboard.buttons.get(btn).copied().unwrap_or(false) {
                return if b.invert { false } else { true };
            }
        }
        if let Some(btn) = &default_b.button {
            if self.keyboard.buttons.get(btn).copied().unwrap_or(false) {
                return true;
            }
        }

        // 2. Check matching device (gamepad controller)
        for dev in self.matching_devices(&b.pad) {
            if dev.name != "Keyboard" {
                if let Some(btn) = &b.button {
                    if let Some(&pressed) = dev.buttons.get(btn) {
                        if if b.invert { !pressed } else { pressed } {
                            return true;
                        }
                    }
                }
            }
            if let Some(ax) = &b.axis {
                if let Some(&val) = dev.axes.get(ax) {
                    let v = if b.invert { -val } else { val };
                    if v >= b.threshold {
                        return true;
                    }
                }
            }
        }

        false
    }
}
