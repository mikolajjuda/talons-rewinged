mod config;
mod pads;

use macroquad::input::get_keys_down;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;

use config::{AxisBinding, InputConfig, SwitchBinding};
use pads::{SharedPads, run_joystick_thread};

pub use pads::DeviceState;

pub struct GameInputs {
    pub yaw: f32,
    pub throttle: f32,
    pub roll: f32,
    pub pitch: f32,
    pub switch: bool,
}

pub struct InputSystem {
    shared: Arc<Mutex<SharedPads>>,
    defaults: InputConfig,
    last_version: u64,
    pub keyboard: DeviceState,
    pads_snapshot: Vec<DeviceState>,
    pub cfg: InputConfig,
    pub config_path: String,
    pub status: String,
    pub selected: usize,
}

impl InputSystem {
    pub fn new(config_path: &str) -> Self {
        let shared = Arc::new(Mutex::new(SharedPads::default()));
        let shared_bg = Arc::clone(&shared);

        thread::spawn(move || run_joystick_thread(shared_bg));

        let (cfg, status) = InputConfig::load_or_default(config_path);
        Self {
            shared,
            defaults: InputConfig::default(),
            last_version: 0,
            keyboard: DeviceState {
                name: "Keyboard".into(),
                connected: true,
                axes: BTreeMap::new(),
                buttons: BTreeMap::new(),
            },
            pads_snapshot: Vec::new(),
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
        devs.extend(self.pads_snapshot.iter().filter(|d| d.connected));
        devs
    }

    pub fn poll(&mut self) -> GameInputs {
        for v in self.keyboard.buttons.values_mut() {
            *v = false;
        }
        for k in get_keys_down() {
            self.keyboard.buttons.insert(format!("{k:?}"), true);
        }

        if let Ok(guard) = self.shared.lock() {
            if guard.version != self.last_version {
                self.pads_snapshot = guard.pads.clone();
                self.last_version = guard.version;
            }
        }

        let def = &self.defaults;
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
        self.pads_snapshot
            .iter()
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
        if let Some(val) = self.eval_binding_on_dev(b, &self.keyboard, unipolar) {
            return val;
        }
        if let Some(val) = self.eval_binding_on_dev(default_b, &self.keyboard, unipolar) {
            return val;
        }

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
        if let Some(btn) = &b.button {
            if self.keyboard.buttons.get(btn).copied().unwrap_or(false) {
                return !b.invert;
            }
        }
        if let Some(btn) = &default_b.button {
            if self.keyboard.buttons.get(btn).copied().unwrap_or(false) {
                return true;
            }
        }

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
