use macroquad::prelude::*;

pub const MAP_WIDTH: usize = 1024;
pub const MAP_HEIGHT: usize = 1024;

pub const MAP_SCALE: i16 = 150;
pub const WATER_ELEVATION: i16 = 15 * MAP_SCALE;
pub const ROCK_ELEVATION: i16 = 45 * MAP_SCALE;
pub const SNOW_ELEVATION: i16 = 55 * MAP_SCALE;

pub const WORLD_STEP: usize = 8;
pub const MESH_SIZE: usize = MAP_WIDTH / WORLD_STEP;
pub const WORLD_SPACING: f32 = 1.5;

pub const NUM_FISH: usize = 10;

pub struct Fish {
    pub pos: Vec3,
    pub caught: bool,
    pub angle: f32,
}

pub struct World {
    pub seed: u64,
    pub elevations: Vec<i16>,
    pub fishes: Vec<Fish>,
}

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x4d595f53454544 } else { seed },
        }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    fn next_u16(&mut self) -> u16 {
        self.next_u32() as u16
    }
}

impl World {
    pub fn new(seed: u64) -> Self {
        let mut world = Self {
            seed,
            elevations: vec![0; MAP_WIDTH * MAP_HEIGHT],
            fishes: Vec::new(),
        };
        world.generate();
        world
    }

    pub fn regenerate(&mut self) {
        self.seed = self.seed.wrapping_add(1013904223);
        self.generate();
    }

    fn generate(&mut self) {
        let mut rng = Rng::new(self.seed);
        self.elevations.fill(0);

        let total_iterations = MAP_WIDTH * MAP_HEIGHT / 128;

        for _ in 0..total_iterations {
            let x = (rng.next_u32() as usize) % MAP_WIDTH;
            let y = (rng.next_u32() as usize) % MAP_HEIGHT;
            let mut j = 1usize << (rng.next_u32() % 6);
            let mut l = 0usize;

            while j > 0 {
                if l == 0 && rng.next_u16() < u16::MAX / 4 {
                    l = (rng.next_u16() as usize) % (j + 1);
                }

                if l > 0 {
                    let x1 = x.saturating_sub(j);
                    let x2 = (x + j).min(MAP_WIDTH - 1);
                    let y1 = y.saturating_sub(j);
                    let y2 = (y + j).min(MAP_HEIGHT - 1);

                    for yy in y1..y2 {
                        for xx in x1..x2 {
                            let idx = yy * MAP_WIDTH + xx;
                            self.elevations[idx] =
                                self.elevations[idx].saturating_add(MAP_SCALE / 2);
                        }
                    }
                    l -= 1;
                }

                j -= 1;
            }
        }

        for val in self.elevations.iter_mut() {
            if *val < WATER_ELEVATION {
                *val = WATER_ELEVATION;
            }
        }

        self.spawn_fish(&mut rng);
    }

    fn spawn_fish(&mut self, rng: &mut Rng) {
        self.fishes.clear();
        let mut attempts = 0;

        while self.fishes.len() < NUM_FISH && attempts < 3000 {
            attempts += 1;
            let gx = 32 + (rng.next_u32() as usize % (MAP_WIDTH - 64));
            let gy = 32 + (rng.next_u32() as usize % (MAP_HEIGHT - 64));

            if self.elevations[gy * MAP_WIDTH + gx] <= WATER_ELEVATION {
                let half_w = (MAP_WIDTH as f32 * 0.5) * WORLD_SPACING;
                let half_h = (MAP_HEIGHT as f32 * 0.5) * WORLD_SPACING;
                let wx = gx as f32 * WORLD_SPACING - half_w;
                let wz = gy as f32 * WORLD_SPACING - half_h;
                let wy = (WATER_ELEVATION as f32) / (MAP_SCALE as f32);

                let angle = (rng.next_u32() as f32 / u32::MAX as f32) * std::f32::consts::TAU;

                self.fishes.push(Fish {
                    pos: vec3(wx, wy - 0.2, wz),
                    caught: false,
                    angle,
                });
            }
        }
    }

    pub fn get_raw_elevation(&self, x: usize, y: usize) -> i16 {
        if x >= MAP_WIDTH || y >= MAP_HEIGHT {
            WATER_ELEVATION
        } else {
            self.elevations[y * MAP_WIDTH + x]
        }
    }

    pub fn get_height(&self, world_x: f32, world_z: f32) -> f32 {
        let half_w = (MAP_WIDTH as f32 * 0.5) * WORLD_SPACING;
        let half_h = (MAP_HEIGHT as f32 * 0.5) * WORLD_SPACING;

        let gx = ((world_x + half_w) / WORLD_SPACING).clamp(0.0, (MAP_WIDTH - 1) as f32) as usize;
        let gy = ((world_z + half_h) / WORLD_SPACING).clamp(0.0, (MAP_HEIGHT - 1) as f32) as usize;

        (self.get_raw_elevation(gx, gy) as f32) / (MAP_SCALE as f32)
    }

    pub fn world_bounds(&self) -> f32 {
        (MAP_WIDTH as f32 * 0.5) * WORLD_SPACING
    }
}
