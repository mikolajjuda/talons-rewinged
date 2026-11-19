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
    let mut fullscreen = false;

    loop {
        let out = input.poll();
        let dt = get_frame_time();

        let alt_down = is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt);
        if is_key_pressed(KeyCode::F11) || (alt_down && is_key_pressed(KeyCode::Enter)) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        }

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
                    if !alt_down && is_key_pressed(KeyCode::Enter) {
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
                        let box_w = 380.0;
                        let box_h = if state.won { 108.0 } else { 82.0 };
                        let cx = (screen_width() - box_w) * 0.5;
                        let cy = (screen_height() - box_h) * 0.45;

                        let (bg, title) = if state.won {
                            (Color::new(0.12, 0.60, 0.22, 0.94), "ALL 10 FISH CAUGHT!")
                        } else {
                            (Color::new(0.85, 0.15, 0.15, 0.94), "CRASHED!")
                        };

                        draw_rectangle(cx, cy, box_w, box_h, bg);
                        draw_rectangle_lines(cx, cy, box_w, box_h, 2.0, WHITE);

                        let title_dim = measure_text(title, None, 28, 1.0);
                        draw_text(
                            title,
                            cx + (box_w - title_dim.width) * 0.5,
                            cy + 30.0,
                            28.0,
                            WHITE,
                        );

                        if state.won {
                            let time_msg = format!(
                                "Time: {:.2}s  (Best: {:.2}s)",
                                state.elapsed, state.best_time
                            );
                            let time_dim = measure_text(&time_msg, None, 18, 1.0);
                            draw_text(
                                &time_msg,
                                cx + (box_w - time_dim.width) * 0.5,
                                cy + 60.0,
                                18.0,
                                YELLOW,
                            );

                            let sub_msg = "[Enter] Restart   [Esc] Menu";
                            let sub_dim = measure_text(sub_msg, None, 16, 1.0);
                            draw_text(
                                sub_msg,
                                cx + (box_w - sub_dim.width) * 0.5,
                                cy + 90.0,
                                16.0,
                                WHITE,
                            );
                        } else {
                            let sub_msg = "[Enter] Restart   [Esc] Menu";
                            let sub_dim = measure_text(sub_msg, None, 16, 1.0);
                            draw_text(
                                sub_msg,
                                cx + (box_w - sub_dim.width) * 0.5,
                                cy + 62.0,
                                16.0,
                                WHITE,
                            );
                        }
                    }
                }
            }
        }

        next_frame().await;
    }
}
