mod classic;
mod drone;
mod physics;

use crate::input::GameInputs;
use crate::world::World;
use macroquad::prelude::*;

pub use classic::ClassicModel;
pub use drone::DroneModel;
pub use physics::PhysicsModel;

#[derive(Clone, Copy, Debug)]
pub struct BirdState {
    pub pos: Vec3,
    pub vel: Vec3,
    pub rot: Quat,
    pub catch_armed: bool,
}

impl BirdState {
    pub fn at_rest(pos: Vec3) -> Self {
        Self {
            pos,
            vel: Vec3::ZERO,
            rot: Quat::IDENTITY,
            catch_armed: false,
        }
    }

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

    pub fn is_upright(&self) -> bool {
        self.up().y > 0.0
    }
}

pub trait FlightModel {
    fn reset(&mut self, spawn_pos: Vec3);
    fn update(&mut self, input: &GameInputs, world: &World, dt: f32) -> &BirdState;
    fn state(&self) -> &BirdState;
}
