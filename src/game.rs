use crate::flight::{BirdState, ClassicModel, FlightModel, RealisticModel};
use crate::input::GameInputs;
use crate::world::{MAP_SCALE, WATER_ELEVATION, World};
use macroquad::prelude::*;

const SPAWN_CLEARANCE: f32 = 40.0;
const CATCH_RADIUS: f32 = 8.0;

pub struct GameState {
    pub world: World,
    pub bird: BirdState,
    pub flight_model: Box<dyn FlightModel>,
    pub alive: bool,
    pub score: u32,
}

impl GameState {
    fn spawn_pos(world: &World) -> Vec3 {
        let x = 0.0;
        let z = 0.0;
        let ground = world.get_height(x, z);
        vec3(x, ground + SPAWN_CLEARANCE, z)
    }

    pub fn new_classic() -> Self {
        let world = World::new(1337);
        let spawn = Self::spawn_pos(&world);
        let mut model = Box::new(ClassicModel::new(spawn));
        model.reset(spawn);
        let bird = *model.state();

        Self {
            world,
            bird,
            flight_model: model,
            alive: true,
            score: 0,
        }
    }

    pub fn new_physics() -> Self {
        let world = World::new(1337);
        let spawn = Self::spawn_pos(&world);
        let mut model = Box::new(RealisticModel::new(spawn));
        model.reset(spawn);
        let bird = *model.state();

        Self {
            world,
            bird,
            flight_model: model,
            alive: true,
            score: 0,
        }
    }

    pub fn reset(&mut self) {
        self.world.regenerate();
        let spawn = Self::spawn_pos(&self.world);
        self.flight_model.reset(spawn);
        self.bird = *self.flight_model.state();
        self.alive = true;
        self.score = 0;
    }

    pub fn update(&mut self, input: &GameInputs, dt: f32) {
        if !self.alive {
            return;
        }

        self.bird = *self.flight_model.update(input, &self.world, dt);

        let ground = self.world.get_height(self.bird.pos.x, self.bird.pos.z);
        let water_h = (WATER_ELEVATION as f32) / (MAP_SCALE as f32);
        if self.bird.pos.y <= ground + 0.2 && ground > water_h {
            self.alive = false;
        }

        if self.bird.catch_armed {
            for fish in &mut self.world.fishes {
                if !fish.caught && self.bird.pos.distance(fish.pos) < CATCH_RADIUS {
                    fish.caught = true;
                    self.score += 1;
                }
            }
        }
    }
}
