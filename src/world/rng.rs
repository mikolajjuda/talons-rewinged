const FALLBACK_SEED: u64 = 0x4d595f53454544;
const MULTIPLIER: u64 = 6364136223846793005;

pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { FALLBACK_SEED } else { seed },
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(MULTIPLIER).wrapping_add(1);
        (self.state >> 32) as u32
    }

    pub fn next_u16(&mut self) -> u16 {
        self.next_u32() as u16
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }

    pub fn chance(&mut self, probability: f32) -> bool {
        self.next_f32() < probability
    }

    pub fn coin(&mut self) -> bool {
        self.next_u16() & 1 != 0
    }
}
