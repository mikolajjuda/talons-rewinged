use crate::input::GameInputs;
use crate::world::{MAP_SCALE, WATER_ELEVATION, World};
use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI};

const CLAW_GRAB_RANGE: f32 = 16.0;
const SPEED_SCALE: f32 = 10.0;
const INITIAL_TEMPLE_SPEED: f32 = 2.5;
const TEMPLE_REFRESH_HZ: f32 = 60.0;

#[derive(Clone, Copy, Debug)]
pub struct BirdState {
    pub pos: Vec3,
    pub vel: Vec3,
    pub rot: Quat,
    pub catch_armed: bool,
    pub claws_down: f32,
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
    temple_speed: f32,
}

impl ClassicModel {
    pub fn new(spawn_pos: Vec3) -> Self {
        let mut model = Self {
            state: BirdState {
                pos: spawn_pos,
                vel: Vec3::ZERO,
                rot: Quat::IDENTITY,
                catch_armed: false,
                claws_down: 0.0,
            },
            ctrl_rate: 2.2,
            temple_speed: INITIAL_TEMPLE_SPEED,
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
        self.temple_speed = INITIAL_TEMPLE_SPEED;
        self.state = BirdState {
            pos: spawn_pos,
            vel: vec3(0.0, 0.0, -(self.temple_speed * SPEED_SCALE)),
            rot: Quat::IDENTITY,
            catch_armed: false,
            claws_down: 0.0,
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

        self.state.vel = self.state.forward() * (self.temple_speed * SPEED_SCALE);
        self.state.pos += self.state.vel * dt;

        let water_h = (WATER_ELEVATION as f32) / (MAP_SCALE as f32);

        if self.state.pos.y < water_h + 3.0 {
            if self.state.pos.y < water_h {
                self.state.pos.y = water_h;

                let pitch = self.state.forward().y.clamp(-1.0, 1.0).asin();
                if pitch < -FRAC_PI_8 {
                    let pull_step = (-FRAC_PI_8 - pitch).min(dt);
                    let pull_dir = if self.state.up().y >= 0.0 { 1.0 } else { -1.0 };
                    let level_rot = Quat::from_rotation_x(pull_dir * pull_step);
                    self.state.rot = (self.state.rot * level_rot).normalize();
                }
            }

            self.temple_speed = 1.0;
            self.state.catch_armed = true;
        } else {
            self.state.catch_armed = false;

            let pitch = self.state.forward().y.clamp(-1.0, 1.0).asin();
            let abs_theta = (pitch + FRAC_PI_2).clamp(0.0, PI);

            let accel_a = if abs_theta <= FRAC_PI_4 {
                0.0005
            } else if abs_theta <= 3.0 * FRAC_PI_4 {
                0.0001
            } else {
                -0.0001
            };
            let accel_b = 0.0005 - 0.0002 * abs_theta / FRAC_PI_4;

            self.temple_speed =
                (self.temple_speed + (accel_a + accel_b) * TEMPLE_REFRESH_HZ * dt).clamp(0.1, 5.0);
        }

        let ground = world.get_height(self.state.pos.x, self.state.pos.z);
        if self.state.pos.y < ground {
            self.state.pos.y = ground;
        }

        self.state.vel = self.state.forward() * (self.temple_speed * SPEED_SCALE);
        self.state.claws_down = compute_claws_down(&self.state, world);

        &self.state
    }
}

fn compute_claws_down(state: &BirdState, world: &World) -> f32 {
    if !state.catch_armed || state.up().y <= 0.0 {
        return 0.0;
    }

    let mut best_dist_sq = f32::MAX;
    let mut best_pos = Vec3::ZERO;
    let mut found = false;

    for fish in &world.fishes {
        if fish.caught {
            continue;
        }
        let dx = fish.pos.x - state.pos.x;
        let dz = fish.pos.z - state.pos.z;
        let dist_sq = dx * dx + dz * dz;
        if dist_sq < best_dist_sq {
            best_dist_sq = dist_sq;
            best_pos = fish.pos;
            found = true;
        }
    }

    if !found {
        return 0.0;
    }

    let dist = best_dist_sq.sqrt();
    let forward = state.forward();
    let ahead =
        forward.x * (best_pos.x - state.pos.x) + forward.z * (best_pos.z - state.pos.z) > 0.0;

    if ahead && dist < CLAW_GRAB_RANGE {
        (dist / CLAW_GRAB_RANGE).clamp(0.01, 1.0)
    } else {
        0.01
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
                claws_down: 0.0,
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
            claws_down: 0.0,
        };
    }

    fn state(&self) -> &BirdState {
        &self.state
    }

    fn update(&mut self, _input: &GameInputs, _world: &World, _dt: f32) -> &BirdState {
        &self.state
    }
}
