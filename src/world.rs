use macroquad::prelude::*;

pub const MAP_WIDTH: usize = 1024;
pub const MAP_HEIGHT: usize = 1024;

pub const MAP_SCALE: i16 = 150;
pub const WATER_ELEVATION: i16 = 15 * MAP_SCALE;
pub const BIRD_ELEVATION: f32 = 10.0;
pub const ROCK_ELEVATION: i16 = 45 * MAP_SCALE;
pub const SNOW_ELEVATION: i16 = 55 * MAP_SCALE;

pub const WORLD_SPACING: f32 = 1.5;

const PANEL_SIZE_MAX: usize = 16;
const SAME_SLOPE_NORM_SQR: f32 = 0.10;

pub const NUM_BIRDS: usize = 256;
const BIRD_SPEED: f32 = 10.0 * WORLD_SPACING;
const BIRD_TURN_RATE: f32 = std::f32::consts::TAU / 10.0;

pub struct Fish {
    pub pos: Vec3,
    pub caught: bool,
    pub angle: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Boat {
    pub pos: Vec3,
    pub sprite_id: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Tree {
    pub pos: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub struct Bird {
    pub pos: Vec3,
    pub angle: f32,
}

pub struct World {
    pub seed: u64,
    pub elevations: Vec<i16>,
    pub fishes: Vec<Fish>,
    pub boats: Vec<Boat>,
    pub trees: Vec<Tree>,
    pub birds: Vec<Bird>,
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

    fn next_f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }
}

impl World {
    pub fn new(seed: u64) -> Self {
        let mut world = Self {
            seed,
            elevations: vec![0; MAP_WIDTH * MAP_HEIGHT],
            fishes: Vec::new(),
            boats: Vec::new(),
            trees: Vec::new(),
            birds: Vec::new(),
        };
        world.generate();
        world
    }

    pub fn regenerate(&mut self) {
        self.seed = self.seed.wrapping_add(1013904223);
        self.generate();
    }

    pub fn update(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        let half_w = (MAP_WIDTH as f32 * 0.5) * WORLD_SPACING;
        let half_h = (MAP_HEIGHT as f32 * 0.5) * WORLD_SPACING;

        for bird in &mut self.birds {
            bird.pos.x += BIRD_SPEED * dt * bird.angle.cos();
            bird.pos.z += BIRD_SPEED * dt * bird.angle.sin();
            bird.angle = (bird.angle + BIRD_TURN_RATE * dt).rem_euclid(std::f32::consts::TAU);

            let gx =
                ((bird.pos.x + half_w) / WORLD_SPACING).clamp(0.0, (MAP_WIDTH - 1) as f32) as usize;
            let gy = ((bird.pos.z + half_h) / WORLD_SPACING).clamp(0.0, (MAP_HEIGHT - 1) as f32)
                as usize;
            let ground_h = self.elevations[gy * MAP_WIDTH + gx] as f32 / MAP_SCALE as f32;

            bird.pos.y = BIRD_ELEVATION + ground_h;
        }
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

        self.spawn_panel_objects(&mut rng);
        self.spawn_birds(&mut rng);
    }

    fn calc_normals(&self) -> Vec<Vec3> {
        let mut normals = vec![Vec3::ZERO; MAP_WIDTH * MAP_HEIGHT];

        for j in 0..(MAP_HEIGHT - 1) {
            for i in 0..(MAP_WIDTH - 1) {
                let idx = j * MAP_WIDTH + i;
                let dx = (self.elevations[idx + 1] - self.elevations[idx]) as f32;
                let dy = (self.elevations[(j + 1) * MAP_WIDTH + i] - self.elevations[idx]) as f32;
                normals[idx] = vec3(dx, dy, -1.0).normalize();
            }
        }

        normals
    }

    fn spawn_panel_objects(&mut self, rng: &mut Rng) {
        self.fishes.clear();
        self.boats.clear();
        self.trees.clear();

        let normals = self.calc_normals();
        let mut occupied = vec![false; MAP_WIDTH * MAP_HEIGHT];
        let water_y = (WATER_ELEVATION as f32) / (MAP_SCALE as f32);

        for threshold in (1..=8).rev() {
            for j in 0..(MAP_HEIGHT - 1) {
                for i in 0..(MAP_WIDTH - 1) {
                    if occupied[j * MAP_WIDTH + i] {
                        continue;
                    }

                    let base_normal = normals[j * MAP_WIDTH + i];
                    let mut w = 1usize;
                    let mut h = 1usize;

                    loop {
                        let mut cont = false;

                        if w < PANEL_SIZE_MAX && i + w + 1 < MAP_WIDTH {
                            let same_col = (0..h).all(|k2| {
                                let n = normals[(j + k2) * MAP_WIDTH + (i + w)];
                                (n - base_normal).length_squared() <= SAME_SLOPE_NORM_SQR
                            });
                            if same_col {
                                w += 1;
                                cont = true;
                            }
                        }

                        if h < PANEL_SIZE_MAX && j + h + 1 < MAP_HEIGHT {
                            let same_row = (0..w).all(|k1| {
                                let n = normals[(j + h) * MAP_WIDTH + (i + k1)];
                                (n - base_normal).length_squared() <= SAME_SLOPE_NORM_SQR
                            });
                            if same_row {
                                h += 1;
                                cont = true;
                            }
                        }

                        if !cont {
                            break;
                        }
                    }

                    if w >= threshold || h >= threshold {
                        for k2 in 0..h {
                            for k1 in 0..w {
                                occupied[(j + k2) * MAP_WIDTH + (i + k1)] = true;
                            }
                        }

                        let is_water = self.elevations[j * MAP_WIDTH + i] <= WATER_ELEVATION
                            && self.elevations[j * MAP_WIDTH + (i + w - 1)] <= WATER_ELEVATION
                            && self.elevations[(j + h - 1) * MAP_WIDTH + i] <= WATER_ELEVATION
                            && self.elevations[(j + h - 1) * MAP_WIDTH + (i + w - 1)]
                                <= WATER_ELEVATION;

                        if is_water {
                            if rng.next_f32() < 0.05 {
                                let (wx, wz) =
                                    self.grid_to_world((i + w / 2) as f32, (j + h / 2) as f32);

                                if rng.next_f32() < 0.1 {
                                    let sprite_id = if (rng.next_u16() & 1) != 0 { 4 } else { 5 };
                                    self.boats.push(Boat {
                                        pos: vec3(wx, water_y + 2.0, wz),
                                        sprite_id,
                                    });
                                } else {
                                    let angle = rng.next_f32() * std::f32::consts::TAU;
                                    self.fishes.push(Fish {
                                        pos: vec3(wx, water_y + 0.25, wz),
                                        caught: false,
                                        angle,
                                    });
                                }
                            }
                        } else if self.elevations[j * MAP_WIDTH + i] < ROCK_ELEVATION
                            && rng.next_f32() < 0.03
                        {
                            let (wx, wz) =
                                self.grid_to_world((i + w / 2) as f32, (j + h / 2) as f32);
                            let wy = self.get_height(wx, wz);
                            self.trees.push(Tree {
                                pos: vec3(wx, wy, wz),
                            });
                        }
                    }
                }
            }
        }
    }

    fn spawn_birds(&mut self, rng: &mut Rng) {
        self.birds.clear();
        self.birds.reserve(NUM_BIRDS);

        for _ in 0..NUM_BIRDS {
            let gx = rng.next_f32() * MAP_WIDTH as f32;
            let gy = rng.next_f32() * MAP_HEIGHT as f32;
            let (wx, wz) = self.grid_to_world(gx, gy);
            let ground_h = self.get_height(wx, wz);

            self.birds.push(Bird {
                pos: vec3(wx, BIRD_ELEVATION + ground_h, wz),
                angle: rng.next_f32() * std::f32::consts::TAU,
            });
        }
    }

    fn grid_to_world(&self, gx: f32, gy: f32) -> (f32, f32) {
        let half_w = (MAP_WIDTH as f32 * 0.5) * WORLD_SPACING;
        let half_h = (MAP_HEIGHT as f32 * 0.5) * WORLD_SPACING;
        (gx * WORLD_SPACING - half_w, gy * WORLD_SPACING - half_h)
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
