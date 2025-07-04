// Game constants for the Fortune project

use bevy::prelude::Color;

pub struct COLORS;
impl COLORS {
    pub const TAN: Color = Color::rgb(0.8, 0.7, 0.4);
    pub const GREEN: Color = Color::rgb(0.2, 0.6, 0.2);
    pub const BLUE: Color = Color::rgb(0.2, 0.4, 0.8);
    pub const GRASS_GREEN: Color = Color::rgb(0.1, 0.8, 0.1);
    pub const WATER_BLUE: Color = Color::rgb(0.1, 0.1, 0.8);
    pub const DESERT_TAN: Color = Color::rgb(0.8, 0.7, 0.4);
}

pub const TILE_SIZE: f32 = 1.0;
pub const MAP_WIDTH: i32 = 256 * CHUNK_SIZE; // 256 chunks wide
pub const MAP_HEIGHT: i32 = 256 * CHUNK_SIZE; // 256 chunks tall
pub const CHUNK_SIZE: i32 = 16; // 16x16 tiles per chunk
pub const RENDER_DISTANCE: i32 = 5; // Render a 3x3 grid of chunks (1 chunk in each direction)
pub const PLAYER_WIDTH: f32 = 0.5;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_DEPTH: f32 = 0.5;
pub const PLAYER_WALK_SPEED: f32 = 1.4; // m/s
pub const PLAYER_RUN_SPEED: f32 = 5.0; // m/s
pub const NUM_CHUNKS_X: i32 = 256;
pub const NUM_CHUNKS_Y: i32 = 256;

