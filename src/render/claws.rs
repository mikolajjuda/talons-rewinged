use super::sprites::SpriteLibrary;
use super::view::ui_scale;
use crate::palette;
use crate::sprites::id;
use macroquad::prelude::*;

const SEGMENTS_TOE: usize = 4;
const SEGMENTS_LEG: usize = 5;

#[derive(Clone, Copy)]
struct ClawSpec {
    thickness: f32,
    segments: usize,
    draw_tip: bool,
}

pub fn draw_claws(sprites: &SpriteLibrary, claws_down: f32) {
    let claws_down = claws_down.clamp(0.0, 1.0);
    if claws_down <= 0.0 {
        return;
    }

    let w = screen_width();
    let h = screen_height();
    if w <= 0.0 || h <= 0.0 {
        return;
    }

    let scale = ui_scale();
    let claws_up = 1.0 - claws_down;

    let hand = vec2(
        claws_up * (0.3 * w) + claws_down * (0.4 * w),
        claws_up * (h - 125.0 * scale) + claws_down * h,
    );

    let toe = ClawSpec {
        thickness: 22.0 * scale,
        segments: SEGMENTS_TOE,
        draw_tip: true,
    };
    let leg = ClawSpec {
        thickness: 38.0 * scale,
        segments: SEGMENTS_LEG,
        draw_tip: false,
    };

    let toes = [
        (vec2(-30.0, -50.0 * claws_up), vec2(-5.0, 0.0)),
        (vec2(-10.0, -60.0 * claws_up), vec2(0.0, 0.0)),
        (vec2(10.0, -60.0 * claws_up), vec2(0.0, 0.0)),
        (vec2(30.0, -50.0 * claws_up), vec2(5.0, 0.0)),
        (vec2(25.0, 40.0 * claws_up), vec2(5.0, 0.0)),
    ];

    for (tip, base) in toes {
        draw_pair(sprites, w, scale, hand + tip * scale, hand + base * scale, toe);
    }

    draw_pair(sprites, w, scale, hand, vec2(6.0 * w / 20.0, h), leg);
}

fn draw_pair(sprites: &SpriteLibrary, width: f32, scale: f32, tip: Vec2, base: Vec2, spec: ClawSpec) {
    draw_claw(sprites, scale, tip, base, spec, false);
    draw_claw(
        sprites,
        scale,
        vec2(width - tip.x, tip.y),
        vec2(width - base.x, base.y),
        spec,
        true,
    );
}

fn draw_claw(sprites: &SpriteLibrary, scale: f32, tip: Vec2, base: Vec2, spec: ClawSpec, flip_x: bool) {
    let segments = spec.segments as f32;
    let inner = (spec.thickness - 2.0 * scale).max(1.0);
    let outline = palette::color(palette::BLACK as i32);
    let fill = palette::color(palette::YELLOW as i32);

    for i in 0..spec.segments {
        let start = tip.lerp(base, i as f32 / segments);
        let end = tip.lerp(base, (i + 1) as f32 / segments);
        draw_thick_line(start, end, spec.thickness, outline);
        draw_thick_line(start, end, inner, fill);
    }

    if spec.draw_tip {
        let sprite_id = if tip.y < base.y { id::TALON_UP } else { id::TALON_DOWN };
        sprites.get(sprite_id).draw_bitmaps_2d(tip.x, tip.y, scale, flip_x);
    }
}

fn draw_thick_line(start: Vec2, end: Vec2, thickness: f32, color: Color) {
    let radius = (thickness * 0.5).max(0.5);
    draw_circle(start.x, start.y, radius, color);

    if start.distance_squared(end) > 0.0 {
        draw_line(start.x, start.y, end.x, end.y, thickness, color);
        draw_circle(end.x, end.y, radius, color);
    }
}
