mod flight;
mod game;
mod input;
mod menu;
mod render;
mod world;

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

    loop {
        let out = input.poll();
        let dt = get_frame_time();

        match screen {
            Screen::MainMenu => {
                clear_background(SKY_BLUE);
                draw_main_menu(&mut screen);
            }
            Screen::InputTest => {
                clear_background(SKY_BLUE);
                draw_input_test(&mut input, &out, &mut screen);
            }
            Screen::Game(ref mut state) => {
                clear_background(TEMPLE_SKY);

                if is_key_pressed(KeyCode::Escape) {
                    screen = Screen::MainMenu;
                } else {
                    if is_key_pressed(KeyCode::Enter) {
                        state.reset();
                    }

                    state.update(&out, dt);
                    renderer.draw(&state.world, &state.bird);

                    if !state.alive {
                        let cx = screen_width() * 0.5 - 160.0;
                        let cy = screen_height() * 0.5 - 30.0;
                        draw_rectangle(
                            cx - 20.0,
                            cy - 26.0,
                            360.0,
                            80.0,
                            Color::new(0.85, 0.15, 0.15, 0.9),
                        );
                        draw_text("CRASHED!", cx + 80.0, cy + 10.0, 36.0, WHITE);
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
