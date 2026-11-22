use crate::flight::BirdState;
use crate::world::{HORIZONTAL_UNIT, MAP_SCALE, World};
use macroquad::prelude::Vec3;

const CATCH_RADIUS: f32 = 2.0 * MAP_SCALE as f32 * HORIZONTAL_UNIT;
const CLAW_RANGE: f32 = 4.0 * MAP_SCALE as f32 * HORIZONTAL_UNIT;
const CLAW_FLOOR: f32 = 0.05;
const CLAW_IDLE: f32 = 0.01;

pub struct FishingOutcome {
    pub claws_down: f32,
    pub caught: Option<usize>,
}

impl FishingOutcome {
    const NONE: FishingOutcome = FishingOutcome {
        claws_down: 0.0,
        caught: None,
    };
}

pub fn resolve_fishing(bird: &BirdState, world: &World) -> FishingOutcome {
    if !bird.catch_armed || !bird.is_upright() {
        return FishingOutcome::NONE;
    }

    let Some((index, distance)) = nearest_fish(world, bird.pos) else {
        return FishingOutcome::NONE;
    };

    let claws_down = if distance < CLAW_RANGE {
        (1.0 - distance / CLAW_RANGE).clamp(CLAW_FLOOR, 1.0)
    } else {
        CLAW_IDLE
    };

    FishingOutcome {
        claws_down,
        caught: (distance < CATCH_RADIUS).then_some(index),
    }
}

pub fn has_crashed(bird: &BirdState, world: &World, margin: f32) -> bool {
    if !world.is_inside(bird.pos.x, bird.pos.z) {
        return true;
    }
    let ground = world.height_at(bird.pos.x, bird.pos.z);
    ground > world.water_level() && bird.pos.y < ground + margin
}

fn nearest_fish(world: &World, from: Vec3) -> Option<(usize, f32)> {
    world
        .fishes
        .iter()
        .enumerate()
        .filter(|(_, fish)| !fish.caught)
        .map(|(index, fish)| {
            let dx = fish.pos.x - from.x;
            let dz = fish.pos.z - from.z;
            (index, dx * dx + dz * dz)
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(index, squared)| (index, squared.sqrt()))
}
