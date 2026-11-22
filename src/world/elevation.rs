use super::rng::Rng;
use super::{MAP_HEIGHT, MAP_SCALE, MAP_WIDTH, WATER_ELEVATION};

const BLOB_DIVISOR: usize = 128;
const BLOB_SIZE_STEPS: u32 = 6;
const BLOB_START_ODDS: u16 = u16::MAX / 4;

pub fn generate(rng: &mut Rng, elevations: &mut [i16]) {
    elevations.fill(0);

    for _ in 0..(MAP_WIDTH * MAP_HEIGHT / BLOB_DIVISOR) {
        let x = rng.next_u32() as usize % MAP_WIDTH;
        let y = rng.next_u32() as usize % MAP_HEIGHT;
        let mut reach = 1usize << (rng.next_u32() % BLOB_SIZE_STEPS);
        let mut layers = 0usize;

        while reach > 0 {
            reach -= 1;

            if layers == 0 && rng.next_u16() < BLOB_START_ODDS {
                layers = rng.next_u16() as usize % (reach + 1);
            }

            if layers > 0 {
                raise(elevations, x, y, reach);
                layers -= 1;
            }
        }
    }

    for value in elevations.iter_mut() {
        *value = (*value).max(WATER_ELEVATION);
    }
}

fn raise(elevations: &mut [i16], x: usize, y: usize, reach: usize) {
    let x_start = x.saturating_sub(reach);
    let x_end = (x + reach).min(MAP_WIDTH - 1);
    let y_start = y.saturating_sub(reach);
    let y_end = (y + reach).min(MAP_HEIGHT - 1);

    for row in y_start..y_end {
        let line = &mut elevations[row * MAP_WIDTH + x_start..row * MAP_WIDTH + x_end];
        for cell in line {
            *cell = cell.saturating_add(MAP_SCALE / 2);
        }
    }
}
