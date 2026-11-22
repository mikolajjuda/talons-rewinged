use super::{BirdState, FlightModel};
use crate::input::GameInputs;
use crate::world::{HORIZONTAL_UNIT, World};
use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI};

const CONTROL_RATE: f32 = 2.2;
const MAX_STEP: f32 = 0.1;
const INITIAL_SPEED: f32 = 2.5;
const WATER_SPEED: f32 = 1.0;
const MIN_SPEED: f32 = 0.1;
const MAX_SPEED: f32 = 5.0;
const SPEED_UNIT: f32 = 1000.0 * HORIZONTAL_UNIT;
const LOOP_HZ: f32 = 30000.0 / 1001.0;
const LEVEL_RATE: f32 = 1.0;
const LEVEL_PITCH: f32 = -FRAC_PI_8;

pub struct ClassicModel {
    state: BirdState,
    speed: f32,
}

impl ClassicModel {
    pub fn new(spawn_pos: Vec3) -> Self {
        let mut model = Self {
            state: BirdState::at_rest(spawn_pos),
            speed: INITIAL_SPEED,
        };
        model.reset(spawn_pos);
        model
    }

    fn accelerate(&mut self, dt: f32) {
        let pitch = self.state.forward().y.clamp(-1.0, 1.0).asin();
        let angle_from_down = (pitch + FRAC_PI_2).clamp(0.0, PI);

        let slope_gain = if angle_from_down <= FRAC_PI_4 {
            0.0005
        } else if angle_from_down <= 3.0 * FRAC_PI_4 {
            0.0001
        } else {
            -0.0001
        };
        let drag_gain = 0.0005 - 0.0002 * angle_from_down / FRAC_PI_4;

        self.speed = (self.speed + (slope_gain + drag_gain) * LOOP_HZ * dt).clamp(MIN_SPEED, MAX_SPEED);
    }

    fn level_nose(&mut self, dt: f32) {
        let pitch = self.state.forward().y.clamp(-1.0, 1.0).asin();
        if pitch >= LEVEL_PITCH {
            return;
        }

        let step = (LEVEL_RATE * dt).min(LEVEL_PITCH - pitch);
        let lift_axis = self.state.forward().cross(Vec3::Y);
        let axis = if lift_axis.length_squared() > 1e-6 {
            lift_axis.normalize()
        } else {
            self.state.right()
        };
        self.state.rot = (Quat::from_axis_angle(axis, step) * self.state.rot).normalize();
    }
}

impl FlightModel for ClassicModel {
    fn reset(&mut self, spawn_pos: Vec3) {
        self.speed = INITIAL_SPEED;
        self.state = BirdState {
            vel: vec3(0.0, 0.0, -(self.speed * SPEED_UNIT)),
            ..BirdState::at_rest(spawn_pos)
        };
    }

    fn state(&self) -> &BirdState {
        &self.state
    }

    fn update(&mut self, input: &GameInputs, world: &World, dt: f32) -> &BirdState {
        let dt = dt.clamp(0.0, MAX_STEP);

        let pitch = -input.pitch * CONTROL_RATE * dt;
        let roll = -input.roll * CONTROL_RATE * dt;
        let turn = Quat::from_euler(EulerRot::XYZ, pitch, 0.0, roll);
        self.state.rot = (self.state.rot * turn).normalize();

        self.state.vel = self.state.forward() * (self.speed * SPEED_UNIT);
        self.state.pos += self.state.vel * dt;

        let water = world.water_level();
        let ceiling = world.shallow_ceiling();

        if self.state.pos.y < ceiling {
            if self.state.pos.y < water {
                self.state.pos.y = water;
                self.level_nose(dt);
            }
            self.speed = WATER_SPEED;
            self.state.catch_armed = true;
        } else {
            self.state.catch_armed = false;
            self.accelerate(dt);
        }

        &self.state
    }
}
