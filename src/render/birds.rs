use super::sprites::{CREATURE_SCALE, SpriteLibrary};
use super::view::View;
use crate::world::World;
use macroquad::models::{Mesh, draw_mesh};
use macroquad::prelude::*;
use std::f32::consts::FRAC_PI_2;

const FLAP_PERIOD: f32 = 0.2;
const BIRD_SPRITE_EXTENT: f32 = 130.0;
const BIRD_MAX_DISTANCE: f32 = 500.0;

const VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
attribute vec4 normal;

varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;
uniform float flap_t;

void main() {
    vec3 animated_pos = mix(position, normal.xyz, flap_t);
    gl_Position = Projection * Model * vec4(animated_pos, 1.0);
    color = color0 / 255.0;
}
"#;

const FRAGMENT_SHADER: &str = r#"#version 100
varying lowp vec4 color;

void main() {
    gl_FragColor = color;
}
"#;

pub struct BirdFlock {
    material: Material,
}

impl BirdFlock {
    pub fn new() -> Self {
        let material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX_SHADER,
                fragment: FRAGMENT_SHADER,
            },
            MaterialParams {
                uniforms: vec![UniformDesc::new("flap_t", UniformType::Float1)],
                pipeline_params: PipelineParams {
                    depth_write: true,
                    depth_test: Comparison::LessOrEqual,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .expect("Failed to compile bird morph material");

        Self { material }
    }

    pub fn draw(&self, sprites: &SpriteLibrary, world: &World, view: &View, limit: f32) {
        if sprites.bird_morph.is_empty() {
            return;
        }

        let radius = BIRD_SPRITE_EXTENT * CREATURE_SCALE;
        let max_distance = BIRD_MAX_DISTANCE.min(limit);

        gl_use_material(&self.material);
        self.material.set_uniform("flap_t", flap_amount());

        for bird in &world.birds {
            if view.reaches(bird.pos, radius, max_distance) {
                draw_transformed(&sprites.bird_morph, bird.pos, bird.angle, CREATURE_SCALE);
            }
        }

        gl_use_default_material();
    }
}

fn flap_amount() -> f32 {
    let phase = (get_time() as f32 / FLAP_PERIOD).fract();
    if phase < 0.5 { phase * 2.0 } else { (1.0 - phase) * 2.0 }
}

fn draw_transformed(meshes: &[Mesh], pos: Vec3, yaw: f32, scale: f32) {
    let model = Mat4::from_translation(pos)
        * Mat4::from_rotation_y(-yaw)
        * Mat4::from_rotation_x(-FRAC_PI_2)
        * Mat4::from_scale(Vec3::splat(scale));

    unsafe { get_internal_gl() }.quad_gl.push_model_matrix(model);
    for mesh in meshes {
        draw_mesh(mesh);
    }
    unsafe { get_internal_gl() }.quad_gl.pop_model_matrix();
}
