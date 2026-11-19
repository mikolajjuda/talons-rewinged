use std::collections::HashMap;

use crate::sprites_parser::{MeshTriangle, Sprite, SpriteElementData};

use macroquad::models::{Mesh, Vertex};
use macroquad::prelude::*;

use super::palette::{palette_color, scale_color};

pub(super) const SPRITE_SCALE: f32 = 0.05;
const SPRITE_AMBIENT: f32 = 0.35;
const SPRITE_DIFFUSE: f32 = 0.65;
const SPRITE_LIGHT_DIRECTION: Vec3 = Vec3::new(-0.4, 0.8, -0.5);

pub struct RenderAssets {
    pub sprites: HashMap<u32, RenderSprite>,
    /// Single static mesh where vertex.position = BI=2 (up) and vertex.normal.xyz = BI=3 (down)
    pub morph_bird_meshes: Vec<Mesh>,
}

pub struct RenderSprite {
    pub meshes: Vec<Mesh>,
    pub bitmaps: Vec<BitmapTexture>,
    pub lines: Vec<SpriteLine>,
}

pub struct BitmapTexture {
    pub texture: Texture2D,
    pub x: i32,
    pub y: i32,
}

pub struct SpriteLine {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub color: Color,
    pub thickness: f32,
}

impl RenderAssets {
    pub fn new() -> Self {
        let parsed = crate::sprites_parser::parse_talons(include_bytes!("../../assets/Talons.HC"))
            .expect("Failed to parse embedded Talons.HC");

        let sprites: HashMap<u32, RenderSprite> = parsed
            .into_iter()
            .map(|(bi, sprite)| (bi, RenderSprite::from_sprite(&sprite)))
            .collect();

        // Build continuous morph target mesh: BI=2 (glide/up) -> BI=3 (flap/down)
        let morph_bird_meshes = if let (Some(a), Some(b)) = (sprites.get(&2), sprites.get(&3)) {
            build_morph_meshes(a, b)
        } else {
            Vec::new()
        };

        Self {
            sprites,
            morph_bird_meshes,
        }
    }

    pub fn get(&self, bi: u32) -> &RenderSprite {
        self.sprites
            .get(&bi)
            .unwrap_or_else(|| panic!("Missing Talons sprite BI={bi}"))
    }
}

impl RenderSprite {
    fn from_sprite(sprite: &Sprite) -> Self {
        let mut result = Self {
            meshes: Vec::new(),
            bitmaps: Vec::new(),
            lines: Vec::new(),
        };

        let mut current_color = 15u8;
        let mut current_thickness = 1.0f32;
        let mut shift_x = 0i32;
        let mut shift_y = 0i32;

        for element in &sprite.elements {
            match &element.data {
                SpriteElementData::Color { color } => {
                    current_color = *color;
                }
                SpriteElementData::Thick { thickness } => {
                    current_thickness = (*thickness).max(1) as f32;
                }
                SpriteElementData::Shift { x, y } => {
                    shift_x = shift_x.saturating_add(*x);
                    shift_y = shift_y.saturating_add(*y);
                }
                SpriteElementData::Line { x1, y1, x2, y2 } => {
                    result.lines.push(SpriteLine {
                        x1: x1 + shift_x,
                        y1: y1 + shift_y,
                        x2: x2 + shift_x,
                        y2: y2 + shift_y,
                        color: palette_color(current_color as i32),
                        thickness: current_thickness,
                    });
                }
                SpriteElementData::Polyline { points } => {
                    let color = palette_color(current_color as i32);
                    for pair in points.windows(2) {
                        result.lines.push(SpriteLine {
                            x1: pair[0].0 + shift_x,
                            y1: pair[0].1 + shift_y,
                            x2: pair[1].0 + shift_x,
                            y2: pair[1].1 + shift_y,
                            color,
                            thickness: current_thickness,
                        });
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
                    if let Some(bitmap) = make_bitmap(
                        x + shift_x,
                        y + shift_y,
                        *width,
                        *height,
                        *row_stride,
                        pixels,
                    ) {
                        result.bitmaps.push(bitmap);
                    }
                }
                SpriteElementData::Mesh {
                    vertices,
                    triangles,
                } => {
                    result
                        .meshes
                        .push(make_mesh(vertices, triangles, (0, 0, 0)));
                }
                SpriteElementData::ShiftableMesh {
                    x,
                    y,
                    z,
                    vertices,
                    triangles,
                } => {
                    result
                        .meshes
                        .push(make_mesh(vertices, triangles, (*x, *y, *z)));
                }
                _ => {}
            }
        }

        result
    }

    pub fn draw_2d(&self, x: f32, y: f32, scale: f32, flip_x: bool) {
        self.draw_lines_2d(x, y, scale, flip_x);
        self.draw_bitmaps_2d(x, y, scale, flip_x);
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

    pub fn draw_lines_2d(&self, x: f32, y: f32, scale: f32, flip_x: bool) {
        let sign = if flip_x { -1.0 } else { 1.0 };

        for line in &self.lines {
            draw_line(
                x + line.x1 as f32 * scale * sign,
                y + line.y1 as f32 * scale,
                x + line.x2 as f32 * scale * sign,
                y + line.y2 as f32 * scale,
                (line.thickness * scale).max(1.0),
                line.color,
            );
        }
    }
}

fn make_mesh(
    vertices: &[(i32, i32, i32)],
    triangles: &[MeshTriangle],
    offset: (i32, i32, i32),
) -> Mesh {
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

        let pa = sprite_position(vertices[a], offset);
        let pb = sprite_position(vertices[b], offset);
        let pc = sprite_position(vertices[c], offset);

        let normal = (pb - pa).cross(pc - pa).normalize_or_zero();
        let diffuse = normal.dot(light_direction).max(0.0);
        let mut brightness = SPRITE_AMBIENT + SPRITE_DIFFUSE * diffuse;

        if triangle.color & 8 != 0 {
            brightness = (brightness * 1.15).min(1.0);
        }

        let color = scale_color(palette_color(triangle.color), brightness);
        let base = mesh_vertices.len();

        if base + 3 > u16::MAX as usize {
            break;
        }

        for position in [pa, pb, pc] {
            mesh_vertices.push(Vertex::new2(position, vec2(0.0, 0.0), color));
        }

        indices.push(base as u16);
        indices.push((base + 1) as u16);
        indices.push((base + 2) as u16);
    }

    Mesh {
        vertices: mesh_vertices,
        indices,
        texture: None,
    }
}

fn build_morph_meshes(a: &RenderSprite, b: &RenderSprite) -> Vec<Mesh> {
    let mut out = Vec::new();
    for (ma, mb) in a.meshes.iter().zip(&b.meshes) {
        if ma.vertices.len() != mb.vertices.len() || ma.indices != mb.indices {
            continue;
        }

        // Store base frame position in `position`, and target frame position in `normal.xyz`
        let vertices = ma
            .vertices
            .iter()
            .zip(&mb.vertices)
            .map(|(va, vb)| {
                let mut v = *va;
                v.normal = vec4(vb.position.x, vb.position.y, vb.position.z, 0.0);
                v
            })
            .collect();

        out.push(Mesh {
            vertices,
            indices: ma.indices.clone(),
            texture: None,
        });
    }
    out
}

fn sprite_position(vertex: (i32, i32, i32), offset: (i32, i32, i32)) -> Vec3 {
    vec3(
        (vertex.0 + offset.0) as f32 * SPRITE_SCALE,
        (vertex.1 + offset.1) as f32 * SPRITE_SCALE,
        (vertex.2 + offset.2) as f32 * SPRITE_SCALE,
    )
}

fn make_bitmap(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    row_stride: usize,
    pixels: &[u8],
) -> Option<BitmapTexture> {
    if width <= 0 || height <= 0 {
        return None;
    }

    let width = width as usize;
    let height = height as usize;

    if width > u16::MAX as usize || height > u16::MAX as usize {
        return None;
    }

    if row_stride < width {
        return None;
    }

    let required_size = row_stride.checked_mul(height)?;

    if pixels.len() < required_size {
        return None;
    }

    let mut rgba = Vec::with_capacity(width.checked_mul(height)?.checked_mul(4)?);

    for row in 0..height {
        for col in 0..width {
            let pixel = pixels[row * row_stride + col];

            if pixel == 0xFF {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let color = palette_color((pixel & 0x0F) as i32);

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
