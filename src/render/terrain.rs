use super::chunks::{ChunkSet, Tiler};
use crate::palette;
use crate::world::{Panel, World};
use macroquad::models::Vertex;
use macroquad::prelude::*;

const TERRAIN_TILE_EXTENT: f32 = 48.0;
const WATER_OVERLAP: f32 = 0.05;
const QUAD_INDICES: [u16; 6] = [0, 2, 1, 0, 3, 2];

pub fn build(world: &World) -> ChunkSet {
    let mut tiler = Tiler::new(TERRAIN_TILE_EXTENT);

    for panel in world.panels() {
        let corners = corners_of(world, panel);
        let color = panel_color(panel);

        let vertices = corners.map(|position| Vertex::new2(position, Vec2::ZERO, color));
        tiler.push((corners[0] + corners[2]) * 0.5, &vertices, &QUAD_INDICES);
    }

    tiler.finish()
}

fn corners_of(world: &World, panel: &Panel) -> [Vec3; 4] {
    let (x, y, w, h) = (
        panel.x as usize,
        panel.y as usize,
        panel.w as usize,
        panel.h as usize,
    );

    let corner = |dx: usize, dy: usize| {
        let (wx, wz) = World::grid_to_world((x + dx) as f32, (y + dy) as f32);
        let raw = world.raw_elevation(x + dx, y + dy) as f32;
        vec3(wx, world.height_from_raw(raw), wz)
    };

    let overlap = if panel.water { WATER_OVERLAP } else { 0.0 };
    let grow = |point: Vec3, sx: f32, sz: f32| point + vec3(sx * overlap, 0.0, sz * overlap);

    [
        grow(corner(0, 0), -1.0, -1.0),
        grow(corner(w, 0), 1.0, -1.0),
        grow(corner(w, h), 1.0, 1.0),
        grow(corner(0, h), -1.0, 1.0),
    ]
}

fn panel_color(panel: &Panel) -> Color {
    let primary = palette::color(panel.color.primary as i32);
    match panel.color.secondary {
        Some(second) => palette::blend(primary, palette::color(second as i32)),
        None => primary,
    }
}
