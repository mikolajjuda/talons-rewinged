mod mode;
mod rules;

use crate::flight::{BirdState, FlightModel};
use crate::input::GameInputs;
use crate::world::{MAP_SCALE, World};
use macroquad::prelude::{Vec3, vec3};

pub use mode::Mode;

pub const TARGET_FISH: u32 = 10;
pub const NO_BEST_TIME: f32 = 9999.0;
const SPAWN_CLEARANCE_RAW: f32 = 64.0 * MAP_SCALE as f32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Playing,
    Crashed,
    Won,
}

#[derive(Default)]
pub struct Events {
    pub fish_caught: bool,
    pub crashed: bool,
    pub won: bool,
}

pub struct Game {
    mode: Mode,
    world: World,
    model: Box<dyn FlightModel>,
    bird: BirdState,
    claws_down: f32,
    phase: Phase,
    score: u32,
    elapsed: f32,
    best_time: f32,
}

impl Game {
    pub fn new(mode: Mode, seed: u64, best_time: f32) -> Self {
        let world = World::new(seed);
        let spawn = Self::spawn_point(&world);
        let model = mode.flight_model(spawn);
        let bird = *model.state();

        Self {
            mode,
            world,
            model,
            bird,
            claws_down: 0.0,
            phase: Phase::Playing,
            score: 0,
            elapsed: 0.0,
            best_time,
        }
    }

    fn spawn_point(world: &World) -> Vec3 {
        let ground = world.height_at(0.0, 0.0);
        let clearance = world.height_from_raw(SPAWN_CLEARANCE_RAW);
        vec3(0.0, ground + clearance, 0.0)
    }

    pub fn restart(&mut self) {
        self.world.regenerate();
        let spawn = Self::spawn_point(&self.world);
        self.model.reset(spawn);
        self.bird = *self.model.state();
        self.claws_down = 0.0;
        self.phase = Phase::Playing;
        self.score = 0;
        self.elapsed = 0.0;
    }

    pub fn update(&mut self, input: &GameInputs, dt: f32) -> Events {
        let mut events = Events::default();
        if self.phase != Phase::Playing {
            return events;
        }

        self.elapsed += dt.max(0.0);
        self.bird = *self.model.update(input, &self.world, dt);
        self.world.update(dt);

        let outcome = rules::resolve_fishing(&self.bird, &self.world);
        self.claws_down = outcome.claws_down;

        if let Some(index) = outcome.caught {
            self.world.fishes[index].caught = true;
            self.score += 1;
            events.fish_caught = true;

            if self.score >= TARGET_FISH {
                self.phase = Phase::Won;
                self.best_time = self.best_time.min(self.elapsed);
                events.won = true;
                return events;
            }
        }

        if rules::has_crashed(&self.bird, &self.world, self.mode.crash_margin()) {
            self.phase = Phase::Crashed;
            events.crashed = true;
        }

        events
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn bird(&self) -> &BirdState {
        &self.bird
    }

    pub fn claws_down(&self) -> f32 {
        self.claws_down
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    pub fn best_time(&self) -> f32 {
        self.best_time
    }

    pub fn fish_remaining(&self) -> u32 {
        TARGET_FISH.saturating_sub(self.score)
    }
}
