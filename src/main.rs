mod audio;
mod flight;
mod game;
mod input;
mod menu;
mod render;
mod sprites_parser;
mod world;

use audio::AudioSystem;
use game::GameState;
use input::InputSystem;
use macroquad::prelude::*;
use menu::{SKY_BLUE, draw_input_test, draw_main_menu};
use render::Renderer;

const TEMPLE_SKY: Color = Color::new(0.33, 1.0, 1.0, 1.0);

pub enum Screen {
    MainMenu,
    InputTest,
    Game(GameState),
}

#[macroquad::main("Talons Rewinged")]
async fn main() {
    let mut input = InputSystem::new("input.json");
    let mut screen = Screen::MainMenu;
    let dummy_world = world::World::new(1);
    let mut renderer = Renderer::new(&dummy_world);
    let mut audio = AudioSystem::new();

    loop {
        let out = input.poll();
        let dt = get_frame_time();

        if is_key_pressed(KeyCode::M) {
            audio.toggle_mute();
        }

        match screen {
            Screen::MainMenu => {
                audio.stop_music();
                clear_background(SKY_BLUE);
                draw_main_menu(&mut screen);
            }
            Screen::InputTest => {
                audio.stop_music();
                clear_background(SKY_BLUE);
                draw_input_test(&mut input, &out, &mut screen);
            }
            Screen::Game(ref mut state) => {
                clear_background(TEMPLE_SKY);

                if is_key_pressed(KeyCode::Escape) {
                    audio.stop_music();
                    screen = Screen::MainMenu;
                } else {
                    if is_key_pressed(KeyCode::Enter) {
                        state.reset();
                    }

                    let was_alive = state.alive;
                    let prev_score = state.score;

                    state.update(&out, dt);

                    if state.score > prev_score {
                        audio.play_catch();
                    }
                    if was_alive && !state.alive {
                        audio.stop_music();
                        audio.play_crash();
                    } else if state.alive && !state.won {
                        audio.ensure_music_playing();
                    } else {
                        audio.stop_music();
                    }

                    renderer.draw(state);

                    if !state.alive || state.won {
                        let cx = screen_width() * 0.5 - 160.0;
                        let cy = screen_height() * 0.5 - 30.0;
                        let (bg, title) = if state.won {
                            (Color::new(0.15, 0.65, 0.25, 0.9), "ALL 10 FISH CAUGHT!")
                        } else {
                            (Color::new(0.85, 0.15, 0.15, 0.9), "CRASHED!")
                        };
                        let dims = measure_text(title, None, 30, 1.0);
                        draw_rectangle(cx - 20.0, cy - 26.0, 360.0, 80.0, bg);
                        draw_text(
                            title,
                            cx - 20.0 + (360.0 - dims.width) * 0.5,
                            cy + 10.0,
                            30.0,
                            WHITE,
                        );
                        draw_text(
                            "[Enter] Restart   [Esc] Menu",
                            cx + 30.0,
                            cy + 40.0,
                            18.0,
                            WHITE,
                        );
                    }
                }
            }
        }

        next_frame().await;
    }
}
