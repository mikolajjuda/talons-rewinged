use crate::audio::AudioSystem;
use crate::game::{Events, Game, Mode};
use crate::input::{GameInputs, InputSystem};
use crate::records::BestTimes;
use crate::render::Renderer;
use crate::ui::{MenuChoice, SKY_BLUE, draw_input_test, draw_main_menu};
use macroquad::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

const INPUT_CONFIG_PATH: &str = "input.json";
const RECORDS_PATH: &str = "best_times.txt";
const MAX_STEP: f32 = 0.1;

enum Screen {
    Menu,
    InputTest,
    Loading { mode: Mode, shown: bool },
    Playing(Box<Game>),
}

pub struct App {
    input: InputSystem,
    audio: AudioSystem,
    renderer: Renderer,
    records: BestTimes,
    screen: Screen,
    show_fps: bool,
    fullscreen: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: InputSystem::new(INPUT_CONFIG_PATH),
            audio: AudioSystem::new(),
            renderer: Renderer::new(),
            records: BestTimes::load(RECORDS_PATH),
            screen: Screen::Menu,
            show_fps: true,
            fullscreen: false,
        }
    }

    pub async fn run(mut self) {
        loop {
            self.frame();
            next_frame().await;
            self.renderer.observe_frame(get_frame_time());
        }
    }

    fn frame(&mut self) {
        let inputs = self.input.poll();
        self.handle_global_keys();

        let screen = std::mem::replace(&mut self.screen, Screen::Menu);
        self.screen = match screen {
            Screen::Menu => self.menu_frame(),
            Screen::InputTest => self.input_test_frame(&inputs),
            Screen::Loading { mode, shown } => self.loading_frame(mode, shown),
            Screen::Playing(game) => self.playing_frame(game, &inputs),
        };
    }

    fn handle_global_keys(&mut self) {
        if is_key_pressed(KeyCode::F11) || (alt_down() && is_key_pressed(KeyCode::Enter)) {
            self.fullscreen = !self.fullscreen;
            set_fullscreen(self.fullscreen);
        }
        if is_key_pressed(KeyCode::M) {
            self.audio.toggle_mute();
        }
        if is_key_pressed(KeyCode::F) {
            self.show_fps = !self.show_fps;
        }
    }

    fn menu_frame(&mut self) -> Screen {
        self.audio.stop_music();
        clear_background(SKY_BLUE);

        match draw_main_menu() {
            Some(MenuChoice::Play(mode)) => Screen::Loading { mode, shown: false },
            Some(MenuChoice::InputTest) => Screen::InputTest,
            None => {
                if is_key_pressed(KeyCode::Escape) {
                    std::process::exit(0);
                }
                Screen::Menu
            }
        }
    }

    fn input_test_frame(&mut self, inputs: &GameInputs) -> Screen {
        self.audio.stop_music();
        clear_background(SKY_BLUE);

        if draw_input_test(&mut self.input, inputs) {
            Screen::Menu
        } else {
            Screen::InputTest
        }
    }

    fn loading_frame(&mut self, mode: Mode, shown: bool) -> Screen {
        if !shown {
            clear_background(SKY_BLUE);
            return Screen::Loading { mode, shown: true };
        }

        let game = Game::new(mode, self.next_seed(), self.records.best(mode));
        Screen::Playing(Box::new(game))
    }

    fn playing_frame(&mut self, mut game: Box<Game>, inputs: &GameInputs) -> Screen {
        if is_key_pressed(KeyCode::Escape) {
            self.audio.stop_music();
            return Screen::Menu;
        }
        if !alt_down() && is_key_pressed(KeyCode::Enter) {
            game.restart();
        }

        let events = game.update(inputs, get_frame_time().min(MAX_STEP));
        self.react(&game, &events);
        self.renderer.draw(&game, self.show_fps);

        Screen::Playing(game)
    }

    fn react(&mut self, game: &Game, events: &Events) {
        self.audio.ensure_music_playing();

        if events.fish_caught {
            self.audio.play_catch();
        }
        if events.crashed {
            self.audio.play_crash();
        }
        if events.won {
            self.records.record(game.mode(), game.best_time());
        }
    }

    fn next_seed(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos() as u64)
            .unwrap_or(1)
    }
}

fn alt_down() -> bool {
    is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt)
}
