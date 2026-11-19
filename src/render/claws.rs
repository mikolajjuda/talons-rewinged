use super::assets::RenderAssets;
use macroquad::prelude::*;

const TEMPLE_REF_WIDTH: f32 = 640.0;
const TEMPLE_REF_HEIGHT: f32 = 480.0;
const TEMPLE_YELLOW: Color = Color::new(1.0, 1.0, 85.0 / 255.0, 1.0);

#[derive(Clone, Copy)]
struct ClawSpec {
    thickness: f32,
    segments: usize,
    draw_tip: bool,
}

pub(super) fn draw_claws(assets: &RenderAssets, claws_down: f32) {
    let claws_down = claws_down.clamp(0.0, 1.0);
    if claws_down <= 0.0 {
        return;
    }

    let w = screen_width();
    let h = screen_height();
    if w <= 0.0 || h <= 0.0 {
        return;
    }

    let scale = (w / TEMPLE_REF_WIDTH).min(h / TEMPLE_REF_HEIGHT);
    let claws_up = 1.0 - claws_down;

    let hand = vec2(
        (1.0 - claws_down) * (0.3 * w) + claws_down * (0.4 * w),
        (1.0 - claws_down) * (h - 125.0 * scale) + claws_down * h,
    );

    let toe_spec = ClawSpec {
        thickness: 22.0 * scale,
        segments: 4,
        draw_tip: true,
    };

    draw_claw_pair(
        assets,
        w,
        scale,
        hand + vec2(-30.0, -50.0 * claws_up) * scale,
        hand + vec2(-5.0, 0.0) * scale,
        toe_spec,
    );
    draw_claw_pair(
        assets,
        w,
        scale,
        hand + vec2(-10.0, -60.0 * claws_up) * scale,
        hand,
        toe_spec,
    );
    draw_claw_pair(
        assets,
        w,
        scale,
        hand + vec2(10.0, -60.0 * claws_up) * scale,
        hand,
        toe_spec,
    );
    draw_claw_pair(
        assets,
        w,
        scale,
        hand + vec2(30.0, -50.0 * claws_up) * scale,
        hand + vec2(5.0, 0.0) * scale,
        toe_spec,
    );
    draw_claw_pair(
        assets,
        w,
        scale,
        hand + vec2(25.0, 40.0 * claws_up) * scale,
        hand + vec2(5.0, 0.0) * scale,
        toe_spec,
    );

    let leg_spec = ClawSpec {
        thickness: 38.0 * scale,
        segments: 5,
        draw_tip: false,
    };

    draw_claw_pair(assets, w, scale, hand, vec2(6.0 * w / 20.0, h), leg_spec);
}

fn draw_claw_pair(
    assets: &RenderAssets,
    screen_width: f32,
    scale: f32,
    tip: Vec2,
    base: Vec2,
    spec: ClawSpec,
) {
    draw_claw(assets, scale, tip, base, spec, false);
    draw_claw(
        assets,
        scale,
        vec2(screen_width - tip.x, tip.y),
        vec2(screen_width - base.x, base.y),
        spec,
        true,
    );
}

fn draw_claw(
    assets: &RenderAssets,
    scale: f32,
    tip: Vec2,
    base: Vec2,
    spec: ClawSpec,
    flip_x: bool,
) {
    let segs = spec.segments as f32;
    let inner_thickness = (spec.thickness - 2.0 * scale).max(1.0);

    for i in 0..spec.segments {
        let start = tip.lerp(base, i as f32 / segs);
        let end = tip.lerp(base, (i + 1) as f32 / segs);

        draw_temple_line(start, end, spec.thickness, BLACK);
        draw_temple_line(start, end, inner_thickness, TEMPLE_YELLOW);
    }

    if spec.draw_tip {
        let bitmap = if tip.y < base.y { 7 } else { 8 };
        assets.get(bitmap).draw_2d(tip.x, tip.y, scale, flip_x);
    }
}

fn draw_temple_line(start: Vec2, end: Vec2, thickness: f32, color: Color) {
    let radius = (thickness * 0.5).max(0.5);
    draw_circle(start.x, start.y, radius, color);

    if start.distance_squared(end) > 0.0 {
        draw_line(start.x, start.y, end.x, end.y, thickness, color);
        draw_circle(end.x, end.y, radius, color);
    }
}
