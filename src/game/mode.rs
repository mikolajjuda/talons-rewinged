use crate::flight::{ClassicModel, DroneModel, FlightModel, PhysicsModel};
use macroquad::prelude::Vec3;

const CLASSIC_CRASH_MARGIN: f32 = 0.0;
const CLAMPED_CRASH_MARGIN: f32 = 0.2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Classic,
    Physics,
    Drone,
}

impl Mode {
    pub const ALL: [Mode; 3] = [Mode::Classic, Mode::Physics, Mode::Drone];

    pub fn key(self) -> &'static str {
        match self {
            Mode::Classic => "classic",
            Mode::Physics => "physics",
            Mode::Drone => "drone",
        }
    }

    pub fn crash_margin(self) -> f32 {
        match self {
            Mode::Classic => CLASSIC_CRASH_MARGIN,
            Mode::Physics | Mode::Drone => CLAMPED_CRASH_MARGIN,
        }
    }

    pub fn flight_model(self, spawn: Vec3) -> Box<dyn FlightModel> {
        match self {
            Mode::Classic => Box::new(ClassicModel::new(spawn)),
            Mode::Physics => Box::new(PhysicsModel::new(spawn)),
            Mode::Drone => Box::new(DroneModel::new(spawn)),
        }
    }
}
