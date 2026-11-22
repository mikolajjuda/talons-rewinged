mod birds;
mod chunks;
mod claws;
mod fish;
mod governor;
mod horizon;
mod hud;
mod sprites;
mod terrain;
mod view;

use crate::game::Game;
use crate::palette;
use crate::sprites::id;
use crate::world::World;
use birds::BirdFlock;
use chunks::{ChunkSet, Tiler};
use governor::Governor;
use macroquad::prelude::*;
use sprites::{LANDSCAPE_SCALE, SpriteLibrary};
use view::View;

const TERRAIN_MAX_DISTANCE: f32 = 2200.0;
const TREE_MAX_DISTANCE: f32 = 380.0;
const BOAT_MAX_DISTANCE: f32 = 500.0;
const PROP_TILE_EXTENT: f32 = 96.0;

struct Scene {
    revision: u64,
    terrain: ChunkSet,
    trees: ChunkSet,
    boats: ChunkSet,
}

impl Scene {
    fn build(world: &World, sprites: &SpriteLibrary) -> Self {
        let mut trees = Tiler::new(PROP_TILE_EXTENT);
        for tree in &world.trees {
            sprites.bake(&mut trees, id::TREE, tree.pos, LANDSCAPE_SCALE);
        }

        let mut boats = Tiler::new(PROP_TILE_EXTENT);
        for boat in &world.boats {
            sprites.bake(&mut boats, boat.sprite, boat.pos, LANDSCAPE_SCALE);
        }

        Self {
            revision: world.revision(),
            terrain: terrain::build(world),
            trees: trees.finish(),
            boats: boats.finish(),
        }
    }
}

pub struct Renderer {
    sprites: SpriteLibrary,
    birds: BirdFlock,
    governor: Governor,
    scene: Option<Scene>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            sprites: SpriteLibrary::load(),
            birds: BirdFlock::new(),
            governor: Governor::new(),
            scene: None,
        }
    }

    pub fn observe_frame(&mut self, frame_time: f32) {
        self.governor.observe(frame_time);
    }

    pub fn draw(&mut self, game: &Game, show_fps: bool) {
        self.ensure_scene(game.world());

        set_default_camera();
        let view = View::new(game.bird());

        draw_background(game, &view);
        self.draw_world(game, &view);

        set_default_camera();
        claws::draw_claws(&self.sprites, game.claws_down());
        hud::draw(game, show_fps);
    }

    fn ensure_scene(&mut self, world: &World) {
        let current = self
            .scene
            .as_ref()
            .is_some_and(|scene| scene.revision == world.revision());

        if !current {
            self.scene = Some(Scene::build(world, &self.sprites));
        }
    }

    fn draw_world(&self, game: &Game, view: &View) {
        let Some(scene) = self.scene.as_ref() else {
            return;
        };
        let world = game.world();
        let terrain_reach = TERRAIN_MAX_DISTANCE * self.governor.scale();

        set_camera(&view.camera());

        scene.terrain.draw(view, terrain_reach);
        scene.boats.draw(view, BOAT_MAX_DISTANCE.min(terrain_reach));
        scene.trees.draw(view, TREE_MAX_DISTANCE.min(terrain_reach));

        fish::draw_fish(self.sprites.get(id::FISH), world, view, terrain_reach);
        self.birds.draw(&self.sprites, world, view, terrain_reach);
    }
}

fn draw_background(game: &Game, view: &View) {
    clear_background(palette::color(palette::LTCYAN as i32));

    let world = game.world();
    let ground = if game.bird().pos.y < world.shallow_ceiling() {
        palette::BLUE
    } else {
        palette::LTGREEN
    };
    horizon::draw_ground(view, palette::color(ground as i32));
}
