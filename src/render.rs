use crate::flight::BirdState;
use crate::world::{
    Fish, MAP_HEIGHT, MAP_SCALE, MAP_WIDTH, MESH_SIZE, ROCK_ELEVATION, SNOW_ELEVATION,
    WATER_ELEVATION, WORLD_SPACING, WORLD_STEP, World,
};
use macroquad::models::{Mesh, Vertex, draw_mesh, draw_sphere};
use macroquad::prelude::*;

const CHUNK_QUADS: usize = 16;

const COLOR_BLUE: Color = Color::new(0.0, 0.0, 0.667, 1.0);
const COLOR_GREEN: Color = Color::new(0.0, 0.667, 0.0, 1.0);
const COLOR_LTGREEN: Color = Color::new(0.333, 1.0, 0.333, 1.0);
const COLOR_DKGRAY: Color = Color::new(0.333, 0.333, 0.333, 1.0);
const COLOR_LTGRAY: Color = Color::new(0.667, 0.667, 0.667, 1.0);
const COLOR_WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
const COLOR_RED: Color = Color::new(1.0, 0.0, 0.0, 1.0);
const COLOR_BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);

pub struct Renderer {
    terrain_chunks: Vec<Mesh>,
    water_mesh: Mesh,
    world_seed: u64,
}

impl Renderer {
    pub fn new(world: &World) -> Self {
        Self {
            terrain_chunks: Self::build_terrain_chunks(world),
            water_mesh: Self::build_water_mesh(world),
            world_seed: world.seed,
        }
    }

    pub fn draw(&mut self, world: &World, bird: &BirdState) {
        if world.seed != self.world_seed {
            self.terrain_chunks = Self::build_terrain_chunks(world);
            self.water_mesh = Self::build_water_mesh(world);
            self.world_seed = world.seed;
        }

        let camera = Camera3D {
            position: bird.pos,
            target: bird.pos + bird.forward() * 50.0,
            up: bird.up(),
            fovy: 75.0f32.to_radians(),
            projection: Projection::Perspective,
            ..Default::default()
        };

        set_camera(&camera);

        for chunk in &self.terrain_chunks {
            draw_mesh(chunk);
        }

        draw_mesh(&self.water_mesh);

        for fish in &world.fishes {
            if !fish.caught {
                draw_sphere(fish.pos, 2.0, None, COLOR_RED);
                let tail_dir = vec3(fish.angle.cos(), 0.0, fish.angle.sin()) * 3.0;
                draw_line_3d(fish.pos, fish.pos - tail_dir, COLOR_WHITE);
            }
        }

        set_default_camera();

        let cx = screen_width() * 0.5;
        let cy = screen_height() * 0.5;
        draw_line(cx - 6.0, cy, cx + 6.0, cy, 2.0, COLOR_BLACK);
        draw_line(cx, cy - 6.0, cx, cy + 6.0, 2.0, COLOR_BLACK);
    }

    fn color_for_elevation(l: i16, x: usize, y: usize, world_seed: u64) -> Color {
        let block_x = x / 2;
        let block_y = y / 2;
        let seed = (world_seed as u32).wrapping_add(1013904223);

        let h = (block_x as u32)
            .wrapping_mul(374761393)
            .wrapping_add((block_y as u32).wrapping_mul(668265263))
            ^ seed;
        let rand_val = (h ^ (h >> 13)) & 3;

        if l <= WATER_ELEVATION {
            COLOR_BLUE
        } else if l < ROCK_ELEVATION {
            if rand_val == 0 {
                COLOR_GREEN
            } else {
                COLOR_LTGREEN
            }
        } else if l < SNOW_ELEVATION {
            if (rand_val & 3) == 0 {
                if (rand_val & 1) == 0 {
                    COLOR_LTGRAY
                } else {
                    COLOR_DKGRAY
                }
            } else {
                COLOR_LTGREEN
            }
        } else {
            if (rand_val & 3) == 0 {
                COLOR_WHITE
            } else {
                COLOR_LTGRAY
            }
        }
    }

    fn build_terrain_chunks(world: &World) -> Vec<Mesh> {
        let half_w = (MAP_WIDTH as f32 * 0.5) * WORLD_SPACING;
        let half_h = (MAP_HEIGHT as f32 * 0.5) * WORLD_SPACING;

        let num_chunks_x = (MESH_SIZE - 1) / CHUNK_QUADS;
        let num_chunks_z = (MESH_SIZE - 1) / CHUNK_QUADS;

        let mut chunks = Vec::new();

        for cz in 0..num_chunks_z {
            let mz_start = cz * CHUNK_QUADS;
            let mz_end = (mz_start + CHUNK_QUADS).min(MESH_SIZE - 1);
            let chunk_h = mz_end - mz_start;

            for cx in 0..num_chunks_x {
                let mx_start = cx * CHUNK_QUADS;
                let mx_end = (mx_start + CHUNK_QUADS).min(MESH_SIZE - 1);
                let chunk_w = mx_end - mx_start;

                let mut vertices = Vec::with_capacity(chunk_w * chunk_h * 4);
                let mut indices = Vec::with_capacity(chunk_w * chunk_h * 6);

                for mz in mz_start..mz_end {
                    let gy0 = mz * WORLD_STEP;
                    let gy1 = (mz + 1) * WORLD_STEP;

                    let wz0 = gy0 as f32 * WORLD_SPACING - half_h;
                    let wz1 = gy1 as f32 * WORLD_SPACING - half_h;

                    for mx in mx_start..mx_end {
                        let gx0 = mx * WORLD_STEP;
                        let gx1 = (mx + 1) * WORLD_STEP;

                        let wx0 = gx0 as f32 * WORLD_SPACING - half_w;
                        let wx1 = gx1 as f32 * WORLD_SPACING - half_w;

                        let e00 = world.get_raw_elevation(gx0, gy0);
                        let e10 = world.get_raw_elevation(gx1, gy0);
                        let e11 = world.get_raw_elevation(gx1, gy1);
                        let e01 = world.get_raw_elevation(gx0, gy1);

                        let wy00 = e00 as f32 / MAP_SCALE as f32;
                        let wy10 = e10 as f32 / MAP_SCALE as f32;
                        let wy11 = e11 as f32 / MAP_SCALE as f32;
                        let wy01 = e01 as f32 / MAP_SCALE as f32;

                        let avg_e = (e00 as i32 + e10 as i32 + e11 as i32 + e01 as i32) / 4;
                        let col = Self::color_for_elevation(avg_e as i16, gx0, gy0, world.seed);

                        let base = vertices.len() as u16;

                        vertices.push(Vertex::new2(vec3(wx0, wy00, wz0), vec2(0.0, 0.0), col));
                        vertices.push(Vertex::new2(vec3(wx1, wy10, wz0), vec2(1.0, 0.0), col));
                        vertices.push(Vertex::new2(vec3(wx1, wy11, wz1), vec2(1.0, 1.0), col));
                        vertices.push(Vertex::new2(vec3(wx0, wy01, wz1), vec2(0.0, 1.0), col));

                        indices.push(base);
                        indices.push(base + 3);
                        indices.push(base + 1);

                        indices.push(base + 1);
                        indices.push(base + 3);
                        indices.push(base + 2);
                    }
                }

                chunks.push(Mesh {
                    vertices,
                    indices,
                    texture: None,
                });
            }
        }

        chunks
    }

    fn build_water_mesh(world: &World) -> Mesh {
        let bounds = world.world_bounds() + 100.0;
        let y = (WATER_ELEVATION as f32) / (MAP_SCALE as f32);
        let c = COLOR_BLUE;

        let vertices = vec![
            Vertex::new2(vec3(-bounds, y, -bounds), vec2(0.0, 0.0), c),
            Vertex::new2(vec3(bounds, y, -bounds), vec2(1.0, 0.0), c),
            Vertex::new2(vec3(bounds, y, bounds), vec2(1.0, 1.0), c),
            Vertex::new2(vec3(-bounds, y, bounds), vec2(0.0, 1.0), c),
        ];

        let indices = vec![0, 2, 1, 0, 3, 2];

        Mesh {
            vertices,
            indices,
            texture: None,
        }
    }
}
