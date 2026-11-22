use super::sprites::{CREATURE_SCALE, RenderSprite};
use super::view::View;
use crate::world::World;
use macroquad::prelude::*;

const FISH_MAX_DISTANCE: f32 = 350.0;
const FISH_SPRITE_EXTENT: f32 = 80.0;

pub fn draw_fish(sprite: &RenderSprite, world: &World, view: &View, limit: f32) {
    let max_distance = FISH_MAX_DISTANCE.min(limit);
    let radius = FISH_SPRITE_EXTENT * CREATURE_SCALE;

    for fish in world.fishes.iter().filter(|fish| !fish.caught) {
        if !view.reaches(fish.pos, radius, max_distance) {
            continue;
        }

        for line in &sprite.lines {
            draw_line_3d(flat_point(fish.pos, line.from), flat_point(fish.pos, line.to), line.color);
        }
    }
}

fn flat_point(origin: Vec3, point: Vec2) -> Vec3 {
    origin + vec3(point.x, 0.0, -point.y) * CREATURE_SCALE
}
