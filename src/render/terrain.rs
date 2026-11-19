use crate::world::{
    MAP_HEIGHT, MAP_SCALE, MAP_WIDTH, ROCK_ELEVATION, SNOW_ELEVATION, WATER_ELEVATION,
    WORLD_SPACING, World,
};

use macroquad::models::{Mesh, Vertex};
use macroquad::prelude::*;

use super::palette::{
    COLOR_BLUE, COLOR_DKGRAY, COLOR_GREEN, COLOR_LTGRAY, COLOR_LTGREEN, COLOR_WHITE, scale_color,
};

const CHUNK_CELLS: usize = 64;
const MAX_CHUNK_VERTICES: usize = 8000;
const MAX_CHUNK_INDICES: usize = 4500;
const PANEL_SIZE_MAX: usize = 16;

const TERRAIN_AMBIENT: f32 = 0.65;
const TERRAIN_DIFFUSE: f32 = 0.35;
const TERRAIN_LIGHT_DIR: Vec3 = Vec3::new(-0.4, 0.8, -0.5);

pub struct TerrainChunk {
    pub mesh: Mesh,
    pub center: Vec3,
    pub radius: f32,
}

struct ChunkBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    min_pt: Vec3,
    max_pt: Vec3,
}

impl ChunkBuilder {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            min_pt: Vec3::splat(f32::MAX),
            max_pt: Vec3::splat(f32::MIN),
        }
    }

    fn push_quad(
        &mut self,
        p0: Vec3,
        p1: Vec3,
        p2: Vec3,
        p3: Vec3,
        col: Color,
        out_chunks: &mut Vec<TerrainChunk>,
    ) {
        if self.vertices.len() + 4 > MAX_CHUNK_VERTICES
            || self.indices.len() + 6 > MAX_CHUNK_INDICES
        {
            self.flush(out_chunks);
        }

        for &p in &[p0, p1, p2, p3] {
            self.min_pt = self.min_pt.min(p);
            self.max_pt = self.max_pt.max(p);
        }

        let base = self.vertices.len() as u16;
        self.vertices.push(Vertex::new2(p0, vec2(0.0, 0.0), col));
        self.vertices.push(Vertex::new2(p1, vec2(1.0, 0.0), col));
        self.vertices.push(Vertex::new2(p2, vec2(1.0, 1.0), col));
        self.vertices.push(Vertex::new2(p3, vec2(0.0, 1.0), col));

        self.indices.push(base);
        self.indices.push(base + 2);
        self.indices.push(base + 1);

        self.indices.push(base);
        self.indices.push(base + 3);
        self.indices.push(base + 2);
    }

    fn flush(&mut self, out_chunks: &mut Vec<TerrainChunk>) {
        if self.indices.is_empty() {
            return;
        }

        let center = (self.min_pt + self.max_pt) * 0.5;
        let radius = (self.max_pt - self.min_pt).length() * 0.5;

        out_chunks.push(TerrainChunk {
            mesh: Mesh {
                vertices: std::mem::take(&mut self.vertices),
                indices: std::mem::take(&mut self.indices),
                texture: None,
            },
            center,
            radius,
        });

        self.min_pt = Vec3::splat(f32::MAX);
        self.max_pt = Vec3::splat(f32::MIN);
    }
}

fn dither_blend(a: Color, b: Color) -> Color {
    Color::new((a.r + b.r) * 0.5, (a.g + b.g) * 0.5, (a.b + b.b) * 0.5, 1.0)
}

fn calc_normals(world: &World) -> Vec<Vec3> {
    let mut normals = vec![Vec3::ZERO; MAP_WIDTH * MAP_HEIGHT];

    for j in 0..(MAP_HEIGHT - 1) {
        let row = j * MAP_WIDTH;
        let next_row = (j + 1) * MAP_WIDTH;
        for i in 0..(MAP_WIDTH - 1) {
            let e = world.elevations[row + i];
            let dx = (world.elevations[row + (i + 1)] - e) as f32;
            let dy = (world.elevations[next_row + i] - e) as f32;
            normals[row + i] = vec3(dx, dy, -1.0).normalize();
        }
    }

    normals
}

fn test_same_slope(
    world: &World,
    normals: &[Vec3],
    occupied: &[bool],
    x: usize,
    y: usize,
    w: usize,
    h: usize,
) -> bool {
    if x + w >= MAP_WIDTH || y + h >= MAP_HEIGHT {
        return false;
    }

    let s = normals[y * MAP_WIDTH + x];
    for k2 in 0..h {
        let row = (y + k2) * MAP_WIDTH;
        for k1 in 0..w {
            let idx = row + (x + k1);
            if occupied[idx] {
                return false;
            }
            if (normals[idx] - s).length_squared() > 0.10 {
                return false;
            }
        }
    }

    let e00 = world.elevations[y * MAP_WIDTH + x] as f32;
    let e10 = world.elevations[y * MAP_WIDTH + (x + w)] as f32;
    let e01 = world.elevations[(y + h) * MAP_WIDTH + x] as f32;
    let e11 = world.elevations[(y + h) * MAP_WIDTH + (x + w)] as f32;

    for k1 in 1..w {
        let t = k1 as f32 / w as f32;
        let expected_top = e00 + t * (e10 - e00);
        let actual_top = world.elevations[y * MAP_WIDTH + (x + k1)] as f32;
        if (actual_top - expected_top).abs() > 30.0 {
            return false;
        }

        let expected_bot = e01 + t * (e11 - e01);
        let actual_bot = world.elevations[(y + h) * MAP_WIDTH + (x + k1)] as f32;
        if (actual_bot - expected_bot).abs() > 30.0 {
            return false;
        }
    }

    for k2 in 1..h {
        let t = k2 as f32 / h as f32;
        let expected_left = e00 + t * (e01 - e00);
        let actual_left = world.elevations[(y + k2) * MAP_WIDTH + x] as f32;
        if (actual_left - expected_left).abs() > 30.0 {
            return false;
        }

        let expected_right = e10 + t * (e11 - e10);
        let actual_right = world.elevations[(y + k2) * MAP_WIDTH + (x + w)] as f32;
        if (actual_right - expected_right).abs() > 30.0 {
            return false;
        }
    }

    true
}

pub(super) fn build_terrain_chunks(world: &World) -> Vec<TerrainChunk> {
    let half_w = (MAP_WIDTH as f32 * 0.5) * WORLD_SPACING;
    let half_h = (MAP_HEIGHT as f32 * 0.5) * WORLD_SPACING;
    let light_dir = TERRAIN_LIGHT_DIR.normalize();

    let normals = calc_normals(world);
    let mut occupied = vec![false; MAP_WIDTH * MAP_HEIGHT];

    let chunks_x = MAP_WIDTH.div_ceil(CHUNK_CELLS);
    let chunks_z = MAP_HEIGHT.div_ceil(CHUNK_CELLS);
    let mut builders: Vec<ChunkBuilder> = (0..(chunks_x * chunks_z))
        .map(|_| ChunkBuilder::new())
        .collect();
    let mut out_chunks = Vec::new();

    for threshold in (1..=8).rev() {
        for j in 0..(MAP_HEIGHT - 1) {
            for i in 0..(MAP_WIDTH - 1) {
                if occupied[j * MAP_WIDTH + i] {
                    continue;
                }

                let mut w = 1usize;
                let mut h = 1usize;

                loop {
                    let mut cont = false;

                    if w < PANEL_SIZE_MAX
                        && test_same_slope(world, &normals, &occupied, i, j, w + 1, h)
                    {
                        w += 1;
                        cont = true;
                    }

                    if h < PANEL_SIZE_MAX
                        && test_same_slope(world, &normals, &occupied, i, j, w, h + 1)
                    {
                        h += 1;
                        cont = true;
                    }

                    if !cont {
                        break;
                    }
                }

                if w >= threshold || h >= threshold {
                    for k2 in 0..h {
                        let row = (j + k2) * MAP_WIDTH;
                        for k1 in 0..w {
                            occupied[row + (i + k1)] = true;
                        }
                    }

                    let l = world.get_raw_elevation(i, j);

                    let r = ((i as u32).wrapping_mul(374761393)
                        ^ (j as u32).wrapping_mul(668265263)
                        ^ (world.seed as u32))
                        .wrapping_add(1013904223);
                    let rand_bit = (r & 1) != 0;
                    let rand_mod4 = (r >> 1) & 3;

                    let is_water = l <= WATER_ELEVATION
                        && world.get_raw_elevation(i + w - 1, j) <= WATER_ELEVATION
                        && world.get_raw_elevation(i, j + h - 1) <= WATER_ELEVATION
                        && world.get_raw_elevation(i + w - 1, j + h - 1) <= WATER_ELEVATION;

                    let base_col = if is_water {
                        COLOR_BLUE
                    } else if l < ROCK_ELEVATION {
                        if rand_bit {
                            COLOR_LTGREEN
                        } else {
                            dither_blend(COLOR_GREEN, COLOR_LTGREEN)
                        }
                    } else if l < SNOW_ELEVATION {
                        if rand_mod4 == 0 {
                            if rand_bit {
                                COLOR_LTGRAY
                            } else {
                                dither_blend(COLOR_DKGRAY, COLOR_LTGRAY)
                            }
                        } else if rand_bit {
                            COLOR_LTGREEN
                        } else {
                            dither_blend(COLOR_GREEN, COLOR_LTGREEN)
                        }
                    } else if rand_mod4 == 0 {
                        if rand_bit { COLOR_WHITE } else { COLOR_LTGRAY }
                    } else if rand_bit {
                        dither_blend(COLOR_LTGRAY, COLOR_WHITE)
                    } else {
                        dither_blend(COLOR_DKGRAY, COLOR_LTGRAY)
                    };

                    let x0 = i as f32 * WORLD_SPACING - half_w;
                    let x1 = (i + w) as f32 * WORLD_SPACING - half_w;
                    let z0 = j as f32 * WORLD_SPACING - half_h;
                    let z1 = (j + h) as f32 * WORLD_SPACING - half_h;

                    let y0 = l as f32 / MAP_SCALE as f32;
                    let y1 = world.get_raw_elevation(i + w, j) as f32 / MAP_SCALE as f32;
                    let y2 = world.get_raw_elevation(i + w, j + h) as f32 / MAP_SCALE as f32;
                    let y3 = world.get_raw_elevation(i, j + h) as f32 / MAP_SCALE as f32;

                    let p0 = vec3(x0, y0, z0);
                    let p1 = vec3(x1, y1, z0);
                    let p2 = vec3(x1, y2, z1);
                    let p3 = vec3(x0, y3, z1);

                    let col = if is_water {
                        base_col
                    } else {
                        let normal = (p3 - p0).cross(p1 - p0).normalize_or_zero();
                        let diffuse = normal.dot(light_dir).max(0.0);
                        scale_color(base_col, TERRAIN_AMBIENT + TERRAIN_DIFFUSE * diffuse)
                    };

                    let chunk_idx = (j / CHUNK_CELLS) * chunks_x + (i / CHUNK_CELLS);
                    builders[chunk_idx].push_quad(p0, p1, p2, p3, col, &mut out_chunks);
                }
            }
        }
    }

    for mut builder in builders {
        builder.flush(&mut out_chunks);
    }

    out_chunks
}

pub(super) fn build_water_mesh(_world: &World) -> Mesh {
    let half_w = (MAP_WIDTH as f32 * 0.5) * WORLD_SPACING;
    let bounds = half_w + 1500.0;
    let y = (WATER_ELEVATION as f32 / MAP_SCALE as f32) - 0.05;

    let vertices = vec![
        Vertex::new2(vec3(-bounds, y, -bounds), vec2(0.0, 0.0), COLOR_BLUE),
        Vertex::new2(vec3(bounds, y, -bounds), vec2(1.0, 0.0), COLOR_BLUE),
        Vertex::new2(vec3(bounds, y, bounds), vec2(1.0, 1.0), COLOR_BLUE),
        Vertex::new2(vec3(-bounds, y, bounds), vec2(0.0, 1.0), COLOR_BLUE),
    ];

    let indices = vec![0, 2, 1, 0, 3, 2];

    Mesh {
        vertices,
        indices,
        texture: None,
    }
}
