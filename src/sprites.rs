mod parser;

pub use parser::{MeshTriangle, Sprite, SpriteElementData, parse_talons};

pub mod id {
    pub const FISH: u32 = 1;
    pub const BIRD_GLIDE: u32 = 2;
    pub const BIRD_FLAP: u32 = 3;
    pub const BOAT_A: u32 = 4;
    pub const BOAT_B: u32 = 5;
    pub const TREE: u32 = 6;
    pub const TALON_UP: u32 = 7;
    pub const TALON_DOWN: u32 = 8;

    pub const REQUIRED: [u32; 8] = [
        FISH, BIRD_GLIDE, BIRD_FLAP, BOAT_A, BOAT_B, TREE, TALON_UP, TALON_DOWN,
    ];
}
