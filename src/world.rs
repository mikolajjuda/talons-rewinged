mod elevation;
mod panels;
mod rng;

use macroquad::prelude::{Vec3, vec3};
use panels::{Prop, PropKind};
use rng::Rng;
use std::f32::consts::TAU;
use std::sync::atomic::{AtomicU64, Ordering};

pub use panels::Panel;

pub const MAP_WIDTH: usize = 1024;
pub const MAP_HEIGHT: usize = 1024;

pub const MAP_SCALE: i16 = 150;
pub const WATER_ELEVATION: i16 = 15 * MAP_SCALE;
pub const ROCK_ELEVATION: i16 = 45 * MAP_SCALE;
pub const SNOW_ELEVATION: i16 = 55 * MAP_SCALE;

pub const WORLD_SPACING: f32 = 1.5;
pub const HORIZONTAL_UNIT: f32 = WORLD_SPACING / MAP_SCALE as f32;
pub const HEIGHT_UNIT: f32 = HORIZONTAL_UNIT;
pub const SHALLOW_BAND_RAW: f32 = 3.0 * MAP_SCALE as f32;

const NUM_BIRDS: usize = 256;
const BIRD_ELEVATION_RAW: f32 = 10.0 * MAP_SCALE as f32;
const BIRD_SPEED: f32 = 10.0 * WORLD_SPACING;
const BIRD_TURN_RATE: f32 = TAU / 10.0;
const BOAT_LIFT_RAW: f32 = 2.0 * MAP_SCALE as f32;
const FISH_LIFT: f32 = 0.25;
const SEED_STEP: u64 = 1013904223;

static NEXT_REVISION: AtomicU64 = AtomicU64::new(1);

pub struct Fish {
    pub pos: Vec3,
    pub caught: bool,
}

#[derive(Clone, Copy)]
pub struct Boat {
    pub pos: Vec3,
    pub sprite: u32,
}

#[derive(Clone, Copy)]
pub struct Tree {
    pub pos: Vec3,
}

#[derive(Clone, Copy)]
pub struct Bird {
    pub pos: Vec3,
    pub angle: f32,
}

pub struct World {
    seed: u64,
    revision: u64,
    elevations: Vec<i16>,
    panels: Vec<Panel>,
    pub fishes: Vec<Fish>,
    pub boats: Vec<Boat>,
    pub trees: Vec<Tree>,
    pub birds: Vec<Bird>,
}

impl World {
    pub fn new(seed: u64) -> Self {
        let mut world = Self {
            seed,
            revision: 0,
            elevations: vec![0; MAP_WIDTH * MAP_HEIGHT],
            panels: Vec::new(),
            fishes: Vec::new(),
            boats: Vec::new(),
            trees: Vec::new(),
            birds: Vec::new(),
        };
        world.generate();
        world
    }

    pub fn regenerate(&mut self) {
        self.seed = self.seed.wrapping_add(SEED_STEP);
        self.generate();
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn panels(&self) -> &[Panel] {
        &self.panels
    }

    pub fn half_extent() -> f32 {
        MAP_WIDTH as f32 * 0.5 * WORLD_SPACING
    }

    pub fn grid_to_world(gx: f32, gy: f32) -> (f32, f32) {
        let half = Self::half_extent();
        (gx * WORLD_SPACING - half, gy * WORLD_SPACING - half)
    }

    fn world_to_grid(x: f32, z: f32) -> (f32, f32) {
        let half = Self::half_extent();
        ((x + half) / WORLD_SPACING, (z + half) / WORLD_SPACING)
    }

    pub fn height_from_raw(&self, raw: f32) -> f32 {
        raw * HEIGHT_UNIT
    }

    pub fn raw_from_height(&self, height: f32) -> f32 {
        height / HEIGHT_UNIT
    }

    pub fn shallow_ceiling(&self) -> f32 {
        self.water_level() + self.height_from_raw(SHALLOW_BAND_RAW)
    }

    pub fn water_level(&self) -> f32 {
        self.height_from_raw(WATER_ELEVATION as f32)
    }

    pub fn raw_elevation(&self, x: usize, y: usize) -> i16 {
        if x >= MAP_WIDTH || y >= MAP_HEIGHT {
            WATER_ELEVATION
        } else {
            self.elevations[y * MAP_WIDTH + x]
        }
    }

    pub fn is_inside(&self, x: f32, z: f32) -> bool {
        let (gx, gy) = Self::world_to_grid(x, z);
        gx >= 0.0 && gy >= 0.0 && gx < MAP_WIDTH as f32 && gy < MAP_HEIGHT as f32
    }

    pub fn height_at(&self, x: f32, z: f32) -> f32 {
        let (gx, gy) = Self::world_to_grid(x, z);
        let cx = gx.clamp(0.0, (MAP_WIDTH - 1) as f32) as usize;
        let cy = gy.clamp(0.0, (MAP_HEIGHT - 1) as f32) as usize;
        self.height_from_raw(self.raw_elevation(cx, cy) as f32)
    }

    pub fn update(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        let cruise = self.height_from_raw(BIRD_ELEVATION_RAW);
        let mut birds = std::mem::take(&mut self.birds);

        for bird in &mut birds {
            bird.pos.x += BIRD_SPEED * dt * bird.angle.cos();
            bird.pos.z += BIRD_SPEED * dt * bird.angle.sin();
            bird.angle = (bird.angle + BIRD_TURN_RATE * dt).rem_euclid(TAU);
            bird.pos.y = cruise + self.height_at(bird.pos.x, bird.pos.z);
        }

        self.birds = birds;
    }

    fn generate(&mut self) {
        self.revision = NEXT_REVISION.fetch_add(1, Ordering::Relaxed);

        let mut rng = Rng::new(self.seed);
        elevation::generate(&mut rng, &mut self.elevations);

        let layout = panels::build(&self.elevations, &mut rng);
        self.panels = layout.panels;
        self.populate(&layout.props);
        self.spawn_birds(&mut rng);
    }

    fn populate(&mut self, props: &[Prop]) {
        self.fishes.clear();
        self.boats.clear();
        self.trees.clear();

        let water = self.water_level();
        let boat_height = water + self.height_from_raw(BOAT_LIFT_RAW);

        for prop in props {
            let (x, z) = Self::grid_to_world(prop.cell_x, prop.cell_y);
            match prop.kind {
                PropKind::Fish => self.fishes.push(Fish {
                    pos: vec3(x, water + FISH_LIFT, z),
                    caught: false,
                }),
                PropKind::Boat(sprite) => self.boats.push(Boat {
                    pos: vec3(x, boat_height, z),
                    sprite,
                }),
                PropKind::Tree => {
                    let ground = self.height_at(x, z);
                    self.trees.push(Tree {
                        pos: vec3(x, ground, z),
                    });
                }
            }
        }
    }

    fn spawn_birds(&mut self, rng: &mut Rng) {
        self.birds.clear();
        self.birds.reserve(NUM_BIRDS);

        let cruise = self.height_from_raw(BIRD_ELEVATION_RAW);

        for _ in 0..NUM_BIRDS {
            let gx = rng.next_f32() * MAP_WIDTH as f32;
            let gy = rng.next_f32() * MAP_HEIGHT as f32;
            let (x, z) = Self::grid_to_world(gx, gy);

            self.birds.push(Bird {
                pos: vec3(x, cruise + self.height_at(x, z), z),
                angle: rng.next_f32() * TAU,
            });
        }
    }
}
