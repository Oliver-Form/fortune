use bevy::prelude::*;
use bevy::window::PresentMode;
use std::fs::File;
use std::io::{BufReader, Read};
use crate::{components::*, constants::*, resources::*};

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load map data from file
    let mut tiles = Vec::new();
    if let Ok(file) = File::open("src/four_quadrants.map") {
        let mut reader = BufReader::new(file);
        let mut buffer = [0; 2]; // Read 2 bytes for u16
        while let Ok(bytes_read) = reader.read(&mut buffer) {
            if bytes_read == 0 {
                break;
            }
            if bytes_read == 2 {
                let tile_value = u16::from_le_bytes(buffer);
                tiles.push(TileType::from_u16(tile_value));
            }
        }
    } else {
        // Fill with default if file not found
        tiles = vec![TileType::Grass; (MAP_WIDTH * MAP_HEIGHT) as usize];
        println!("Could not load map file, generating default world.");
    }

    // Initialize empty world data
    let world_data = WorldData {
        tiles,
        chunks: std::collections::HashMap::new(),
        explored_chunks: std::collections::HashSet::new(),
    };

    commands.insert_resource(world_data);

    // Load cowboy models and animations from the combined GLB file
    let cowboy_scene: Handle<Scene> = asset_server.load("models/cowboy_combined.glb#Scene0");
    let idle_animation: Handle<AnimationClip> =
        asset_server.load("models/cowboy_combined.glb#Animation4");
    let walking_animation: Handle<AnimationClip> =
        asset_server.load("models/cowboy_combined.glb#Animation9");
    let running_animation: Handle<AnimationClip> =
        asset_server.load("models/cowboy_combined.glb#Animation5");
    let shooting_animation: Handle<AnimationClip> =
        asset_server.load("models/cowboy_combined.glb#Animation7");
    let aiming_animation: Handle<AnimationClip> =
        asset_server.load("models/cowboy_combined.glb#Animation0");
    let holster_animation: Handle<AnimationClip> =
        asset_server.load("models/cowboy_combined.glb#Animation2");

    commands.insert_resource(PlayerAssets {
        cowboy_scene,
        idle_animation,
        walking_animation,
        running_animation,
        shooting_animation,
        aiming_animation,
        holster_animation,
    });

    // Create temporary player cube (will be replaced when model loads)
    let start_x = (MAP_WIDTH as f32) / 2.0;
    let start_z = (MAP_HEIGHT as f32) / 2.0;
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_DEPTH)),
            material: materials.add(Color::rgb(0.8, 0.2, 0.2)),
            transform: Transform::from_xyz(start_x, PLAYER_HEIGHT / 2.0, start_z),
            ..default()
        },
        Player,
        PlayerState::default(),
    ));

    // Add a light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(50.0, 50.0, 50.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)
                * Quat::from_rotation_y(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });

    // Create isometric camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraController {
            zoom: 10.0, // Default zoom distance
        },
    ));

    // Create map UI (initially hidden)
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(80.0),
                    height: Val::Percent(80.0),
                    position_type: PositionType::Absolute,
                    left: Val::Percent(10.0),
                    top: Val::Percent(10.0),
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.8).into(),
                ..default()
            },
            MapUI,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "World Map (Press M to close)",
                TextStyle {
                    font_size: 30.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });

    // Create FPS counter UI
    commands.spawn((
        TextBundle::from_section(
            "FPS: 0",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        FpsText,
    ));

    // Create Biome indicator UI
    commands.spawn((
        TextBundle::from_section(
            "Desert",
            TextStyle {
                font_size: 24.0,
                color: Color::rgb(0.8, 0.7, 0.4), // Desert color
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(10.0),
            ..default()
        }),
        BiomeText,
    ));
    // Add coordinates text below biome
    commands.spawn((
        TextBundle::from_section(
            "(0.0, 0.0)",
            TextStyle {
                font_size: 18.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(70.0),
            left: Val::Px(10.0),
            ..default()
        }),
        CoordText,
    ));
}
// at the moment i am trying to 