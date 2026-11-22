use super::view::View;
use crate::world::World;
use macroquad::models::{Mesh, Vertex, draw_mesh};
use macroquad::prelude::*;

const MAX_CHUNK_VERTICES: usize = 8000;
const MAX_CHUNK_INDICES: usize = 4500;

struct Chunk {
    mesh: Mesh,
    center: Vec3,
    radius: f32,
}

pub struct ChunkSet {
    chunks: Vec<Chunk>,
}

impl ChunkSet {
    pub fn draw(&self, view: &View, max_distance: f32) {
        for chunk in &self.chunks {
            if view.reaches(chunk.center, chunk.radius, max_distance) {
                draw_mesh(&chunk.mesh);
            }
        }
    }
}

struct ChunkBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    min: Vec3,
    max: Vec3,
}

impl ChunkBuilder {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            min: Vec3::splat(f32::MAX),
            max: Vec3::splat(f32::MIN),
        }
    }

    fn push(&mut self, vertices: &[Vertex], indices: &[u16], out: &mut Vec<Chunk>) {
        let overflows = self.vertices.len() + vertices.len() > MAX_CHUNK_VERTICES
            || self.indices.len() + indices.len() > MAX_CHUNK_INDICES;
        if overflows {
            self.flush(out);
        }

        let base = self.vertices.len() as u16;
        for vertex in vertices {
            self.min = self.min.min(vertex.position);
            self.max = self.max.max(vertex.position);
        }
        self.vertices.extend_from_slice(vertices);
        self.indices.extend(indices.iter().map(|index| base + index));
    }

    fn flush(&mut self, out: &mut Vec<Chunk>) {
        if self.indices.is_empty() {
            return;
        }

        let center = (self.min + self.max) * 0.5;
        let radius = (self.max - self.min).length() * 0.5;

        out.push(Chunk {
            mesh: Mesh {
                vertices: std::mem::take(&mut self.vertices),
                indices: std::mem::take(&mut self.indices),
                texture: None,
            },
            center,
            radius,
        });

        self.min = Vec3::splat(f32::MAX);
        self.max = Vec3::splat(f32::MIN);
    }
}

pub struct Tiler {
    builders: Vec<ChunkBuilder>,
    columns: usize,
    tile_extent: f32,
    finished: Vec<Chunk>,
}

impl Tiler {
    pub fn new(tile_extent: f32) -> Self {
        let columns = (World::half_extent() * 2.0 / tile_extent).ceil() as usize;
        Self {
            builders: (0..columns * columns).map(|_| ChunkBuilder::new()).collect(),
            columns,
            tile_extent,
            finished: Vec::new(),
        }
    }

    pub fn push(&mut self, at: Vec3, vertices: &[Vertex], indices: &[u16]) {
        let slot = self.slot(at);
        self.builders[slot].push(vertices, indices, &mut self.finished);
    }

    pub fn finish(mut self) -> ChunkSet {
        for builder in self.builders.iter_mut() {
            builder.flush(&mut self.finished);
        }
        ChunkSet {
            chunks: self.finished,
        }
    }

    fn slot(&self, at: Vec3) -> usize {
        let half = World::half_extent();
        let last = self.columns - 1;
        let column = (((at.x + half) / self.tile_extent).max(0.0) as usize).min(last);
        let row = (((at.z + half) / self.tile_extent).max(0.0) as usize).min(last);
        row * self.columns + column
    }
}
