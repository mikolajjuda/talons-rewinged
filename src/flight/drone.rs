use super::{BirdState, FlightModel};
use crate::input::GameInputs;
use crate::world::World;
use macroquad::prelude::*;

pub struct DroneModel {
    state: BirdState,
    angular_vel: Vec3,
}

impl DroneModel {
    pub fn new(spawn_pos: Vec3) -> Self {
        let mut model = Self {
            state: BirdState {
                pos: spawn_pos,
                vel: Vec3::ZERO,
                rot: Quat::IDENTITY,
                catch_armed: false,
            },
            angular_vel: Vec3::ZERO,
        };
        model.reset(spawn_pos);
        model
    }
}

fn acro_rate(input: f32, center_rate: f32, max_rate: f32) -> f32 {
    let x = input.clamp(-1.0, 1.0);
    let expo = x * x * x;
    x * center_rate + expo * (max_rate - center_rate)
}

impl FlightModel for DroneModel {
    fn reset(&mut self, spawn_pos: Vec3) {
        self.state = BirdState {
            pos: spawn_pos,
            vel: Vec3::ZERO,
            rot: Quat::IDENTITY,
            catch_armed: false,
        };
        self.angular_vel = Vec3::ZERO;
    }

    fn state(&self) -> &BirdState {
        &self.state
    }

    fn update(&mut self, input: &GameInputs, world: &World, dt: f32) -> &BirdState {
        let dt = dt.clamp(0.0, 0.05);
        if dt <= 0.0 {
            return &self.state;
        }

        let target_pitch_rate = acro_rate(-input.pitch, 3.0, 11.5);
        let target_roll_rate = acro_rate(-input.roll, 3.0, 11.5);
        let target_yaw_rate = acro_rate(-input.yaw, 2.5, 9.0);

        let rate_response = 30.0;
        let blend = (rate_response * dt).min(1.0);
        self.angular_vel.x += (target_pitch_rate - self.angular_vel.x) * blend;
        self.angular_vel.y += (target_yaw_rate - self.angular_vel.y) * blend;
        self.angular_vel.z += (target_roll_rate - self.angular_vel.z) * blend;

        let d_rot = Quat::from_euler(
            EulerRot::XYZ,
            self.angular_vel.x * dt,
            self.angular_vel.y * dt,
            self.angular_vel.z * dt,
        );
        self.state.rot = (self.state.rot * d_rot).normalize();

        let mass = 0.8;
        let gravity = 28.0;

        let throttle = input.throttle.clamp(0.0, 1.0);
        let max_thrust_n = mass * gravity * 4.5;
        let thrust_force = self.state.up() * (throttle * max_thrust_n);

        let gravity_force = vec3(0.0, -mass * gravity, 0.0);

        let speed = self.state.vel.length();
        let drag_force = -self.state.vel * (mass * (0.25 + 0.008 * speed));

        let total_force = thrust_force + gravity_force + drag_force;
        self.state.vel += (total_force / mass) * dt;
        self.state.pos += self.state.vel * dt;

        let water_h = world.water_level();

        if self.state.pos.y < water_h {
            self.state.pos.y = water_h;
            if self.state.vel.y < 0.0 {
                self.state.vel.y = 0.0;
            }
        }

        let ground = world.height_at(self.state.pos.x, self.state.pos.z);
        if ground > water_h && self.state.pos.y < ground {
            self.state.pos.y = ground;
        }

        if self.state.pos.y < world.shallow_ceiling() {
            self.state.catch_armed = true;
        } else {
            self.state.catch_armed = false;
        }

        &self.state
    }
}
