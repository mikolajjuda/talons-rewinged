use macroquad::prelude::Color;

pub(super) const COLOR_BLUE: Color = Color::new(0.0, 0.0, 0.667, 1.0);
pub(super) const COLOR_GREEN: Color = Color::new(0.0, 0.667, 0.0, 1.0);
pub(super) const COLOR_LTGREEN: Color = Color::new(0.333, 1.0, 0.333, 1.0);
pub(super) const COLOR_DKGRAY: Color = Color::new(0.333, 0.333, 0.333, 1.0);
pub(super) const COLOR_LTGRAY: Color = Color::new(0.667, 0.667, 0.667, 1.0);
pub(super) const COLOR_WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
pub(super) const COLOR_RED: Color = Color::new(1.0, 0.0, 0.0, 1.0);
pub(super) const COLOR_BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);

const TEMPLE_PALETTE: [Color; 16] = [
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

pub(super) fn palette_color(value: i32) -> Color {
    TEMPLE_PALETTE[(value as u32 & 0x0F) as usize]
}

pub(super) fn scale_color(color: Color, brightness: f32) -> Color {
    Color::new(
        (color.r * brightness).min(1.0),
        (color.g * brightness).min(1.0),
        (color.b * brightness).min(1.0),
        color.a,
    )
}
