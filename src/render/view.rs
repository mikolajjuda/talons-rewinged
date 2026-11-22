use crate::flight::BirdState;
use macroquad::prelude::*;

const TARGET_DISTANCE: f32 = 50.0;
const REF_WIDTH: f32 = 640.0;
const REF_HEIGHT: f32 = 480.0;
const DIAGONAL_FOV_DEGREES: f32 = 110.0;

pub fn ui_scale() -> f32 {
    (screen_width() / REF_WIDTH).min(screen_height() / REF_HEIGHT)
}

pub struct View {
    pub pos: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub tan_y: f32,
    tan_x: f32,
    sec_x: f32,
    sec_y: f32,
}

impl View {
    pub fn new(bird: &BirdState) -> Self {
        let width = screen_width();
        let height = screen_height().max(1.0);
        let aspect = width / height;

        let tan_diagonal = (DIAGONAL_FOV_DEGREES.to_radians() * 0.5).tan();
        let tan_y = tan_diagonal / (1.0 + aspect * aspect).sqrt();
        let tan_x = tan_y * aspect;

        Self {
            pos: bird.pos,
            forward: bird.forward(),
            right: bird.right(),
            up: bird.up(),
            tan_y,
            tan_x,
            sec_x: (1.0 + tan_x * tan_x).sqrt(),
            sec_y: (1.0 + tan_y * tan_y).sqrt(),
        }
    }

    pub fn camera(&self) -> Camera3D {
        Camera3D {
            position: self.pos,
            target: self.pos + self.forward * TARGET_DISTANCE,
            up: self.up,
            fovy: 2.0 * self.tan_y.atan(),
            projection: Projection::Perspective,
            ..Default::default()
        }
    }

    pub fn focal_px(&self) -> f32 {
        screen_height().max(1.0) * 0.5 / self.tan_y
    }

    pub fn distance_squared(&self, point: Vec3) -> f32 {
        (point - self.pos).length_squared()
    }

    pub fn sees(&self, center: Vec3, radius: f32) -> bool {
        let offset = center - self.pos;
        let depth = offset.dot(self.forward);
        let sideways = offset.dot(self.right).abs();
        let vertical = offset.dot(self.up).abs();
        sideways - self.tan_x * depth <= radius * self.sec_x
            && vertical - self.tan_y * depth <= radius * self.sec_y
    }

    pub fn reaches(&self, center: Vec3, radius: f32, max_distance: f32) -> bool {
        let limit = max_distance + radius;
        self.distance_squared(center) <= limit * limit && self.sees(center, radius)
    }
}
