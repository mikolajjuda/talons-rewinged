use super::view::ui_scale;
use crate::game::{Game, Phase, TARGET_FISH};
use crate::palette;
use crate::world::HORIZONTAL_UNIT;
use macroquad::prelude::*;

const PROMPT_SECONDS: f32 = 5.0;
const CROSSHAIR_HALF_SIZE: f32 = 5.0;
const CROSSHAIR_THICKNESS: f32 = 2.0;
const SPEED_UNIT: f32 = 1000.0 * HORIZONTAL_UNIT;

pub fn draw(game: &Game, show_fps: bool) {
    let ink = palette::color(palette::BLACK as i32);

    draw_crosshair(ink);
    draw_readouts(game, ink, show_fps);

    if game.phase() == Phase::Playing && game.elapsed() < PROMPT_SECONDS && blink_on() {
        draw_prompt();
    }

    if game.phase() != Phase::Playing {
        draw_end_screen(game);
    }
}

fn blink_on() -> bool {
    ((get_time() * 2.0) as u32) % 2 == 0
}

fn font_size() -> f32 {
    (9.0 * ui_scale()).max(14.0)
}

fn draw_crosshair(color: Color) {
    let scale = ui_scale().max(1.0);
    let half = CROSSHAIR_HALF_SIZE * scale;
    let thickness = CROSSHAIR_THICKNESS * scale.sqrt();
    let cx = screen_width() * 0.5;
    let cy = screen_height() * 0.5;

    draw_line(cx - half, cy, cx + half, cy, thickness, color);
    draw_line(cx, cy - half, cx, cy + half, thickness, color);
}

fn draw_readouts(game: &Game, ink: Color, show_fps: bool) {
    let bird = game.bird();
    let world = game.world();

    let forward = bird.forward();
    let pitch = forward.y.clamp(-1.0, 1.0).asin().to_degrees();
    let roll = (-bird.right().y).atan2(bird.up().y).to_degrees();
    let heading = forward.x.atan2(-forward.z).to_degrees().rem_euclid(360.0);
    let height = world
        .raw_from_height(bird.pos.y - world.height_at(bird.pos.x, bird.pos.z))
        .round() as i32;
    let speed = bird.speed() / SPEED_UNIT;

    let first =
        format!("Pitch:{pitch:5.1} Roll:{roll:5.1} Heading:{heading:5.1} Height:{height:5} Speed:{speed:4.2}");
    let second = format!(
        "Fish Remaining:{} Time:{:3.2} Best:{:3.2}",
        game.fish_remaining(),
        game.elapsed(),
        game.best_time()
    );

    let size = font_size();
    draw_text(&first, 8.0, size, size, ink);
    draw_text(&second, 8.0, size * 2.1, size, ink);

    if show_fps {
        let text = format!("FPS: {}", get_fps());
        let width = measure_text(&text, None, size as u16, 1.0).width;
        draw_text(&text, screen_width() - width - 10.0, size, size, ink);
    }
}

fn draw_prompt() {
    let message = format!("Catch {TARGET_FISH} Fish");
    let size = 28.0;
    let width = measure_text(&message, None, size as u16, 1.0).width;
    draw_text(
        &message,
        (screen_width() - width) * 0.5,
        screen_height() * 0.5 - 64.0,
        size,
        WHITE,
    );
}

fn draw_end_screen(game: &Game) {
    let won = game.phase() == Phase::Won;
    let scale = ui_scale().clamp(1.0, 2.0);

    let (accent, title) = if won {
        (Color::new(0.16, 0.62, 0.30, 1.0), format!("ALL {TARGET_FISH} FISH CAUGHT"))
    } else {
        (Color::new(0.80, 0.20, 0.20, 1.0), "CRASHED".to_string())
    };

    let mut lines = vec![format!("Time  {:.2} s", game.elapsed())];
    if won {
        lines.push(format!("Best  {:.2} s", game.best_time()));
    } else {
        lines.push(format!("Fish  {} / {TARGET_FISH}", TARGET_FISH - game.fish_remaining()));
    }

    let panel_w = 420.0 * scale;
    let panel_h = (150.0 + 28.0 * lines.len() as f32) * scale;
    let left = (screen_width() - panel_w) * 0.5;
    let top = (screen_height() - panel_h) * 0.4;

    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.35));
    draw_rectangle(left, top, panel_w, panel_h, Color::new(0.08, 0.10, 0.14, 0.96));
    draw_rectangle(left, top, panel_w, 8.0 * scale, accent);
    draw_rectangle_lines(left, top, panel_w, panel_h, 2.0, Color::new(1.0, 1.0, 1.0, 0.25));

    draw_centered(&title, left, panel_w, top + 62.0 * scale, 34.0 * scale, WHITE);

    let mut baseline = top + 108.0 * scale;
    for line in &lines {
        draw_centered(line, left, panel_w, baseline, 22.0 * scale, Color::new(0.85, 0.88, 0.92, 1.0));
        baseline += 28.0 * scale;
    }

    draw_centered(
        "[Enter] New world     [Esc] Menu",
        left,
        panel_w,
        top + panel_h - 20.0 * scale,
        17.0 * scale,
        Color::new(0.6, 0.65, 0.72, 1.0),
    );
}

fn draw_centered(text: &str, left: f32, width: f32, baseline: f32, size: f32, color: Color) {
    let text_width = measure_text(text, None, size as u16, 1.0).width;
    draw_text(text, left + (width - text_width) * 0.5, baseline, size, color);
}
