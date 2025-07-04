// Map loader module for the main Fortune game
// Add this to your main game to load map files created by the editor

use std::fs::File;
use std::io::Read;

pub const MAP_SIZE: usize = 4096;
pub const CHUNK_SIZE: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum MapTileType {
    Grass = 0,
    Desert = 1,
    Water = 2,
    Rock = 3,
    Sand = 4,
}

impl MapTileType {
    pub fn from_u16(value: u16) -> Self {
        match value {
            0 => MapTileType::Grass,
            1 => MapTileType::Desert,
            2 => MapTileType::Water,
            3 => MapTileType::Rock,
            4 => MapTileType::Sand,
            _ => MapTileType::Grass,
        }
    }

    pub fn get_color(self) -> bevy::prelude::Color {
        match self {
            MapTileType::Grass => bevy::prelude::Color::srgb(0.2, 0.8, 0.2),
            MapTileType::Desert => bevy::prelude::Color::srgb(0.8, 0.7, 0.4),
            MapTileType::Water => bevy::prelude::Color::srgb(0.2, 0.4, 0.8),
            MapTileType::Rock => bevy::prelude::Color::srgb(0.5, 0.5, 0.5),
            MapTileType::Sand => bevy::prelude::Color::srgb(0.9, 0.8, 0.6),
        }
    }
}

pub struct GameMap {
    pub tiles: Vec<Vec<MapTileType>>,
}

impl GameMap {
    pub fn new() -> Self {
        Self {
            tiles: vec![vec![MapTileType::Grass; MAP_SIZE]; MAP_SIZE],
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        if buffer.len() != MAP_SIZE * MAP_SIZE * 2 {
            return Err("Invalid map file size".into());
        }

        let mut tiles = vec![vec![MapTileType::Grass; MAP_SIZE]; MAP_SIZE];

        for z in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                let index = (z * MAP_SIZE + x) * 2;
                let value = u16::from_le_bytes([buffer[index], buffer[index + 1]]);
                tiles[z][x] = MapTileType::from_u16(value);
            }
        }

        Ok(Self { tiles })
    }

    pub fn get_tile(&self, x: usize, z: usize) -> MapTileType {
        if x < MAP_SIZE && z < MAP_SIZE {
            self.tiles[z][x]
        } else {
            MapTileType::Grass // Default for out of bounds
        }
    }

    pub fn get_chunk_tiles(&self, chunk_x: i32, chunk_z: i32) -> Vec<Vec<MapTileType>> {
        let mut chunk_tiles = vec![vec![MapTileType::Grass; CHUNK_SIZE]; CHUNK_SIZE];
        
        let start_x = chunk_x as usize * CHUNK_SIZE;
        let start_z = chunk_z as usize * CHUNK_SIZE;

        for local_z in 0..CHUNK_SIZE {
            for local_x in 0..CHUNK_SIZE {
                let world_x = start_x + local_x;
                let world_z = start_z + local_z;
                
                if world_x < MAP_SIZE && world_z < MAP_SIZE {
                    chunk_tiles[local_z][local_x] = self.tiles[world_z][world_x];
                }
            }
        }

        chunk_tiles
    }
}

// Example usage in your main game:
// 
// fn load_game_map(mut commands: Commands) {
//     match GameMap::load_from_file("world_map.map") {
//         Ok(map) => {
//             commands.insert_resource(map);
//             info!("Game map loaded successfully!");
//         }
//         Err(e) => {
//             warn!("Failed to load map file, using default: {}", e);
//             commands.insert_resource(GameMap::new());
//         }
//     }
// }
