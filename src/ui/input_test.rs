use super::theme::*;
use crate::input::{GameInputs, InputSystem};
use macroquad::prelude::*;

pub fn draw_input_test(input: &mut InputSystem, out: &GameInputs) -> bool {
    let mut leave = false;
    if is_key_pressed(KeyCode::R) {
        input.reload();
    }
    if is_key_pressed(KeyCode::Escape) {
        leave = true;
    }

    let mut next_selected = input.selected;
    let mut reload_requested = false;

    let (dev_names, active_name, active_axes, active_buttons) = {
        let devs = input.devices();
        let sel = if next_selected >= devs.len() {
            0
        } else {
            next_selected
        };
        let d = devs[sel];
        (
            devs.iter().map(|d| d.name.clone()).collect::<Vec<_>>(),
            d.name.clone(),
            d.axes.clone(),
            d.buttons.clone(),
        )
    };

    if next_selected >= dev_names.len() {
        next_selected = 0;
    }
    if is_key_pressed(KeyCode::Tab) && !dev_names.is_empty() {
        next_selected = (next_selected + 1) % dev_names.len();
    }

    let top_y = 16.0;
    let nav_back = Rect::new(16.0, top_y, 140.0, 32.0);
    if draw_button(nav_back, "[Esc] Menu", false) {
        leave = true;
    }

    let nav_reload = Rect::new(166.0, top_y, 140.0, 32.0);
    if draw_button(nav_reload, "[R] Reload", false) {
        reload_requested = true;
    }

    draw_text(&input.status, 320.0, top_y + 22.0, 18.0, DARK_GREEN);

    let left_w = 380.0;
    let left_h = 520.0;
    let left_x = 16.0;
    let left_y = 58.0;

    draw_rectangle(left_x, left_y, left_w, left_h, PANEL_BG);
    draw_rectangle_lines(left_x, left_y, left_w, left_h, 2.5, PANEL_BORDER);
    draw_rectangle(left_x, left_y, left_w, 36.0, LIGHT_GREEN);
    draw_rectangle_lines(left_x, left_y, left_w, 36.0, 2.0, DARK_GREEN);
    draw_text(
        "GAME INPUTS",
        left_x + 14.0,
        left_y + 25.0,
        22.0,
        DARK_GREEN,
    );

    let mut gy = left_y + 60.0;
    let rows = [
        ("yaw", out.yaw, false),
        ("throttle", out.throttle, true),
        ("roll", out.roll, false),
        ("pitch", out.pitch, false),
    ];

    for (label, val, unipolar) in rows {
        draw_text(
            &format!("{label:<8}"),
            left_x + 18.0,
            gy + 14.0,
            19.0,
            DEEP_NAVY,
        );
        draw_text(
            &format!("{val:+.2}"),
            left_x + 104.0,
            gy + 14.0,
            18.0,
            DARK_GREEN,
        );
        draw_axis_meter(left_x + 168.0, gy, 190.0, 18.0, val, unipolar);
        gy += 38.0;
    }

    draw_line(
        left_x + 16.0,
        gy + 6.0,
        left_x + left_w - 16.0,
        gy + 6.0,
        1.5,
        PANEL_BORDER,
    );
    gy += 28.0;

    draw_text("switch", left_x + 18.0, gy + 16.0, 19.0, DEEP_NAVY);
    let pill_rect = Rect::new(left_x + 168.0, gy, 100.0, 24.0);
    let pill_col = if out.switch { LIGHT_GREEN } else { TRACK_BG };
    draw_rectangle(pill_rect.x, pill_rect.y, pill_rect.w, pill_rect.h, pill_col);
    draw_rectangle_lines(
        pill_rect.x,
        pill_rect.y,
        pill_rect.w,
        pill_rect.h,
        1.5,
        DARK_GREEN,
    );
    let pill_text = if out.switch { "ACTIVE" } else { "OFF" };
    draw_text(
        pill_text,
        pill_rect.x + 22.0,
        pill_rect.y + 17.0,
        16.0,
        DARK_GREEN,
    );

    let right_x = left_x + left_w + 16.0;
    let right_w = screen_width() - right_x - 16.0;
    let right_h = left_h;
    let right_y = left_y;

    draw_rectangle(right_x, right_y, right_w, right_h, PANEL_BG);
    draw_rectangle_lines(right_x, right_y, right_w, right_h, 2.5, PANEL_BORDER);
    draw_rectangle(right_x, right_y, right_w, 36.0, LIGHT_GREEN);
    draw_rectangle_lines(right_x, right_y, right_w, 36.0, 2.0, DARK_GREEN);
    draw_text(
        "DEVICE INSPECTOR",
        right_x + 14.0,
        right_y + 25.0,
        22.0,
        DARK_GREEN,
    );

    let mut ry = right_y + 48.0;
    draw_text(
        "Connected Devices ([Tab] / click):",
        right_x + 14.0,
        ry + 14.0,
        17.0,
        DEEP_NAVY,
    );
    ry += 26.0;

    for (i, name) in dev_names.iter().enumerate() {
        let is_sel = i == next_selected;
        let btn_r = Rect::new(right_x + 14.0, ry, right_w - 28.0, 26.0);
        let prefix = if is_sel { ">" } else { " " };
        if draw_button(btn_r, &format!("{prefix} [{i}] {name}"), is_sel) {
            next_selected = i;
        }
        ry += 30.0;
    }

    ry += 6.0;
    draw_line(
        right_x + 14.0,
        ry,
        right_x + right_w - 14.0,
        ry,
        1.5,
        PANEL_BORDER,
    );
    ry += 18.0;

    draw_text(
        &format!("Inspecting: {active_name}"),
        right_x + 14.0,
        ry,
        18.0,
        DARK_GREEN,
    );
    ry += 24.0;

    draw_text("Axes:", right_x + 14.0, ry, 17.0, DEEP_NAVY);
    ry += 18.0;
    for (id, val) in &active_axes {
        draw_text(
            &format!("  {id}"),
            right_x + 14.0,
            ry + 12.0,
            15.0,
            DEEP_NAVY,
        );
        draw_axis_meter(right_x + 180.0, ry, 120.0, 14.0, *val, false);
        draw_text(
            &format!("{val:+.2}"),
            right_x + 310.0,
            ry + 12.0,
            15.0,
            DARK_GREEN,
        );
        ry += 20.0;
        if ry > right_y + right_h - 100.0 {
            break;
        }
    }

    ry += 8.0;
    draw_text("Buttons:", right_x + 14.0, ry, 17.0, DEEP_NAVY);
    ry += 18.0;
    let mut bx = right_x + 14.0;
    for (id, &on) in &active_buttons {
        let b_col = if on { LIGHT_GREEN } else { TRACK_BG };
        let b_txt = if on {
            format!("{id}: ON")
        } else {
            format!("{id}")
        };
        let b_dim = measure_text(&b_txt, None, 14, 1.0);
        let bw = b_dim.width + 12.0;

        if bx + bw > right_x + right_w - 14.0 {
            bx = right_x + 14.0;
            ry += 24.0;
        }
        if ry > right_y + right_h - 26.0 {
            break;
        }

        draw_rectangle(bx, ry, bw, 20.0, b_col);
        draw_rectangle_lines(bx, ry, bw, 20.0, 1.0, DARK_GREEN);
        draw_text(&b_txt, bx + 6.0, ry + 15.0, 14.0, DARK_GREEN);
        bx += bw + 6.0;
    }

    if reload_requested {
        input.reload();
    }
    input.selected = next_selected;

    leave
}
