use sdl2::JoystickSubsystem;
use sdl2::event::Event;
use sdl2::joystick::{HatState, Joystick};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct DeviceState {
    pub name: String,
    pub connected: bool,
    pub axes: BTreeMap<String, f32>,
    pub buttons: BTreeMap<String, bool>,
}

struct PadEntry {
    instance_id: u32,
    _joystick: Joystick,
    state: DeviceState,
}

#[derive(Default)]
pub struct SharedPads {
    pub version: u64,
    pub pads: Vec<DeviceState>,
}

pub fn run_joystick_thread(shared: Arc<Mutex<SharedPads>>) {
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
                    Ok(joy) => local_pads.push(make_pad_entry(joy)),
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
            let mut changed = handle_event(first, &joystick_sub, &mut local_pads);
            while let Some(ev) = event_pump.poll_event() {
                if handle_event(ev, &joystick_sub, &mut local_pads) {
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
                let entry = make_pad_entry(joy);
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
                    .insert(format!("Axis{axis_idx}"), norm_axis(value));
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
            axes.insert(format!("Axis{a}"), norm_axis(val));
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
