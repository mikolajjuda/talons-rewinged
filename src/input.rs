use macroquad::input::get_keys_down;
use sdl2::JoystickSubsystem;
use sdl2::event::Event;
use sdl2::joystick::{HatState, Joystick};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;

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

#[derive(Clone)]
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

struct PadEntry {
    instance_id: u32,
    _joystick: Joystick,
    state: DeviceState,
}

#[derive(Default)]
struct SharedPads {
    version: u64,
    pads: Vec<DeviceState>,
}

pub struct InputSystem {
    shared: Arc<Mutex<SharedPads>>,
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

        thread::spawn(move || Self::joystick_thread(shared_bg));

        let (cfg, status) = InputConfig::load_or_default(config_path);
        Self {
            shared,
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

    fn joystick_thread(shared: Arc<Mutex<SharedPads>>) {
        sdl2::hint::set("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1");
        sdl2::hint::set("SDL_JOYSTICK_THREAD", "1");
        sdl2::hint::set("SDL_JOYSTICK_WGI", "0");

        let sdl = sdl2::init().expect("sdl2 init failed");
        let joystick_sub = sdl.joystick().expect("sdl2 joystick init failed");
        let mut event_pump = sdl.event_pump().expect("sdl2 event_pump failed");

        let mut local_pads = Vec::new();
        match joystick_sub.num_joysticks() {
            Ok(count) => {
                for idx in 0..count {
                    match joystick_sub.open(idx) {
                        Ok(joy) => local_pads.push(Self::make_pad_entry(joy)),
                        Err(e) => eprintln!("sdl2 joystick open({idx}) failed: {e}"),
                    }
                }
            }
            Err(e) => eprintln!("sdl2 num_joysticks failed: {e}"),
        }

        if let Ok(mut guard) = shared.lock() {
            guard.pads = local_pads.iter().map(|p| p.state.clone()).collect();
            guard.version = guard.version.wrapping_add(1);
        }

        loop {
            if let Some(first) = event_pump.wait_event_timeout(4) {
                let mut changed = Self::handle_event(first, &joystick_sub, &mut local_pads);
                while let Some(ev) = event_pump.poll_event() {
                    if Self::handle_event(ev, &joystick_sub, &mut local_pads) {
                        changed = true;
                    }
                }
                if changed {
                    if let Ok(mut guard) = shared.lock() {
                        guard.pads = local_pads.iter().map(|p| p.state.clone()).collect();
                        guard.version = guard.version.wrapping_add(1);
                    }
                }
            }
        }
    }

    fn handle_event(
        event: Event,
        joystick_sub: &JoystickSubsystem,
        pads: &mut Vec<PadEntry>,
    ) -> bool {
        match event {
            Event::JoyDeviceAdded { which, .. } => match joystick_sub.open(which) {
                Ok(joy) => {
                    let id = joy.instance_id();
                    let entry = Self::make_pad_entry(joy);
                    if let Some(existing) = pads.iter_mut().find(|p| p.instance_id == id) {
                        *existing = entry;
                    } else {
                        pads.push(entry);
                    }
                    true
                }
                Err(e) => {
                    eprintln!("sdl2 joystick hotplug open({which}) failed: {e}");
                    false
                }
            },
            Event::JoyDeviceRemoved { which, .. } => {
                if let Some(p) = pads.iter_mut().find(|p| p.instance_id == which) {
                    p.state.connected = false;
                    true
                } else {
                    false
                }
            }
            Event::JoyAxisMotion {
                which,
                axis_idx,
                value,
                ..
            } => {
                if let Some(p) = pads.iter_mut().find(|p| p.instance_id == which) {
                    p.state
                        .axes
                        .insert(format!("Axis{axis_idx}"), Self::norm_axis(value));
                    true
                } else {
                    false
                }
            }
            Event::JoyButtonDown {
                which, button_idx, ..
            } => {
                if let Some(p) = pads.iter_mut().find(|p| p.instance_id == which) {
                    p.state.buttons.insert(format!("Button{button_idx}"), true);
                    true
                } else {
                    false
                }
            }
            Event::JoyButtonUp {
                which, button_idx, ..
            } => {
                if let Some(p) = pads.iter_mut().find(|p| p.instance_id == which) {
                    p.state.buttons.insert(format!("Button{button_idx}"), false);
                    true
                } else {
                    false
                }
            }
            Event::JoyHatMotion {
                which,
                hat_idx,
                state,
                ..
            } => {
                if let Some(p) = pads.iter_mut().find(|p| p.instance_id == which) {
                    let (hx, hy) = match state {
                        HatState::Centered => (0.0, 0.0),
                        HatState::Up => (0.0, 1.0),
                        HatState::Down => (0.0, -1.0),
                        HatState::Left => (-1.0, 0.0),
                        HatState::Right => (1.0, 0.0),
                        HatState::LeftUp => (-1.0, 1.0),
                        HatState::LeftDown => (-1.0, -1.0),
                        HatState::RightUp => (1.0, 1.0),
                        HatState::RightDown => (1.0, -1.0),
                    };
                    p.state.axes.insert(format!("Hat{hat_idx}X"), hx);
                    p.state.axes.insert(format!("Hat{hat_idx}Y"), hy);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn make_pad_entry(joy: Joystick) -> PadEntry {
        let mut axes = BTreeMap::new();
        for a in 0..joy.num_axes() {
            if let Ok(val) = joy.axis(a) {
                axes.insert(format!("Axis{a}"), Self::norm_axis(val));
            }
        }
        let mut buttons = BTreeMap::new();
        for b in 0..joy.num_buttons() {
            if let Ok(pressed) = joy.button(b) {
                buttons.insert(format!("Button{b}"), pressed);
            }
        }
        PadEntry {
            instance_id: joy.instance_id(),
            state: DeviceState {
                name: joy.name(),
                connected: joy.attached(),
                axes,
                buttons,
            },
            _joystick: joy,
        }
    }

    fn norm_axis(v: i16) -> f32 {
        if v < 0 {
            v as f32 / 32768.0
        } else {
            v as f32 / 32767.0
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
