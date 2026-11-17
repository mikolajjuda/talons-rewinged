use crate::input::GameInputs;
use crate::world::{MAP_SCALE, WATER_ELEVATION, World};
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct BirdState {
    pub pos: Vec3,
    pub vel: Vec3,
    pub rot: Quat,
    pub catch_armed: bool,
}

impl BirdState {
    pub fn speed(&self) -> f32 {
        self.vel.length()
    }

    pub fn forward(&self) -> Vec3 {
        self.rot * vec3(0.0, 0.0, -1.0)
    }

    pub fn up(&self) -> Vec3 {
        self.rot * Vec3::Y
    }

    pub fn right(&self) -> Vec3 {
        self.rot * Vec3::X
    }
}

pub trait FlightModel {
    fn name(&self) -> &'static str;
    fn reset(&mut self, spawn_pos: Vec3);
    fn update(&mut self, input: &GameInputs, world: &World, dt: f32) -> &BirdState;
    fn state(&self) -> &BirdState;
}

pub struct ClassicModel {
    state: BirdState,
    ctrl_rate: f32,
    speed: f32,
}

impl ClassicModel {
    pub fn new(spawn_pos: Vec3) -> Self {
        let mut model = Self {
            state: BirdState {
                pos: spawn_pos,
                vel: Vec3::ZERO,
                rot: Quat::IDENTITY,
                catch_armed: false,
            },
            ctrl_rate: 2.2,
            speed: 55.0,
        };
        model.reset(spawn_pos);
        model
    }
}

impl FlightModel for ClassicModel {
    fn name(&self) -> &'static str {
        "Classic (Talons.HC)"
    }

    fn reset(&mut self, spawn_pos: Vec3) {
        self.speed = 55.0;
        self.state = BirdState {
            pos: spawn_pos,
            vel: vec3(0.0, 0.0, -self.speed),
            rot: Quat::IDENTITY,
            catch_armed: false,
        };
    }

    fn state(&self) -> &BirdState {
        &self.state
    }

    fn update(&mut self, input: &GameInputs, world: &World, dt: f32) -> &BirdState {
        let dt = dt.clamp(0.0, 0.1);

        let local_pitch = -input.pitch * self.ctrl_rate * dt;
        let local_roll = -input.roll * self.ctrl_rate * dt;

        let d_rot = Quat::from_euler(EulerRot::XYZ, local_pitch, 0.0, local_roll);
        self.state.rot = (self.state.rot * d_rot).normalize();

        let water_h = (WATER_ELEVATION as f32) / (MAP_SCALE as f32);

        if self.state.pos.y <= water_h + 3.0 {
            self.speed = 28.0;
        } else {
            self.speed = 55.0;
        }

        self.state.vel = self.state.forward() * self.speed;
        self.state.pos += self.state.vel * dt;

        let ground = world.get_height(self.state.pos.x, self.state.pos.z);
        if self.state.pos.y < ground {
            self.state.pos.y = ground;
        }

        self.state.catch_armed = self.state.pos.y <= water_h + 4.0;

        &self.state
    }
}

pub struct RealisticModel {
    state: BirdState,
}

impl RealisticModel {
    pub fn new(spawn_pos: Vec3) -> Self {
        Self {
            state: BirdState {
                pos: spawn_pos,
                vel: Vec3::ZERO,
                rot: Quat::IDENTITY,
                catch_armed: false,
            },
        }
    }
}

impl FlightModel for RealisticModel {
    fn name(&self) -> &'static str {
        "Realistic"
    }

    fn reset(&mut self, spawn_pos: Vec3) {
        self.state = BirdState {
            pos: spawn_pos,
            vel: Vec3::ZERO,
            rot: Quat::IDENTITY,
            catch_armed: false,
        };
    }

    fn state(&self) -> &BirdState {
        &self.state
    }

    fn update(&mut self, _input: &GameInputs, _world: &World, _dt: f32) -> &BirdState {
        &self.state
    }
}
