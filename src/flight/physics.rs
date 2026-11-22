use super::{BirdState, FlightModel};
use crate::input::GameInputs;
use crate::world::World;
use macroquad::prelude::*;
use std::f32::consts::FRAC_PI_8;

pub struct PhysicsModel {
    state: BirdState,
    angular_vel: Vec3,
    deflections: Vec3,
}

impl PhysicsModel {
    pub fn new(spawn_pos: Vec3) -> Self {
        let mut model = Self {
            state: BirdState {
                pos: spawn_pos,
                vel: Vec3::ZERO,
                rot: Quat::IDENTITY,
                catch_armed: false,
            },
            angular_vel: Vec3::ZERO,
            deflections: Vec3::ZERO,
        };
        model.reset(spawn_pos);
        model
    }
}

impl FlightModel for PhysicsModel {
    fn reset(&mut self, spawn_pos: Vec3) {
        let initial_speed = 24.0;
        let initial_pitch = -0.06;
        let rot = Quat::from_rotation_x(initial_pitch);
        let vel = rot * vec3(0.0, 0.0, -initial_speed);

        self.state = BirdState {
            pos: spawn_pos,
            vel,
            rot,
            catch_armed: false,
        };
        self.angular_vel = Vec3::ZERO;
        self.deflections = Vec3::ZERO;
    }

    fn state(&self) -> &BirdState {
        &self.state
    }

    fn update(&mut self, input: &GameInputs, world: &World, dt: f32) -> &BirdState {
        let dt = dt.clamp(0.0, 0.05);
        if dt <= 0.0 {
            return &self.state;
        }

        let mass = 4.0;
        let gravity = 9.81;

        let forward = self.state.forward();
        let up = self.state.up();
        let right = self.state.right();

        let airspeed = self.state.vel.length();
        let air_dir = if airspeed > 0.1 {
            self.state.vel / airspeed
        } else {
            forward
        };

        let v_fwd = self.state.vel.dot(forward);
        let v_up = self.state.vel.dot(up);
        let v_right = self.state.vel.dot(right);

        let alpha = (-v_up).atan2(v_fwd.max(0.5));
        let beta = v_right.atan2(v_fwd.max(0.5));

        let dive_pitch = (-forward.y).clamp(0.0, 1.0);
        let water_h = world.water_level();
        let is_on_water = self.state.pos.y <= water_h + 0.15;

        let q_eff = (airspeed / 24.0).clamp(0.25, 2.5);

        let mut cmd_pitch = -input.pitch;
        let cmd_roll = -input.roll;
        let cmd_yaw = -input.yaw;

        let alpha_stall = 0.28;
        if cmd_pitch > 0.0 {
            let stall_margin = ((alpha_stall - alpha) / 0.08).clamp(0.0, 1.0);
            cmd_pitch *= stall_margin;
        }

        let alpha_protect_torque = if alpha > alpha_stall {
            -(alpha - alpha_stall) * 28.0 * q_eff
        } else {
            0.0
        };

        let target_deflection = vec3(cmd_pitch, cmd_yaw, cmd_roll);
        self.deflections += (target_deflection - self.deflections) * (14.0 * dt).min(1.0);

        let roll_aoa_eff = (1.0 - ((alpha.abs() - 0.16).max(0.0) / 0.14)).clamp(0.25, 1.0);

        let pitch_torque = self.deflections.x * 16.0 * q_eff;
        let roll_torque = self.deflections.z * 18.0 * q_eff * roll_aoa_eff;
        let yaw_torque = self.deflections.y * 3.2 * q_eff;

        let damp_pitch = -self.angular_vel.x * 6.0;
        let damp_roll = -self.angular_vel.z * (6.5 + 2.5 * q_eff);
        let damp_yaw = -self.angular_vel.y * 8.5;

        let trim_glide_alpha = if input.pitch.abs() < 0.05 { -0.02 } else { 0.0 };
        let stab_pitch = -(alpha - trim_glide_alpha) * 10.0 * q_eff;
        let stab_yaw = -beta * 10.0 * q_eff;

        let turn_coordination = right.y * 1.8 * q_eff;

        self.angular_vel.x += (pitch_torque + damp_pitch + stab_pitch + alpha_protect_torque) * dt;
        self.angular_vel.y += (yaw_torque + damp_yaw + stab_yaw + turn_coordination) * dt;
        self.angular_vel.z += (roll_torque + damp_roll) * dt;

        self.angular_vel = self
            .angular_vel
            .clamp(vec3(-4.5, -1.2, -6.5), vec3(4.5, 1.2, 6.5));

        let d_rot = Quat::from_euler(
            EulerRot::XYZ,
            self.angular_vel.x * dt,
            self.angular_vel.y * dt,
            self.angular_vel.z * dt,
        );
        self.state.rot = (self.state.rot * d_rot).normalize();

        let is_diving = forward.y < -0.15;
        let is_braking = input.switch;

        let wing_area = if is_braking {
            0.95
        } else if is_diving {
            lerp(0.85, 0.18, dive_pitch)
        } else {
            0.85
        };

        let cd0 = if is_braking {
            0.15
        } else if is_diving {
            lerp(0.024, 0.006, dive_pitch)
        } else {
            0.024
        };

        let cl_base = (0.24 + 4.2 * alpha).clamp(-0.4, 1.5);
        let cl = if is_braking { cl_base * 1.35 } else { cl_base };
        let cd = cd0 + (cl * cl) / 18.0;

        let q_dyn = 0.5 * 1.2 * airspeed * airspeed * wing_area;
        let lift_dir = right.cross(air_dir).normalize_or_zero();

        let lift_force = lift_dir * (q_dyn * cl);
        let drag_force = -air_dir * (q_dyn * cd);

        let flap_thrust = if is_braking {
            0.0
        } else {
            input.throttle.clamp(0.0, 1.0) * if is_on_water { 65.0 } else { 45.0 }
        };
        let thrust_force = forward * flap_thrust;
        let gravity_force = vec3(0.0, -mass * gravity, 0.0);

        let water_takeoff_lift = if is_on_water && input.throttle > 0.1 && !is_braking {
            vec3(0.0, 48.0 * input.throttle, 0.0)
        } else {
            Vec3::ZERO
        };

        let dive_boost = if is_diving && !is_braking {
            forward * (dive_pitch * 25.0 * mass)
        } else {
            Vec3::ZERO
        };

        let total_force = lift_force
            + drag_force
            + thrust_force
            + gravity_force
            + water_takeoff_lift
            + dive_boost;
        self.state.vel += (total_force / mass) * dt;
        self.state.pos += self.state.vel * dt;

        if self.state.pos.y < water_h {
            self.state.pos.y = water_h;
            if self.state.vel.y < 0.0 {
                self.state.vel.y = 0.0;
            }

            let pitch = self.state.forward().y.clamp(-1.0, 1.0).asin();
            if pitch < -FRAC_PI_8 {
                let pull_step = (-FRAC_PI_8 - pitch).min(dt * 2.5);
                let level_rot = Quat::from_rotation_x(pull_step);
                self.state.rot = (self.state.rot * level_rot).normalize();
                self.angular_vel.x = 0.0;
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

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}
