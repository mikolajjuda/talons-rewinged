use macroquad::prelude::Color;

pub const BLACK: u8 = 0;
pub const BLUE: u8 = 1;
pub const GREEN: u8 = 2;
pub const LTGRAY: u8 = 7;
pub const DKGRAY: u8 = 8;
pub const LTGREEN: u8 = 10;
pub const LTCYAN: u8 = 11;
pub const YELLOW: u8 = 14;
pub const WHITE: u8 = 15;

const TABLE: [Color; 16] = [
    Color::new(0.000, 0.000, 0.000, 1.0),
    Color::new(0.000, 0.000, 0.667, 1.0),
    Color::new(0.000, 0.667, 0.000, 1.0),
    Color::new(0.000, 0.667, 0.667, 1.0),
    Color::new(0.667, 0.000, 0.000, 1.0),
    Color::new(0.667, 0.000, 0.667, 1.0),
    Color::new(0.667, 0.333, 0.000, 1.0),
    Color::new(0.667, 0.667, 0.667, 1.0),
    Color::new(0.333, 0.333, 0.333, 1.0),
    Color::new(0.333, 0.333, 1.000, 1.0),
    Color::new(0.333, 1.000, 0.333, 1.0),
    Color::new(0.333, 1.000, 1.000, 1.0),
    Color::new(1.000, 0.333, 0.333, 1.0),
    Color::new(1.000, 0.333, 1.000, 1.0),
    Color::new(1.000, 1.000, 0.333, 1.0),
    Color::new(1.000, 1.000, 1.000, 1.0),
];

pub fn color(index: i32) -> Color {
    TABLE[(index & 0x0F) as usize]
}

pub fn shade(color: Color, brightness: f32) -> Color {
    Color::new(
        (color.r * brightness).min(1.0),
        (color.g * brightness).min(1.0),
        (color.b * brightness).min(1.0),
        color.a,
    )
}

pub fn blend(a: Color, b: Color) -> Color {
    Color::new((a.r + b.r) * 0.5, (a.g + b.g) * 0.5, (a.b + b.b) * 0.5, 1.0)
}
