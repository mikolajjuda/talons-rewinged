use super::view::View;
use macroquad::prelude::*;

const FULL_SKY_SINE: f32 = std::f32::consts::FRAC_1_SQRT_2;

pub fn draw_ground(view: &View, color: Color) {
    if view.forward.y >= FULL_SKY_SINE {
        return;
    }

    let width = screen_width();
    let height = screen_height();
    let focal = view.focal_px();

    let corners = [
        vec2(0.0, 0.0),
        vec2(width, 0.0),
        vec2(width, height),
        vec2(0.0, height),
    ];

    let below_horizon = |p: Vec2| {
        focal * view.forward.y + (p.x - width * 0.5) * view.right.y - (p.y - height * 0.5) * view.up.y
    };

    let polygon = clip(&corners, below_horizon);
    for i in 1..polygon.len().saturating_sub(1) {
        draw_triangle(polygon[0], polygon[i], polygon[i + 1], color);
    }
}

fn clip(corners: &[Vec2; 4], value: impl Fn(Vec2) -> f32) -> Vec<Vec2> {
    let mut polygon = Vec::with_capacity(6);

    for i in 0..4 {
        let a = corners[i];
        let b = corners[(i + 1) % 4];
        let (va, vb) = (value(a), value(b));

        if va < 0.0 {
            polygon.push(a);
        }
        if (va < 0.0) != (vb < 0.0) {
            polygon.push(a + (b - a) * (va / (va - vb)));
        }
    }

    polygon
}
