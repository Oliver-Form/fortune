use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::collections::HashMap;

const TILE_SIZE: f32 = 1.0;
const WORLD_SIZE: i32 = 100; // 100x100 world
const CHUNK_SIZE: i32 = 10; // 10x10 tiles per chunk

// Components
#[derive(Component)]
struct Player;

#[derive(Component)]
struct CameraController;

#[derive(Component)]
struct WorldTile {
    chunk_pos: IVec2,
    tile_pos: IVec2,
}

#[derive(Component)]
struct MapUI;

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct BiomeText;

// Resources
#[derive(Resource)]
struct WorldData {
    chunks: HashMap<IVec2, ChunkData>,
    explored_chunks: std::collections::HashSet<IVec2>,
}

#[derive(Resource)]
struct MapVisible(bool);

#[derive(Resource)]
struct FpsTimer(Timer);

// Data structures
struct ChunkData {
    position: IVec2,
    color: Color,
}

#[derive(Clone, Copy, Debug)]
enum TileType {
    Desert,    // Tan
    Grassland, // Green  
    Mountain,  // Gray
    Water,     // Blue
}

impl TileType {
    fn get_name(&self) -> &'static str {
        match self {
            TileType::Desert => "Desert",
            TileType::Grassland => "Grassland",
            TileType::Mountain => "Mountain",
            TileType::Water => "Water",
        }
    }

    fn get_color(&self) -> Color {
        match self {
            TileType::Desert => Color::rgb(0.8, 0.7, 0.4),    // Tan
            TileType::Grassland => Color::rgb(0.2, 0.6, 0.2), // Green
            TileType::Mountain => Color::rgb(0.5, 0.5, 0.5),  // Gray
            TileType::Water => Color::rgb(0.2, 0.4, 0.8),     // Blue
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .insert_resource(MapVisible(false))
        .insert_resource(FpsTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            player_movement, 
            camera_follow, 
            update_explored_chunks,
            toggle_map,
            update_map_display,
            update_fps,
            update_biome_display,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Generate world
    let world_data = generate_world();
    
    // Spawn all tiles
    spawn_world_tiles(&world_data, &mut commands, &mut meshes, &mut materials);
    
    commands.insert_resource(world_data);

    // Create the player cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::rgb(0.8, 0.2, 0.2)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Player,
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
            transform: Transform::from_xyz(10.0, 10.0, 10.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraController,
    ));

    // Create map UI (initially hidden)
    commands.spawn((
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
    )).with_children(|parent| {
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
}

fn generate_world() -> WorldData {
    let mut world_data = WorldData {
        chunks: HashMap::new(),
        explored_chunks: std::collections::HashSet::new(),
    };

    let noise = Perlin::new(42); // Fixed seed for consistent world
    let chunks_per_side = WORLD_SIZE / CHUNK_SIZE;

    for chunk_x in 0..chunks_per_side {
        for chunk_z in 0..chunks_per_side {
            let chunk_pos = IVec2::new(chunk_x, chunk_z);
            
            // Generate chunk color based on noise
            let noise_value = noise.get([chunk_x as f64 * 0.1, chunk_z as f64 * 0.1]);
            
            let tile_type = match noise_value {
                n if n < -0.3 => TileType::Water,
                n if n < 0.0 => TileType::Desert,
                n if n < 0.3 => TileType::Grassland,
                _ => TileType::Mountain,
            };

            let color = tile_type.get_color();

            world_data.chunks.insert(chunk_pos, ChunkData {
                position: chunk_pos,
                color,
            });
        }
    }

    world_data
}

fn spawn_world_tiles(
    world_data: &WorldData,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let tile_mesh = meshes.add(Plane3d::default().mesh().size(TILE_SIZE, TILE_SIZE));
    
    for (chunk_pos, chunk_data) in &world_data.chunks {
        let chunk_material = materials.add(chunk_data.color);
        
        // Spawn tiles for this chunk
        for tile_x in 0..CHUNK_SIZE {
            for tile_z in 0..CHUNK_SIZE {
                let world_x = chunk_pos.x * CHUNK_SIZE + tile_x;
                let world_z = chunk_pos.y * CHUNK_SIZE + tile_z;
                
                commands.spawn((
                    PbrBundle {
                        mesh: tile_mesh.clone(),
                        material: chunk_material.clone(),
                        transform: Transform::from_xyz(
                            world_x as f32 * TILE_SIZE,
                            0.0,
                            world_z as f32 * TILE_SIZE,
                        ),
                        ..default()
                    },
                    WorldTile {
                        chunk_pos: *chunk_pos,
                        tile_pos: IVec2::new(tile_x, tile_z),
                    },
                ));
            }
        }
    }
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = player_query.get_single_mut() {
        let mut direction = Vec3::ZERO;
        let speed = 5.0;

        // WASD movement
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction += Vec3::new(0.0, 0.0, -1.0);
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction += Vec3::new(0.0, 0.0, 1.0);
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction += Vec3::new(-1.0, 0.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }

        // Normalize direction and apply movement
        if direction.length() > 0.0 {
            direction = direction.normalize();
            let new_pos = transform.translation + direction * speed * time.delta_seconds();
            
            // Keep player within world bounds
            let max_pos = (WORLD_SIZE as f32 - 1.0) * TILE_SIZE;
            transform.translation = Vec3::new(
                new_pos.x.clamp(0.0, max_pos),
                0.5, // Keep above ground
                new_pos.z.clamp(0.0, max_pos),
            );
        }
    }
}

fn camera_follow(
    player_query: Query<&Transform, (With<Player>, Without<CameraController>)>,
    mut camera_query: Query<&mut Transform, (With<CameraController>, Without<Player>)>,
) {
    if let (Ok(player_transform), Ok(mut camera_transform)) = 
        (player_query.get_single(), camera_query.get_single_mut()) {
        
        // Maintain isometric view by keeping the camera at a fixed offset from the player
        let offset = Vec3::new(10.0, 10.0, 10.0);
        camera_transform.translation = player_transform.translation + offset;
        
        // Always look at the player
        camera_transform.look_at(player_transform.translation, Vec3::Y);
    }
}

fn update_explored_chunks(
    player_query: Query<&Transform, With<Player>>,
    mut world_data: ResMut<WorldData>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let chunk_x = (player_transform.translation.x / (CHUNK_SIZE as f32 * TILE_SIZE)) as i32;
        let chunk_z = (player_transform.translation.z / (CHUNK_SIZE as f32 * TILE_SIZE)) as i32;
        let current_chunk = IVec2::new(chunk_x, chunk_z);
        
        world_data.explored_chunks.insert(current_chunk);
    }
}

fn toggle_map(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut map_visible: ResMut<MapVisible>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        map_visible.0 = !map_visible.0;
    }
}

fn update_map_display(
    map_visible: Res<MapVisible>,
    mut map_ui_query: Query<&mut Style, With<MapUI>>,
) {
    if let Ok(mut style) = map_ui_query.get_single_mut() {
        style.display = if map_visible.0 {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn update_fps(
    time: Res<Time>,
    mut fps_timer: ResMut<FpsTimer>,
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
    mut fps_text_query: Query<&mut Text, With<FpsText>>,
) {
    fps_timer.0.tick(time.delta());
    
    if fps_timer.0.just_finished() {
        if let Ok(mut text) = fps_text_query.get_single_mut() {
            if let Some(fps_diagnostic) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(fps_smoothed) = fps_diagnostic.smoothed() {
                    text.sections[0].value = format!("FPS: {:.1}", fps_smoothed);
                }
            }
        }
    }
}

fn update_biome_display(
    player_query: Query<&Transform, With<Player>>,
    world_data: Res<WorldData>,
    mut biome_text_query: Query<&mut Text, With<BiomeText>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok(mut text) = biome_text_query.get_single_mut() {
            let current_biome = get_biome_at_position(&world_data, player_transform.translation);
            
            text.sections[0].value = current_biome.get_name().to_string();
            text.sections[0].style.color = current_biome.get_color();
        }
    }
}

fn get_biome_at_position(world_data: &WorldData, world_pos: Vec3) -> TileType {
    let chunk_x = (world_pos.x / (CHUNK_SIZE as f32 * TILE_SIZE)) as i32;
    let chunk_z = (world_pos.z / (CHUNK_SIZE as f32 * TILE_SIZE)) as i32;
    let chunk_pos = IVec2::new(chunk_x, chunk_z);
    
    // Use the same noise logic as world generation
    let noise = Perlin::new(42);
    let noise_value = noise.get([chunk_x as f64 * 0.1, chunk_z as f64 * 0.1]);
    
    match noise_value {
        n if n < -0.3 => TileType::Water,
        n if n < 0.0 => TileType::Desert,
        n if n < 0.3 => TileType::Grassland,
        _ => TileType::Mountain,
    }
}

