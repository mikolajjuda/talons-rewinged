use super::chunks::Tiler;
use crate::palette;
use crate::world::HORIZONTAL_UNIT;
use crate::sprites::{MeshTriangle, Sprite, SpriteElementData, id, parse_talons};
use macroquad::models::{Mesh, Vertex};
use macroquad::prelude::*;
use std::collections::HashMap;

pub const CREATURE_SCALE: f32 = HORIZONTAL_UNIT;
pub const LANDSCAPE_SCALE: f32 = 10.0 * HORIZONTAL_UNIT;

const SPRITE_AMBIENT: f32 = 0.35;
const SPRITE_DIFFUSE: f32 = 0.65;
const SPRITE_LIGHT_DIRECTION: Vec3 = Vec3::new(-0.4, 0.8, -0.5);
const BRIGHT_BOOST: f32 = 1.15;
const DEFAULT_COLOR: u8 = 15;

pub struct SpriteLine {
    pub from: Vec2,
    pub to: Vec2,
    pub color: Color,
}

pub struct BitmapTexture {
    pub texture: Texture2D,
    pub x: i32,
    pub y: i32,
}

pub struct RenderSprite {
    pub meshes: Vec<Mesh>,
    pub bitmaps: Vec<BitmapTexture>,
    pub lines: Vec<SpriteLine>,
}

pub struct SpriteLibrary {
    sprites: HashMap<u32, RenderSprite>,
    pub bird_morph: Vec<Mesh>,
}

impl SpriteLibrary {
    pub fn load() -> Self {
        let parsed = parse_talons(include_bytes!("../../assets/Talons.HC"))
            .expect("Failed to parse embedded Talons.HC");

        let sprites: HashMap<u32, RenderSprite> = parsed
            .into_iter()
            .filter(|(sprite_id, _)| id::REQUIRED.contains(sprite_id))
            .map(|(sprite_id, sprite)| (sprite_id, RenderSprite::from_sprite(&sprite)))
            .collect();

        let bird_morph = match (sprites.get(&id::BIRD_GLIDE), sprites.get(&id::BIRD_FLAP)) {
            (Some(glide), Some(flap)) => build_morph_meshes(glide, flap),
            _ => Vec::new(),
        };

        Self {
            sprites,
            bird_morph,
        }
    }

    pub fn get(&self, sprite_id: u32) -> &RenderSprite {
        self.sprites
            .get(&sprite_id)
            .unwrap_or_else(|| panic!("Missing Talons sprite BI={sprite_id}"))
    }

    pub fn bake(&self, tiler: &mut Tiler, sprite_id: u32, origin: Vec3, scale: f32) {
        for mesh in &self.get(sprite_id).meshes {
            let vertices: Vec<Vertex> = mesh
                .vertices
                .iter()
                .map(|vertex| {
                    let mut placed = *vertex;
                    placed.position = origin + vec3(vertex.position.x, vertex.position.z, -vertex.position.y) * scale;
                    placed
                })
                .collect();
            tiler.push(origin, &vertices, &mesh.indices);
        }
    }
}

impl RenderSprite {
    fn from_sprite(sprite: &Sprite) -> Self {
        let mut result = Self {
            meshes: Vec::new(),
            bitmaps: Vec::new(),
            lines: Vec::new(),
        };

        let mut current_color = DEFAULT_COLOR;
        let mut shift_x = 0i32;
        let mut shift_y = 0i32;

        for element in &sprite.elements {
            match element {
                SpriteElementData::Color { color } => current_color = *color,
                SpriteElementData::Shift { x, y } => {
                    shift_x = shift_x.saturating_add(*x);
                    shift_y = shift_y.saturating_add(*y);
                }
                SpriteElementData::Line { x1, y1, x2, y2 } => {
                    let color = palette::color(current_color as i32);
                    result.lines.push(line(x1 + shift_x, y1 + shift_y, x2 + shift_x, y2 + shift_y, color));
                }
                SpriteElementData::Polyline { points } => {
                    let color = palette::color(current_color as i32);
                    for pair in points.windows(2) {
                        result.lines.push(line(
                            pair[0].0 + shift_x,
                            pair[0].1 + shift_y,
                            pair[1].0 + shift_x,
                            pair[1].1 + shift_y,
                            color,
                        ));
                    }
                }
                SpriteElementData::Bitmap {
                    x,
                    y,
                    width,
                    height,
                    row_stride,
                    pixels,
                } => {
                    if let Some(bitmap) = make_bitmap(x + shift_x, y + shift_y, *width, *height, *row_stride, pixels) {
                        result.bitmaps.push(bitmap);
                    }
                }
                SpriteElementData::Mesh { vertices, triangles } => {
                    result.meshes.push(make_mesh(vertices, triangles, (0, 0, 0)));
                }
                SpriteElementData::ShiftableMesh {
                    x,
                    y,
                    z,
                    vertices,
                    triangles,
                } => {
                    result.meshes.push(make_mesh(vertices, triangles, (*x, *y, *z)));
                }
                _ => {}
            }
        }

        result
    }

    pub fn draw_bitmaps_2d(&self, x: f32, y: f32, scale: f32, flip_x: bool) {
        for bitmap in &self.bitmaps {
            let width = bitmap.texture.width() * scale;
            let height = bitmap.texture.height() * scale;
            let draw_x = if flip_x {
                x - bitmap.x as f32 * scale - width
            } else {
                x + bitmap.x as f32 * scale
            };

            draw_texture_ex(
                &bitmap.texture,
                draw_x,
                y + bitmap.y as f32 * scale,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(width, height)),
                    flip_x,
                    ..Default::default()
                },
            );
        }
    }
}

fn line(x1: i32, y1: i32, x2: i32, y2: i32, color: Color) -> SpriteLine {
    SpriteLine {
        from: vec2(x1 as f32, y1 as f32),
        to: vec2(x2 as f32, y2 as f32),
        color,
    }
}

fn make_mesh(vertices: &[(i32, i32, i32)], triangles: &[MeshTriangle], offset: (i32, i32, i32)) -> Mesh {
    let mut mesh_vertices = Vec::with_capacity(triangles.len() * 3);
    let mut indices = Vec::with_capacity(triangles.len() * 3);
    let light_direction = SPRITE_LIGHT_DIRECTION.normalize();

    for triangle in triangles {
        let [a, b, c] = triangle.indices;
        if a < 0 || b < 0 || c < 0 {
            continue;
        }

        let (a, b, c) = (a as usize, b as usize, c as usize);
        if a >= vertices.len() || b >= vertices.len() || c >= vertices.len() {
            continue;
        }

        let base = mesh_vertices.len();
        if base + 3 > u16::MAX as usize {
            break;
        }

        let pa = sprite_position(vertices[a], offset);
        let pb = sprite_position(vertices[b], offset);
        let pc = sprite_position(vertices[c], offset);

        let normal = (pb - pa).cross(pc - pa).normalize_or_zero();
        let diffuse = normal.dot(light_direction).max(0.0);
        let mut brightness = SPRITE_AMBIENT + SPRITE_DIFFUSE * diffuse;
        if triangle.color & 8 != 0 {
            brightness = (brightness * BRIGHT_BOOST).min(1.0);
        }

        let color = palette::shade(palette::color(triangle.color), brightness);
        for position in [pa, pb, pc] {
            mesh_vertices.push(Vertex::new2(position, Vec2::ZERO, color));
        }
        indices.extend_from_slice(&[base as u16, (base + 1) as u16, (base + 2) as u16]);
    }

    Mesh {
        vertices: mesh_vertices,
        indices,
        texture: None,
    }
}

fn sprite_position(vertex: (i32, i32, i32), offset: (i32, i32, i32)) -> Vec3 {
    vec3(
        (vertex.0 + offset.0) as f32,
        (vertex.1 + offset.1) as f32,
        (vertex.2 + offset.2) as f32,
    )
}

fn build_morph_meshes(glide: &RenderSprite, flap: &RenderSprite) -> Vec<Mesh> {
    let mut morphs = Vec::new();

    for (a, b) in glide.meshes.iter().zip(&flap.meshes) {
        if a.vertices.len() != b.vertices.len() || a.indices != b.indices {
            continue;
        }

        let vertices = a
            .vertices
            .iter()
            .zip(&b.vertices)
            .map(|(from, to)| {
                let mut vertex = *from;
                vertex.normal = vec4(to.position.x, to.position.y, to.position.z, 0.0);
                vertex
            })
            .collect();

        morphs.push(Mesh {
            vertices,
            indices: a.indices.clone(),
            texture: None,
        });
    }

    morphs
}

fn make_bitmap(x: i32, y: i32, width: i32, height: i32, row_stride: usize, pixels: &[u8]) -> Option<BitmapTexture> {
    if width <= 0 || height <= 0 {
        return None;
    }

    let width = width as usize;
    let height = height as usize;
    if width > u16::MAX as usize || height > u16::MAX as usize || row_stride < width {
        return None;
    }
    if pixels.len() < row_stride.checked_mul(height)? {
        return None;
    }

    let mut rgba = Vec::with_capacity(width * height * 4);
    for row in 0..height {
        for col in 0..width {
            let pixel = pixels[row * row_stride + col];
            if pixel == 0xFF {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let color = palette::color((pixel & 0x0F) as i32);
                rgba.push((color.r * 255.0).round() as u8);
                rgba.push((color.g * 255.0).round() as u8);
                rgba.push((color.b * 255.0).round() as u8);
                rgba.push(255);
            }
        }
    }

    let texture = Texture2D::from_rgba8(width as u16, height as u16, &rgba);
    texture.set_filter(FilterMode::Nearest);
    Some(BitmapTexture { texture, x, y })
}
