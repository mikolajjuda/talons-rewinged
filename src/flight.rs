use crate::input::GameInputs;
use crate::world::{MAP_SCALE, WATER_ELEVATION, World};
use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_8, PI};

const CLAW_GRAB_RANGE: f32 = 18.0;
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

// -----------------------------------------------------------------------------
// Classic Model (Exact replication of TempleOS Talons.HC behavior)
// -----------------------------------------------------------------------------

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

        // In classic, locks speed to 1.0 (10.0 m/s), arms catch, phases through water
        if self.state.pos.y < water_h + 3.0 {
            self.temple_speed = 1.0;
            self.state.catch_armed = true;
        } else {
            self.state.catch_armed = false;

            let pitch = self.state.forward().y.clamp(-1.0, 1.0).asin();
            let abs_theta = (pitch + FRAC_PI_2).clamp(0.0, PI);

            let accel_a = if abs_theta <= std::f32::consts::FRAC_PI_4 {
                0.0005
            } else if abs_theta <= 3.0 * std::f32::consts::FRAC_PI_4 {
                0.0001
            } else {
                -0.0001
            };
            let accel_b = 0.0005 - 0.0002 * abs_theta / std::f32::consts::FRAC_PI_4;

            self.temple_speed =
                (self.temple_speed + (accel_a + accel_b) * TEMPLE_REFRESH_HZ * dt).clamp(0.1, 5.0);
        }

        // Only clamp ground over land (elev > water_h). Over water, phase freely through.
        let ground = world.get_height(self.state.pos.x, self.state.pos.z);
        if ground > water_h && self.state.pos.y < ground {
            self.state.pos.y = ground;
        }

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
        (1.0 - (dist / CLAW_GRAB_RANGE)).clamp(0.05, 1.0)
    } else {
        0.01
    }
}

// -----------------------------------------------------------------------------
// Bird Physics Model (Aerodynamic Flight with Surface Constraint & Classic Rules)
// -----------------------------------------------------------------------------

pub struct PhysicsModel {
    state: BirdState,
    angular_vel: Vec3, // Body angular rates in rad/s: x = pitch, y = yaw, z = roll
    deflections: Vec3, // Control surfaces [-1.0, 1.0]: x = elevator, y = rudder, z = ailerons
}

pub type BirdPhysicsModel = PhysicsModel;

impl PhysicsModel {
    pub fn new(spawn_pos: Vec3) -> Self {
        let mut model = Self {
            state: BirdState {
                pos: spawn_pos,
                vel: Vec3::ZERO,
                rot: Quat::IDENTITY,
                catch_armed: false,
                claws_down: 0.0,
            },
            angular_vel: Vec3::ZERO,
            deflections: Vec3::ZERO,
        };
        model.reset(spawn_pos);
        model
    }
}

impl FlightModel for PhysicsModel {
    fn name(&self) -> &'static str {
        "Bird Physics (Aerodynamic)"
    }

    fn reset(&mut self, spawn_pos: Vec3) {
        let initial_speed = 24.0;
        let initial_pitch = -0.06; // Gentle natural descent angle (~3.5 deg)
        let rot = Quat::from_rotation_x(initial_pitch);
        let vel = rot * vec3(0.0, 0.0, -initial_speed);

        self.state = BirdState {
            pos: spawn_pos,
            vel,
            rot,
            catch_armed: false,
            claws_down: 0.0,
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

        // Project velocity into body coordinates
        let v_fwd = self.state.vel.dot(forward);
        let v_up = self.state.vel.dot(up);
        let v_right = self.state.vel.dot(right);

        let alpha = (-v_up).atan2(v_fwd.max(0.5)); // Angle of attack
        let beta = v_right.atan2(v_fwd.max(0.5)); // Sideslip angle

        let dive_pitch = (-forward.y).clamp(0.0, 1.0);
        let water_h = (WATER_ELEVATION as f32) / (MAP_SCALE as f32);
        let is_on_water = self.state.pos.y <= water_h + 0.15;

        let q_eff = (airspeed / 24.0).clamp(0.25, 2.5);

        // -------------------------------------------------------------
        // Control Surfaces & Fly-By-Wire Stall Protection
        // -------------------------------------------------------------
        let mut cmd_pitch = -input.pitch; // W = pitch down, S = pitch up
        let cmd_roll = -input.roll; // A = roll left, D = roll right
        let cmd_yaw = -input.yaw; // Q = yaw left, E = yaw right

        // FBW Stall Protection: Limit pitch-up command as alpha approaches critical stall (~16 deg)
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

        // Aileron authority softens at high AoA
        let roll_aoa_eff = (1.0 - ((alpha.abs() - 0.16).max(0.0) / 0.14)).clamp(0.25, 1.0);

        // Aerodynamic moments
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

        // -------------------------------------------------------------
        // Aerodynamic Forces & Speed Dynamics
        // -------------------------------------------------------------
        let is_diving = forward.y < -0.15;
        let is_braking = input.switch;

        // Wing morphing
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

        // Flapping propulsion (Shift held)
        let flap_thrust = if is_braking {
            0.0
        } else {
            input.throttle.clamp(0.0, 1.0) * if is_on_water { 65.0 } else { 45.0 }
        };
        let thrust_force = forward * flap_thrust;
        let gravity_force = vec3(0.0, -mass * gravity, 0.0);

        // Takeoff lift assist when flapping while skimming the water surface
        let water_takeoff_lift = if is_on_water && input.throttle > 0.1 && !is_braking {
            vec3(0.0, 48.0 * input.throttle, 0.0)
        } else {
            Vec3::ZERO
        };

        // Dive impulse so speed surges to 60-80+ m/s
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

        // -------------------------------------------------------------
        // Water Surface Constraint (Cannot Go Below Water)
        // -------------------------------------------------------------
        if self.state.pos.y < water_h {
            self.state.pos.y = water_h; // Hard surface boundary
            if self.state.vel.y < 0.0 {
                self.state.vel.y = 0.0; // Stop downward velocity
            }

            // Gently level nose if pitched down into the water
            let pitch = self.state.forward().y.clamp(-1.0, 1.0).asin();
            if pitch < -FRAC_PI_8 {
                let pull_step = (-FRAC_PI_8 - pitch).min(dt * 2.5);
                let level_rot = Quat::from_rotation_x(pull_step);
                self.state.rot = (self.state.rot * level_rot).normalize();
                self.angular_vel.x = 0.0;
            }
        }

        // -------------------------------------------------------------
        // Land & Fish Catching Rules (Same Simple Rules as Classic)
        // -------------------------------------------------------------
        // Only clamp land (elev > water_h). Crashes on land handled in game.rs
        let ground = world.get_height(self.state.pos.x, self.state.pos.z);
        if ground > water_h && self.state.pos.y < ground {
            self.state.pos.y = ground;
        }

        // Catch is armed whenever near the water surface
        if self.state.pos.y < water_h + 3.0 {
            self.state.catch_armed = true;
        } else {
            self.state.catch_armed = false;
        }

        self.state.claws_down = compute_claws_down(&self.state, world);

        &self.state
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

// -----------------------------------------------------------------------------
// Drone Acro Model (FPV Quadcopter Dynamics & Kinematics)
// -----------------------------------------------------------------------------

pub struct DroneAcroModel {
    state: BirdState,
    angular_vel: Vec3, // Current angular velocity (pitch, yaw, roll rates in rad/s)
}

impl DroneAcroModel {
    pub fn new(spawn_pos: Vec3) -> Self {
        let mut model = Self {
            state: BirdState {
                pos: spawn_pos,
                vel: Vec3::ZERO,
                rot: Quat::IDENTITY,
                catch_armed: false,
                claws_down: 0.0,
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

impl FlightModel for DroneAcroModel {
    fn name(&self) -> &'static str {
        "Drone (Acro Mode)"
    }

    fn reset(&mut self, spawn_pos: Vec3) {
        self.state = BirdState {
            pos: spawn_pos,
            vel: Vec3::ZERO,
            rot: Quat::IDENTITY,
            catch_armed: false,
            claws_down: 0.0,
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

        // -------------------------------------------------------------
        // Acro Mode Kinematics (Betaflight-style Rate Curve)
        // -------------------------------------------------------------
        let target_pitch_rate = acro_rate(-input.pitch, 3.0, 11.5); // ~170 to ~660 deg/s
        let target_roll_rate = acro_rate(-input.roll, 3.0, 11.5); // ~170 to ~660 deg/s
        let target_yaw_rate = acro_rate(-input.yaw, 2.5, 9.0); // ~145 to ~515 deg/s

        // Crisp PID rate tracking (Air Mode: full authority even at 0% throttle)
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

        // -------------------------------------------------------------
        // Drone Dynamics (Scaled for World Units, 4.5:1 TWR)
        // -------------------------------------------------------------
        let mass = 0.8;
        let gravity = 28.0; // Scaled to world units (mountains are ~40-55 units tall)

        // 1:1 throttle (0.0..1.0) along the quad's top/normal axis
        let throttle = input.throttle.clamp(0.0, 1.0);
        let max_thrust_n = mass * gravity * 4.5; // 4.5:1 TWR -> hover at ~22%, massive punchouts
        let thrust_force = self.state.up() * (throttle * max_thrust_n);

        let gravity_force = vec3(0.0, -mass * gravity, 0.0);

        // Scaled body drag -> full-throttle top speed ~115-120 units/s
        let speed = self.state.vel.length();
        let drag_force = -self.state.vel * (mass * (0.25 + 0.008 * speed));

        let total_force = thrust_force + gravity_force + drag_force;
        self.state.vel += (total_force / mass) * dt;
        self.state.pos += self.state.vel * dt;

        // -------------------------------------------------------------
        // Water & Land Constraints (Classic Fish Catching Rules)
        // -------------------------------------------------------------
        let water_h = (WATER_ELEVATION as f32) / (MAP_SCALE as f32);

        if self.state.pos.y < water_h {
            self.state.pos.y = water_h;
            if self.state.vel.y < 0.0 {
                self.state.vel.y = 0.0;
            }
        }

        let ground = world.get_height(self.state.pos.x, self.state.pos.z);
        if ground > water_h && self.state.pos.y < ground {
            self.state.pos.y = ground;
        }

        if self.state.pos.y < water_h + 3.0 {
            self.state.catch_armed = true;
        } else {
            self.state.catch_armed = false;
        }

        self.state.claws_down = compute_claws_down(&self.state, world);

        &self.state
    }
}
