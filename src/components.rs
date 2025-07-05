use bevy::prelude::*;

// Player-related components
#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct PlayerState {
    pub is_moving: bool,
    pub is_running: bool,
    pub is_aiming: bool,
    pub is_shooting: bool,
}

// Camera components
#[derive(Component)]
pub struct CameraController {
    pub zoom: f32,
}

// World components
#[derive(Component, Debug)]
pub struct ChunkEntity {
    pub position: IVec2,
}

// UI components
#[derive(Component)]
pub struct MapUI;

#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct BiomeText;

#[derive(Component)]
pub struct CoordText;

#[derive(Component)]
pub struct MapPlayerMarker;

// Environment components
#[derive(Component)]
pub struct Cactus;

#[derive(Component)]
pub struct Tree;

#[derive(Component)]
pub struct Enemy;

// Weapon components
#[derive(Component)]
pub struct Gun;

#[derive(Component)]
pub struct TargetIndicator;

// Timers
#[derive(Component)]
pub struct ShootingTimer(pub Timer);
