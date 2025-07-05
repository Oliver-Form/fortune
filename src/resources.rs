use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default, PartialEq, Clone, Copy, Debug)]
pub enum CameraView {
    #[default]
    Isometric,
    FirstPerson,
}

// Resources
#[derive(Resource)]
pub struct WorldData {
    pub tiles: Vec<TileType>,
    pub chunks: HashMap<IVec2, ChunkData>,
    pub explored_chunks: std::collections::HashSet<IVec2>,
}

#[derive(Resource)]
pub struct MapVisible(pub bool);

#[derive(Resource)]
pub struct FpsTimer(pub Timer);

#[derive(Resource)]
pub struct AimState {
    pub target_enemy: Option<Entity>,
    pub gun_drawn: bool,
}

#[derive(Resource)]
pub struct PlayerAssets {
    pub cowboy_scene: Handle<Scene>,
    pub idle_animation: Handle<AnimationClip>,
    pub walking_animation: Handle<AnimationClip>,
    pub running_animation: Handle<AnimationClip>,
    pub shooting_animation: Handle<AnimationClip>,
    pub aiming_animation: Handle<AnimationClip>,
    pub holster_animation: Handle<AnimationClip>,
}

#[derive(Resource)]
pub struct CameraViewResource(pub CameraView);

#[derive(Resource, Default)]
pub struct ChunkBorderVisible(pub bool);

// Data structures
pub struct ChunkData {
    pub position: IVec2,
    pub tiles: [[TileType; crate::constants::CHUNK_SIZE as usize]; crate::constants::CHUNK_SIZE as usize],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TileType {
    Grass,    // 0
    Water,    // 1
    Desert,   // 2
    Stone,    // 3
    Wood,     // 4
    Unknown,  // fallback
}

impl TileType {
    pub fn from_u16(value: u16) -> Self {
        match value {
            0 => TileType::Grass,
            1 => TileType::Water,
            2 => TileType::Desert,
            3 => TileType::Stone,
            4 => TileType::Wood,
            _ => TileType::Unknown,
        }
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            TileType::Grass => "Grassland",
            TileType::Water => "Water",
            TileType::Desert => "Desert",
            TileType::Stone => "Stone",
            TileType::Wood => "Wood",
            TileType::Unknown => "Unknown",
        }
    }

    pub fn get_color(&self) -> Color {
        match self {
            TileType::Grass => crate::constants::COLORS::GRASS_GREEN,
            TileType::Water => crate::constants::COLORS::WATER_BLUE,
            TileType::Desert => crate::constants::COLORS::DESERT_TAN,
            TileType::Stone => Color::rgb(0.5, 0.5, 0.5),
            TileType::Wood => Color::rgb(0.6, 0.3, 0.1),
            TileType::Unknown => Color::rgb(1.0, 0.0, 1.0), // Bright pink for unknown tiles
        }
    }
}
