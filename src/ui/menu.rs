use super::theme::*;
use crate::game::Mode;
use macroquad::prelude::*;

const PANEL_W: f32 = 340.0;
const PANEL_H: f32 = 280.0;
const BUTTON_W: f32 = 220.0;
const BUTTON_H: f32 = 36.0;
const BUTTON_STEP: f32 = 46.0;

#[derive(Clone, Copy)]
pub enum MenuChoice {
    Play(Mode),
    InputTest,
}

struct Entry {
    label: &'static str,
    choice: MenuChoice,
}

const ENTRIES: [Entry; 4] = [
    Entry {
        label: "Classic Game",
        choice: MenuChoice::Play(Mode::Classic),
    },
    Entry {
        label: "Bird Physics",
        choice: MenuChoice::Play(Mode::Physics),
    },
    Entry {
        label: "Drone Acro (FPV)",
        choice: MenuChoice::Play(Mode::Drone),
    },
    Entry {
        label: "Input Test",
        choice: MenuChoice::InputTest,
    },
];

pub fn draw_main_menu() -> Option<MenuChoice> {
    let left = (screen_width() - PANEL_W) * 0.5;
    let top = (screen_height() - PANEL_H) * 0.35;

    draw_rectangle(left, top, PANEL_W, PANEL_H, PANEL_BG);
    draw_rectangle_lines(left, top, PANEL_W, PANEL_H, 3.0, PANEL_BORDER);
    draw_text("Talons Rewinged", left + 48.0, top + 38.0, 32.0, DARK_GREEN);

    let mut selected = None;
    for (row, entry) in ENTRIES.iter().enumerate() {
        let rect = Rect::new(
            left + (PANEL_W - BUTTON_W) * 0.5,
            top + 58.0 + BUTTON_STEP * row as f32,
            BUTTON_W,
            BUTTON_H,
        );
        if draw_button(rect, entry.label, false) {
            selected = Some(entry.choice);
        }
    }

    selected
}
