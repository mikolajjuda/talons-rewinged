use super::rng::Rng;
use super::{MAP_HEIGHT, MAP_WIDTH, ROCK_ELEVATION, SNOW_ELEVATION, WATER_ELEVATION};
use crate::palette;
use crate::sprites::id;
use macroquad::prelude::{Vec3, vec3};

const PANEL_SIZE_MAX: usize = 16;
const MAX_THRESHOLD: usize = 8;
const SAME_SLOPE_NORM_SQR: f32 = 0.10;
const EDGE_TOLERANCE: f32 = 30.0;
const WATER_PROP_CHANCE: f32 = 0.05;
const BOAT_SHARE: f32 = 0.1;
const TREE_CHANCE: f32 = 0.03;

#[derive(Clone, Copy)]
pub struct PanelColor {
    pub primary: u8,
    pub secondary: Option<u8>,
}

impl PanelColor {
    fn solid(index: u8) -> Self {
        Self {
            primary: index,
            secondary: None,
        }
    }

    fn dither(first: u8, second: u8) -> Self {
        Self {
            primary: first,
            secondary: Some(second),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Panel {
    pub x: u16,
    pub y: u16,
    pub w: u8,
    pub h: u8,
    pub water: bool,
    pub color: PanelColor,
}

#[derive(Clone, Copy)]
pub enum PropKind {
    Fish,
    Boat(u32),
    Tree,
}

#[derive(Clone, Copy)]
pub struct Prop {
    pub kind: PropKind,
    pub cell_x: f32,
    pub cell_y: f32,
}

pub struct Layout {
    pub panels: Vec<Panel>,
    pub props: Vec<Prop>,
}

pub fn build(elevations: &[i16], rng: &mut Rng) -> Layout {
    let mut grower = Grower::new(elevations);
    let mut panels = Vec::new();
    let mut props = Vec::new();

    for threshold in (1..=MAX_THRESHOLD).rev() {
        for y in 0..(MAP_HEIGHT - 1) {
            for x in 0..(MAP_WIDTH - 1) {
                if grower.is_occupied(x, y) {
                    continue;
                }

                let (w, h) = grower.grow(x, y);
                if w < threshold && h < threshold {
                    continue;
                }

                grower.claim(x, y, w, h);
                panels.push(furnish(elevations, rng, &mut props, x, y, w, h));
            }
        }
    }

    Layout { panels, props }
}

fn furnish(
    elevations: &[i16],
    rng: &mut Rng,
    props: &mut Vec<Prop>,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
) -> Panel {
    let at = |dx: usize, dy: usize| elevations[(y + dy) * MAP_WIDTH + x + dx];
    let base = at(0, 0);
    let water = base <= WATER_ELEVATION
        && at(w - 1, 0) <= WATER_ELEVATION
        && at(0, h - 1) <= WATER_ELEVATION
        && at(w - 1, h - 1) <= WATER_ELEVATION;

    let center_x = (x + w / 2) as f32;
    let center_y = (y + h / 2) as f32;
    let mut place = |kind: PropKind| {
        props.push(Prop {
            kind,
            cell_x: center_x,
            cell_y: center_y,
        });
    };

    let color = if water {
        if rng.chance(WATER_PROP_CHANCE) {
            if rng.chance(BOAT_SHARE) {
                place(PropKind::Boat(if rng.coin() { id::BOAT_A } else { id::BOAT_B }));
            } else {
                place(PropKind::Fish);
            }
        }
        PanelColor::solid(palette::BLUE)
    } else {
        let color = paint(rng, base);
        if base < ROCK_ELEVATION && rng.chance(TREE_CHANCE) {
            place(PropKind::Tree);
        }
        color
    };

    Panel {
        x: x as u16,
        y: y as u16,
        w: w as u8,
        h: h as u8,
        water,
        color,
    }
}

fn paint(rng: &mut Rng, height: i16) -> PanelColor {
    if height < ROCK_ELEVATION {
        meadow(rng)
    } else if height < SNOW_ELEVATION {
        if rng.next_u16() & 3 == 0 {
            if rng.coin() {
                PanelColor::solid(palette::LTGRAY)
            } else {
                PanelColor::dither(palette::DKGRAY, palette::LTGRAY)
            }
        } else {
            meadow(rng)
        }
    } else if rng.next_u16() & 3 == 0 {
        if rng.coin() {
            PanelColor::solid(palette::WHITE)
        } else {
            PanelColor::solid(palette::LTGRAY)
        }
    } else if rng.coin() {
        PanelColor::dither(palette::LTGRAY, palette::WHITE)
    } else {
        PanelColor::dither(palette::DKGRAY, palette::LTGRAY)
    }
}

fn meadow(rng: &mut Rng) -> PanelColor {
    if rng.coin() {
        PanelColor::solid(palette::LTGREEN)
    } else {
        PanelColor::dither(palette::GREEN, palette::LTGREEN)
    }
}

struct Grower<'a> {
    elevations: &'a [i16],
    normals: Vec<Vec3>,
    occupied: Vec<bool>,
}

impl<'a> Grower<'a> {
    fn new(elevations: &'a [i16]) -> Self {
        Self {
            elevations,
            normals: slope_normals(elevations),
            occupied: vec![false; MAP_WIDTH * MAP_HEIGHT],
        }
    }

    fn is_occupied(&self, x: usize, y: usize) -> bool {
        self.occupied[y * MAP_WIDTH + x]
    }

    fn claim(&mut self, x: usize, y: usize, w: usize, h: usize) {
        for row in y..y + h {
            let start = row * MAP_WIDTH + x;
            self.occupied[start..start + w].fill(true);
        }
    }

    fn fits(&self, base: Vec3, x: usize, y: usize) -> bool {
        let index = y * MAP_WIDTH + x;
        !self.occupied[index] && (self.normals[index] - base).length_squared() <= SAME_SLOPE_NORM_SQR
    }

    fn column_fits(&self, base: Vec3, x: usize, y: usize, h: usize) -> bool {
        (0..h).all(|k| self.fits(base, x, y + k))
    }

    fn row_fits(&self, base: Vec3, x: usize, y: usize, w: usize) -> bool {
        (0..w).all(|k| self.fits(base, x + k, y))
    }

    fn grow(&self, x: usize, y: usize) -> (usize, usize) {
        let base = self.normals[y * MAP_WIDTH + x];
        let mut w = 1usize;
        let mut h = 1usize;

        loop {
            let mut grew = false;

            if w < PANEL_SIZE_MAX
                && x + w + 1 < MAP_WIDTH
                && self.column_fits(base, x + w, y, h)
                && self.edges_fit(x, y, w + 1, h)
            {
                w += 1;
                grew = true;
            }

            if h < PANEL_SIZE_MAX
                && y + h + 1 < MAP_HEIGHT
                && self.row_fits(base, x, y + h, w)
                && self.edges_fit(x, y, w, h + 1)
            {
                h += 1;
                grew = true;
            }

            if !grew {
                return (w, h);
            }
        }
    }

    fn edges_fit(&self, x: usize, y: usize, w: usize, h: usize) -> bool {
        let at = |dx: usize, dy: usize| self.elevations[(y + dy) * MAP_WIDTH + x + dx] as f32;
        let (e00, e10, e01, e11) = (at(0, 0), at(w, 0), at(0, h), at(w, h));

        let line_fits = |count: usize, from: f32, to: f32, sample: &dyn Fn(usize) -> f32| {
            (1..count).all(|k| {
                let expected = from + (to - from) * k as f32 / count as f32;
                (sample(k) - expected).abs() <= EDGE_TOLERANCE
            })
        };

        line_fits(w, e00, e10, &|k| at(k, 0))
            && line_fits(w, e01, e11, &|k| at(k, h))
            && line_fits(h, e00, e01, &|k| at(0, k))
            && line_fits(h, e10, e11, &|k| at(w, k))
    }
}

fn slope_normals(elevations: &[i16]) -> Vec<Vec3> {
    let mut normals = vec![Vec3::ZERO; MAP_WIDTH * MAP_HEIGHT];

    for y in 0..(MAP_HEIGHT - 1) {
        for x in 0..(MAP_WIDTH - 1) {
            let index = y * MAP_WIDTH + x;
            let here = elevations[index];
            let dx = (elevations[index + 1] - here) as f32;
            let dy = (elevations[index + MAP_WIDTH] - here) as f32;
            normals[index] = vec3(dx, dy, -1.0).normalize();
        }
    }

    normals
}
