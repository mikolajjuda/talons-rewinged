mod app;
mod audio;
mod flight;
mod game;
mod input;
mod palette;
mod records;
mod render;
mod sprites;
mod ui;
mod world;

use macroquad::prelude::Conf;

fn window_conf() -> Conf {
    Conf {
        window_title: "Talons Rewinged".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    app::App::new().run().await;
}
