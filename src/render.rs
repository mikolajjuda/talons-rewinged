mod assets;
mod claws;
mod palette;
mod terrain;

use crate::game::GameState;
use crate::world::World;
use macroquad::models::{Mesh, draw_mesh, draw_sphere};
use macroquad::prelude::*;

use self::assets::{RenderAssets, RenderSprite, SPRITE_SCALE, SpriteLine};
use self::palette::{COLOR_BLACK, COLOR_RED, COLOR_WHITE};
use self::terrain::{TerrainChunk, build_terrain_chunks, build_water_mesh};

const LANDSCAPE_SCALE: f32 = 2.0;

const TREE_MAX_DIST_SQ: f32 = 380.0 * 380.0;
const BOAT_MAX_DIST_SQ: f32 = 500.0 * 500.0;
const BIRD_MAX_DIST_SQ: f32 = 500.0 * 500.0;
const FISH_MAX_DIST_SQ: f32 = 350.0 * 350.0;

pub struct Renderer {
    terrain_chunks: Vec<TerrainChunk>,
    water_mesh: Mesh,
    assets: RenderAssets,
    world_seed: u64,
}

impl Renderer {
    pub fn new(world: &World) -> Self {
        Self {
            terrain_chunks: build_terrain_chunks(world),
            water_mesh: build_water_mesh(world),
            assets: RenderAssets::new(),
            world_seed: world.seed,
        }
    }

    pub fn draw(&mut self, state: &GameState) {
        let world = &state.world;
        let bird = &state.bird;

        if world.seed != self.world_seed {
            self.terrain_chunks = build_terrain_chunks(world);
            self.water_mesh = build_water_mesh(world);
            self.world_seed = world.seed;
        }

        let forward = bird.forward();
        let cam_right = bird.right();
        let cam_up = bird.up();

        let w = screen_width();
        let h = screen_height();
        let aspect = w / h.max(1.0);
        let fovy = calculate_dynamic_fovy(aspect);

        let camera = Camera3D {
            position: bird.pos,
            target: bird.pos + forward * 50.0,
            up: cam_up,
            fovy,
            projection: Projection::Perspective,
            ..Default::default()
        };

        set_camera(&camera);

        for chunk in &self.terrain_chunks {
            let to_chunk = chunk.center - bird.pos;
            if to_chunk.dot(forward) < -chunk.radius {
                continue;
            }
            draw_mesh(&chunk.mesh);
        }

        draw_mesh(&self.water_mesh);

        let fish_sprite = self.assets.get(1);
        for fish in &world.fishes {
            if fish.caught {
                continue;
            }
            let delta = fish.pos - bird.pos;
            if delta.length_squared() > FISH_MAX_DIST_SQ || delta.dot(forward) < -10.0 {
                continue;
            }

            if !self.draw_sprite_fast(1, fish_sprite, fish.pos, 0.0, 1.0, cam_right, cam_up) {
                draw_sphere(fish.pos, 2.0, None, COLOR_RED);
                let tail_dir = vec3(fish.angle.cos(), 0.0, fish.angle.sin()) * 3.0;
                draw_line_3d(fish.pos, fish.pos - tail_dir, COLOR_WHITE);
            }
        }

        for boat in &world.boats {
            let delta = boat.pos - bird.pos;
            if delta.length_squared() > BOAT_MAX_DIST_SQ || delta.dot(forward) < -20.0 {
                continue;
            }

            let sprite = self.assets.get(boat.sprite_id);
            if !self.draw_sprite_fast(
                boat.sprite_id,
                sprite,
                boat.pos,
                0.0,
                LANDSCAPE_SCALE,
                cam_right,
                cam_up,
            ) {
                draw_cube(boat.pos, vec3(4.0, 2.0, 2.0), None, COLOR_WHITE);
            }
        }

        let tree_sprite = self.assets.get(6);
        for tree in &world.trees {
            let delta = tree.pos - bird.pos;
            if delta.length_squared() > TREE_MAX_DIST_SQ || delta.dot(forward) < -20.0 {
                continue;
            }

            if !self.draw_sprite_fast(
                6,
                tree_sprite,
                tree.pos,
                0.0,
                LANDSCAPE_SCALE,
                cam_right,
                cam_up,
            ) {
                draw_line_3d(
                    tree.pos,
                    tree.pos + vec3(0.0, 2.0, 0.0),
                    Color::new(0.45, 0.25, 0.1, 1.0),
                );
                draw_sphere(
                    tree.pos + vec3(0.0, 3.0, 0.0),
                    1.5,
                    None,
                    Color::new(0.0, 0.65, 0.0, 1.0),
                );
            }
        }

        let phase = (get_time() as f32 / 0.2).fract();
        let flap_t = if phase < 0.5 {
            phase * 2.0
        } else {
            (1.0 - phase) * 2.0
        };

        let bird_a = self.assets.get(2);
        let bird_b = self.assets.get(3);
        let interpolated_meshes = build_interpolated_meshes(bird_a, bird_b, flap_t);

        for other_bird in &world.birds {
            let delta = other_bird.pos - bird.pos;
            if delta.length_squared() > BIRD_MAX_DIST_SQ || delta.dot(forward) < -20.0 {
                continue;
            }

            let mut drew = false;
            if let Some(ref meshes) = interpolated_meshes {
                draw_meshes_transformed(meshes, other_bird.pos, other_bird.angle, 1.0);
                drew = true;
            } else if !bird_a.meshes.is_empty() {
                draw_meshes_transformed(&bird_a.meshes, other_bird.pos, other_bird.angle, 1.0);
                drew = true;
            }

            if !drew {
                let wing = vec3(-other_bird.angle.sin(), 0.0, other_bird.angle.cos()) * 1.5;
                draw_line_3d(other_bird.pos - wing, other_bird.pos + wing, COLOR_WHITE);
            }
        }

        set_default_camera();

        claws::draw_claws(&self.assets, bird.claws_down);

        let cx = screen_width() * 0.5;
        let cy = screen_height() * 0.5;

        draw_line(cx - 6.0, cy, cx + 6.0, cy, 2.0, COLOR_BLACK);
        draw_line(cx, cy - 6.0, cx, cy + 6.0, 2.0, COLOR_BLACK);

        draw_hud(state);
    }

    fn draw_sprite_fast(
        &self,
        _sprite_id: u32,
        sprite: &RenderSprite,
        pos: Vec3,
        yaw: f32,
        scale: f32,
        cam_right: Vec3,
        cam_up: Vec3,
    ) -> bool {
        let mut drew = false;

        if !sprite.meshes.is_empty() {
            draw_meshes_transformed(&sprite.meshes, pos, yaw, scale);
            drew = true;
        }

        if !sprite.lines.is_empty() {
            draw_lines_3d_fast(&sprite.lines, pos, scale, cam_right, cam_up);
            drew = true;
        }

        drew
    }
}

fn calculate_dynamic_fovy(aspect: f32) -> f32 {
    const DIAGONAL_FACTOR: f32 = 1.482;
    let aspect = aspect.max(0.1);
    let half_fovy_tan = DIAGONAL_FACTOR / (1.0 + aspect * aspect).sqrt();
    2.0 * half_fovy_tan.atan()
}

fn draw_hud(state: &GameState) {
    let bird = &state.bird;
    let forward = bird.forward();
    let right = bird.right();
    let up = bird.up();

    let pitch_deg = forward.y.clamp(-1.0, 1.0).asin().to_degrees();
    let roll_deg = (-right.y).atan2(up.y).to_degrees();
    let mut heading_deg = forward.x.atan2(-forward.z).to_degrees();
    if heading_deg < 0.0 {
        heading_deg += 360.0;
    }

    let ground = state.world.get_height(bird.pos.x, bird.pos.z);
    let height = (bird.pos.y - ground).round() as i32;

    let line1 = format!(
        "Pitch:{:5.1} Roll:{:5.1} Heading:{:5.1} Height:{:5}",
        pitch_deg, roll_deg, heading_deg, height
    );
    let line2 = format!(
        "Fish Remaining:{} Time:{:3.2} Best:{:3.2}",
        state.fish_remaining(),
        state.elapsed,
        state.best_time
    );

    let font_size = 20.0;
    draw_text(&line1, 8.0, 20.0, font_size, COLOR_BLACK);
    draw_text(&line2, 8.0, 42.0, font_size, COLOR_BLACK);

    let fps_text = format!("FPS: {}", get_fps());
    let fps_dims = measure_text(&fps_text, None, font_size as u16, 1.0);
    draw_text(
        &fps_text,
        screen_width() - fps_dims.width - 10.0,
        20.0,
        font_size,
        COLOR_BLACK,
    );

    if state.alive && !state.won && state.elapsed < 5.0 {
        let blink_on = ((get_time() * 2.0) as u32) % 2 == 0;
        if blink_on {
            let msg = "Catch 10 Fish";
            let prompt_size = 28.0;
            let dims = measure_text(msg, None, prompt_size as u16, 1.0);
            draw_text(
                msg,
                (screen_width() - dims.width) * 0.5,
                screen_height() * 0.5 - 64.0,
                prompt_size,
                COLOR_WHITE,
            );
        }
    }
}

fn model_matrix(pos: Vec3, yaw: f32, scale: f32) -> Mat4 {
    Mat4::from_translation(pos)
        * Mat4::from_rotation_y(-yaw)
        * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2)
        * Mat4::from_scale(Vec3::splat(scale))
}

fn draw_meshes_transformed(meshes: &[Mesh], pos: Vec3, yaw: f32, scale: f32) {
    let model = model_matrix(pos, yaw, scale);
    unsafe { get_internal_gl() }
        .quad_gl
        .push_model_matrix(model);
    for mesh in meshes {
        draw_mesh(mesh);
    }
    unsafe { get_internal_gl() }.quad_gl.pop_model_matrix();
}

fn draw_lines_3d_fast(lines: &[SpriteLine], pos: Vec3, scale: f32, cam_right: Vec3, cam_up: Vec3) {
    let s = SPRITE_SCALE * scale;
    for line in lines {
        let p1 = pos + cam_right * (line.x1 as f32 * s) - cam_up * (line.y1 as f32 * s);
        let p2 = pos + cam_right * (line.x2 as f32 * s) - cam_up * (line.y2 as f32 * s);
        draw_line_3d(p1, p2, line.color);
    }
}

fn build_interpolated_meshes(a: &RenderSprite, b: &RenderSprite, t: f32) -> Option<Vec<Mesh>> {
    if a.meshes.is_empty() || a.meshes.len() != b.meshes.len() {
        return None;
    }

    let mut out = Vec::with_capacity(a.meshes.len());

    for (ma, mb) in a.meshes.iter().zip(&b.meshes) {
        if ma.vertices.len() != mb.vertices.len() || ma.indices != mb.indices {
            return None;
        }

        let vertices = ma
            .vertices
            .iter()
            .zip(&mb.vertices)
            .map(|(va, vb)| {
                let mut v = *va;
                v.position = va.position.lerp(vb.position, t);
                v
            })
            .collect();

        out.push(Mesh {
            vertices,
            indices: ma.indices.clone(),
            texture: ma.texture.clone(),
        });
    }

    Some(out)
}
