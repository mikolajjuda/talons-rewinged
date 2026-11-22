use macroquad::prelude::*;

pub const SKY_BLUE: Color = Color::new(0.66, 0.84, 0.96, 1.0);
pub const PANEL_BG: Color = Color::new(0.90, 0.96, 0.90, 0.92);
pub const PANEL_BORDER: Color = Color::new(0.40, 0.68, 0.35, 1.0);
pub const LIGHT_GREEN: Color = Color::new(0.46, 0.82, 0.36, 1.0);
pub const LIGHT_GREEN_HOVER: Color = Color::new(0.56, 0.92, 0.44, 1.0);
pub const DARK_GREEN: Color = Color::new(0.12, 0.32, 0.10, 1.0);
pub const DEEP_NAVY: Color = Color::new(0.08, 0.16, 0.22, 1.0);
pub const TRACK_BG: Color = Color::new(0.78, 0.88, 0.78, 1.0);

pub fn draw_button(rect: Rect, label: &str, active: bool) -> bool {
    let (mx, my) = mouse_position();
    let hovered = rect.contains(vec2(mx, my));
    let bg = if active || hovered {
        LIGHT_GREEN_HOVER
    } else {
        LIGHT_GREEN
    };

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, DARK_GREEN);

    let font_size = 18.0;
    let text_dim = measure_text(label, None, font_size as u16, 1.0);
    draw_text(
        label,
        rect.x + (rect.w - text_dim.width) * 0.5,
        rect.y + (rect.h + text_dim.height) * 0.5 - 2.0,
        font_size,
        DARK_GREEN,
    );

    hovered && is_mouse_button_pressed(MouseButton::Left)
}

pub fn draw_axis_meter(x: f32, y: f32, w: f32, h: f32, val: f32, unipolar: bool) {
    draw_rectangle(x, y, w, h, TRACK_BG);
    draw_rectangle_lines(x, y, w, h, 1.5, DARK_GREEN);

    if unipolar {
        let fill = val.clamp(0.0, 1.0) * w;
        if fill > 0.0 {
            draw_rectangle(x, y, fill, h, LIGHT_GREEN);
            draw_rectangle_lines(x, y, fill, h, 1.0, DARK_GREEN);
        }
    } else {
        let center = x + w * 0.5;
        draw_line(center, y, center, y + h, 1.5, DARK_GREEN);
        let norm = val.clamp(-1.0, 1.0);
        if norm >= 0.0 {
            let fill = norm * (w * 0.5);
            draw_rectangle(center, y, fill, h, LIGHT_GREEN);
            draw_rectangle_lines(center, y, fill, h, 1.0, DARK_GREEN);
        } else {
            let fill = -norm * (w * 0.5);
            draw_rectangle(center - fill, y, fill, h, LIGHT_GREEN);
            draw_rectangle_lines(center - fill, y, fill, h, 1.0, DARK_GREEN);
        }
    }
}
