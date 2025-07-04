mod constants;
use constants::*;

use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::window::PresentMode; // switches off vsync
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hasher;
use std::io::{BufReader, Read};

// Components
#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct PlayerState {
    is_moving: bool,
    is_running: bool,
    is_aiming: bool,
    is_shooting: bool,
}

#[derive(Component)]
struct CameraController {
    zoom: f32,
}

#[derive(Component, Debug)]
struct ChunkEntity {
    position: IVec2,
}

#[derive(Component)]
struct MapUI;

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct BiomeText;

#[derive(Component)]
struct Cactus;

#[derive(Component)]
struct Tree;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Gun;

#[derive(Component)]
struct TargetIndicator;

#[derive(Component)]
struct MapPlayerMarker;

#[derive(Component)]
struct CoordText;

#[derive(Resource, Default, PartialEq, Clone, Copy, Debug)]
enum CameraView {
    #[default]
    Isometric,
    FirstPerson,
}

// Resources
#[derive(Resource)]
struct WorldData {
    tiles: Vec<TileType>,
    chunks: HashMap<IVec2, ChunkData>,
    explored_chunks: std::collections::HashSet<IVec2>,
}

#[derive(Resource)]
struct MapVisible(bool);

#[derive(Resource)]
struct FpsTimer(Timer);

#[derive(Resource)]
struct AimState {
    target_enemy: Option<Entity>,
    gun_drawn: bool,
}

#[derive(Resource)]
struct PlayerAssets {
    cowboy_scene: Handle<Scene>,
    idle_animation: Handle<AnimationClip>,
    walking_animation: Handle<AnimationClip>,
    running_animation: Handle<AnimationClip>,
    shooting_animation: Handle<AnimationClip>,
    aiming_animation: Handle<AnimationClip>,
    holster_animation: Handle<AnimationClip>,
}

#[derive(Resource)]
struct CameraViewResource(CameraView);

#[derive(Resource, Default)]
struct ChunkBorderVisible(bool);

// Data structures
struct ChunkData {
    position: IVec2,
    tiles: [[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum TileType {
    Grass,    // 0
    Water,    // 1
    Desert,   // 2
    Stone,    // 3
    Wood,     // 4
    Unknown,  // fallback
}

impl TileType {
    fn from_u16(value: u16) -> Self {
        match value {
            0 => TileType::Grass,
            1 => TileType::Water,
            2 => TileType::Desert,
            3 => TileType::Stone,
            4 => TileType::Wood,
            _ => TileType::Unknown,
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            TileType::Grass => "Grassland",
            TileType::Water => "Water",
            TileType::Desert => "Desert",
            TileType::Stone => "Stone",
            TileType::Wood => "Wood",
            TileType::Unknown => "Unknown",
        }
    }

    fn get_color(&self) -> Color {
        match self {
            TileType::Grass => COLORS::GRASS_GREEN,
            TileType::Water => COLORS::WATER_BLUE,
            TileType::Desert => COLORS::DESERT_TAN,
            TileType::Stone => Color::rgb(0.5, 0.5, 0.5),
            TileType::Wood => Color::rgb(0.6, 0.3, 0.1),
            TileType::Unknown => Color::rgb(1.0, 0.0, 1.0), // Bright pink for unknown tiles
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .insert_resource(MapVisible(false))
        .insert_resource(FpsTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .insert_resource(AimState {
            target_enemy: None,
            gun_drawn: false,
        })
        .insert_resource(CameraViewResource(CameraView::Isometric))
        .insert_resource(ChunkBorderVisible(false))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                camera_follow,
                camera_zoom,
                update_explored_chunks,
                manage_world_chunks,
                gun_control,
                aim_system,
                shooting_system,
                toggle_map,
                update_map_display,
                update_fps,
                update_biome_display,
                spawn_player_when_loaded,
                handle_animations,
                toggle_camera_view,
                toggle_chunk_borders,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load map data from file
    let mut tiles = Vec::new();
    if let Ok(file) = File::open("src/file_checkers.map") {
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
        chunks: HashMap::new(),
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

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut PlayerState), With<Player>>,
    camera_query: Query<&Transform, (With<CameraController>, Without<Player>)>,
    camera_view: Res<CameraViewResource>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut player_state)) = player_query.get_single_mut() {
        let mut direction = Vec3::ZERO;
        let walk_speed = PLAYER_WALK_SPEED;
        let run_speed = PLAYER_RUN_SPEED;

        // Check if running (Shift key held)
        let is_running = keyboard_input.pressed(KeyCode::ShiftLeft)
            || keyboard_input.pressed(KeyCode::ShiftRight);

        match camera_view.0 {
            CameraView::Isometric => {
                // Original third-person movement logic
                if let Ok(camera_transform) = camera_query.get_single() {
                    // Calculate camera's forward and right vectors (projected onto XZ plane)
                    let camera_forward = camera_transform.forward();
                    let camera_right = camera_transform.right();
                    
                    // Project onto XZ plane (ignore Y component for ground movement)
                    let forward_xz = Vec3::new(camera_forward.x, 0.0, camera_forward.z).normalize();
                    let right_xz = Vec3::new(camera_right.x, 0.0, camera_right.z).normalize();

                    // Track which keys are pressed for rotation calculation
                    let w_pressed = keyboard_input.pressed(KeyCode::KeyW);
                    let s_pressed = keyboard_input.pressed(KeyCode::KeyS);
                    let a_pressed = keyboard_input.pressed(KeyCode::KeyA);
                    let d_pressed = keyboard_input.pressed(KeyCode::KeyD);

                    // WASD movement relative to camera orientation
                    if w_pressed {
                        direction += forward_xz; // Move away from camera (up on screen)
                    }
                    if s_pressed {
                        direction -= forward_xz; // Move toward camera (down on screen)
                    }
                    if a_pressed {
                        direction -= right_xz; // Move left relative to camera
                    }
                    if d_pressed {
                        direction += right_xz; // Move right relative to camera
                    }

                    // Calculate target rotation based on key combinations (in degrees)
                    let target_angle_degrees: f32 = match (w_pressed, s_pressed, a_pressed, d_pressed) {
                        // Single keys
                        (true, false, false, false) => 225.0,    // W only
                        (false, true, false, false) => 45.0,  // S only
                        (false, false, true, false) => 315.0,  // A only
                        (false, false, false, true) => 135.0,   // D only
                        
                        // Diagonal combinations
                        (true, false, false, true) => 180.0,    // W+D
                        (true, false, true, false) => 270.0,   // W+A
                        (false, true, false, true) => 90.0,   // S+D
                        (false, true, true, false) => 360.0,   // S+A
                        
                        // Default case (no movement or conflicting keys)
                        _ => {
                            // Update movement state for isometric mode
                            player_state.is_moving = direction.length() > 0.0;
                            player_state.is_running = player_state.is_moving && is_running;

                            // Normalize direction and apply movement
                            if direction.length() > 0.0 {
                                direction = direction.normalize();
                                
                                let speed = if is_running { run_speed } else { walk_speed };
                                let new_pos = transform.translation + direction * speed * time.delta_seconds();

                                // Keep player within world bounds
                                let max_pos_x = (MAP_WIDTH as f32 - 1.0) * TILE_SIZE;
                                let max_pos_z = (MAP_HEIGHT as f32 - 1.0) * TILE_SIZE;
                                transform.translation = Vec3::new(
                                    new_pos.x.clamp(0.0, max_pos_x),
                                    0.5, // Keep above ground
                                    new_pos.z.clamp(0.0, max_pos_z),
                                );
                            }
                            return;
                        }
                    };

                    // Convert degrees to radians and apply rotation
                    if direction.length() > 0.0 {
                        let target_angle_radians = target_angle_degrees.to_radians();
                        let target_rotation = Quat::from_rotation_y(target_angle_radians);
                        transform.rotation = transform.rotation.slerp(target_rotation, 10.0 * time.delta_seconds());
                    }
                }
            }
            CameraView::FirstPerson => {
                // FPS-style movement logic
                let w_pressed = keyboard_input.pressed(KeyCode::KeyW);
                let s_pressed = keyboard_input.pressed(KeyCode::KeyS);
                let a_pressed = keyboard_input.pressed(KeyCode::KeyA);
                let d_pressed = keyboard_input.pressed(KeyCode::KeyD);

                // Get player's forward and right vectors
                let player_forward = transform.forward();
                let player_right = transform.right();
                
                // Project onto XZ plane for ground movement
                let forward_xz = Vec3::new(player_forward.x, 0.0, player_forward.z).normalize();
                let right_xz = Vec3::new(player_right.x, 0.0, player_right.z).normalize();

                // WASD movement relative to player orientation
                if w_pressed {
                    direction -= forward_xz; // Move forward (inverted because of camera orientation)
                }
                if s_pressed {
                    direction += forward_xz; // Move backward (inverted because of camera orientation)
                }
                if a_pressed {
                    direction += right_xz; // Strafe left (inverted because of camera orientation)
                }
                if d_pressed {
                    direction -= right_xz; // Strafe right (inverted because of camera orientation)
                }

                // Mouse look for rotation
                let mouse_sensitivity = 0.002;
                for motion in mouse_motion.read() {
                    // Rotate player around Y axis for horizontal look
                    let yaw_rotation = Quat::from_rotation_y(-motion.delta.x * mouse_sensitivity);
                    transform.rotation = yaw_rotation * transform.rotation;
                }
            }
        }

        // Update movement state
        player_state.is_moving = direction.length() > 0.0;
        player_state.is_running = player_state.is_moving && is_running;

        // Normalize direction and apply movement
        if direction.length() > 0.0 {
            direction = direction.normalize();
            let speed = if is_running { run_speed } else { walk_speed };
            let new_pos = transform.translation + direction * speed * time.delta_seconds();

            // Keep player within world bounds
            let max_pos_x = (MAP_WIDTH as f32 - 1.0) * TILE_SIZE;
            let max_pos_z = (MAP_HEIGHT as f32 - 1.0) * TILE_SIZE;
            transform.translation = Vec3::new(
                new_pos.x.clamp(0.0, max_pos_x),
                0.5, // Keep above ground
                new_pos.z.clamp(0.0, max_pos_z),
            );
        }
    }
}

fn camera_follow(
    player_query: Query<&Transform, (With<Player>, Without<CameraController>)>,
    mut camera_query: Query<
        (&mut Transform, &CameraController),
        (With<CameraController>, Without<Player>),
    >,
    camera_view: Res<CameraViewResource>,
) {
    if let (Ok(player_transform), Ok((mut camera_transform, camera_controller))) =
        (player_query.get_single(), camera_query.get_single_mut())
    {
        match camera_view.0 {
            CameraView::Isometric => {
                // Use zoom value for offset distance
                let zoom_distance = camera_controller.zoom;
                let offset = Vec3::new(zoom_distance, zoom_distance, zoom_distance);
                camera_transform.translation = player_transform.translation + offset;

                // Always look at the player
                camera_transform.look_at(player_transform.translation, Vec3::Y);
            }
            CameraView::FirstPerson => {
                // Position camera at player's head
                let head_offset = Vec3::new(0.0, 1.8, 0.0); // Adjust Y for head height
                camera_transform.translation = player_transform.translation + head_offset;

                // Set camera rotation to match player rotation but face forward
                camera_transform.rotation = player_transform.rotation * Quat::from_rotation_y(std::f32::consts::PI);
            }
        }
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

fn toggle_map(keyboard_input: Res<ButtonInput<KeyCode>>, mut map_visible: ResMut<MapVisible>) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        map_visible.0 = !map_visible.0;
    }
}

fn update_map_display(
    map_visible: Res<MapVisible>,
    mut map_ui_query: Query<(Entity, &mut Style), With<MapUI>>,
    player_query: Query<&Transform, With<Player>>,
    mut commands: Commands,
    marker_query: Query<Entity, With<MapPlayerMarker>>,
) {
    if let Ok((map_ui_entity, mut style)) = map_ui_query.get_single_mut() {
        style.display = if map_visible.0 {
            Display::Flex
        } else {
            Display::None
        };
        // Remove old marker if exists
        for entity in marker_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        if map_visible.0 {
            if let Ok(player_transform) = player_query.get_single() {
                // Map UI green square always represents the whole map
                let map_width_px = 400.0; // 80% of 500px, adjust as needed
                let map_height_px = 400.0;
                let map_left_px = 0.0;
                let map_top_px = 0.0;

                // Calculate player's position as a percentage of the map
                let player_x = player_transform.translation.x.clamp(0.0, (MAP_WIDTH as f32 - 1.0) * TILE_SIZE);
                let player_y = player_transform.translation.z.clamp(0.0, (MAP_HEIGHT as f32 - 1.0) * TILE_SIZE);
                let percent_x = player_x / (MAP_WIDTH as f32 * TILE_SIZE);
                let percent_y = player_y / (MAP_HEIGHT as f32 * TILE_SIZE);

                // Add green square for the whole map
                let marker = commands.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(map_width_px),
                        height: Val::Px(map_height_px),
                        position_type: PositionType::Absolute,
                        left: Val::Px(map_left_px),
                        top: Val::Px(map_top_px),
                        ..default()
                    },
                    background_color: Color::GREEN.into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Add a small red dot to mark the player's position on the map
                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(8.0),
                            height: Val::Px(8.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(percent_x * map_width_px - 4.0),
                            top: Val::Px(percent_y * map_height_px - 4.0),
                            ..default()
                        },
                        background_color: Color::RED.into(),
                        ..default()
                    });
                })
                .insert(MapPlayerMarker)
                .id();
                commands.entity(map_ui_entity).add_child(marker);
            }
        }
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
            if let Some(fps_diagnostic) =
                diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
            {
                if let Some(fps_smoothed) = fps_diagnostic.smoothed() {
                    text.sections[0].value = format!("FPS: {:.1}", fps_smoothed);
                }
            }
        }
    }
}

use bevy::ecs::system::ParamSet;

fn update_biome_display(
    player_query: Query<&Transform, With<Player>>,
    world_data: Res<WorldData>,
    mut param_set: ParamSet<(
        Query<&mut Text, With<BiomeText>>,
        Query<&mut Text, With<CoordText>>,
    )>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        // Update biome text
        if let Ok(mut text) = param_set.p0().get_single_mut() {
            let tile_x = (player_transform.translation.x / TILE_SIZE) as i32;
            let tile_y = (player_transform.translation.z / TILE_SIZE) as i32;

            let current_biome = get_tile_at_position(&world_data, tile_x, tile_y);

            text.sections[0].value = current_biome.get_name().to_string();
            text.sections[0].style.color = current_biome.get_color();
        }
        // Update coordinates text
        if let Ok(mut coord_text) = param_set.p1().get_single_mut() {
            let x = player_transform.translation.x;
            let z = player_transform.translation.z;
            coord_text.sections[0].value = format!("({:.1}, {:.1})", x, z);
        }
    }
}

fn get_tile_at_position(world_data: &WorldData, x: i32, y: i32) -> TileType {
    if x >= 0 && x < MAP_WIDTH && y >= 0 && y < MAP_HEIGHT {
        let index = (y * MAP_WIDTH + x) as usize;
        if index < world_data.tiles.len() {
            return world_data.tiles[index];
        }
    }
    TileType::Unknown
}

fn manage_world_chunks(
    player_query: Query<&Transform, With<Player>>,
    mut world_data: ResMut<WorldData>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunk_entities: Query<(Entity, &ChunkEntity)>,
    chunk_border_visible: Res<ChunkBorderVisible>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let player_chunk = IVec2::new(
            (player_transform.translation.x / (CHUNK_SIZE as f32 * TILE_SIZE)) as i32,
            (player_transform.translation.z / (CHUNK_SIZE as f32 * TILE_SIZE)) as i32,
        );

        let max_chunk_x = MAP_WIDTH / CHUNK_SIZE;
        let max_chunk_y = MAP_HEIGHT / CHUNK_SIZE;

        let mut chunks_to_keep = std::collections::HashSet::new();

        // Generate and spawn chunks around player
        for x in -RENDER_DISTANCE..=RENDER_DISTANCE {
            for z in -RENDER_DISTANCE..=RENDER_DISTANCE {
                let chunk_pos = player_chunk + IVec2::new(x, z);
                chunks_to_keep.insert(chunk_pos);

                // Check if chunk is within world boundaries
                if chunk_pos.x >= 0
                    && chunk_pos.x < max_chunk_x
                    && chunk_pos.y >= 0
                    && chunk_pos.y < max_chunk_y
                {
                    if !world_data.chunks.contains_key(&chunk_pos) {
                        if let Some(chunk_data) = generate_chunk_from_map(&world_data, chunk_pos) {
                            spawn_chunk(&chunk_data, &mut commands, &mut meshes, &mut materials, &chunk_border_visible);
                            world_data.chunks.insert(chunk_pos, chunk_data);
                        }
                    }
                }
            }
        }

        // Despawn chunks that are too far away
        for (entity, chunk_comp) in chunk_entities.iter() {
            if !chunks_to_keep.contains(&chunk_comp.position) {
                commands.entity(entity).despawn_recursive();
                world_data.chunks.remove(&chunk_comp.position);
            }
        }
    }
}

fn generate_chunk_from_map(world_data: &WorldData, chunk_pos: IVec2) -> Option<ChunkData> {
    let mut tiles = [[TileType::Unknown; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
    let start_x = chunk_pos.x * CHUNK_SIZE;
    let start_y = chunk_pos.y * CHUNK_SIZE;

    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let world_x = start_x + x;
            let world_y = start_y + y;
            tiles[y as usize][x as usize] = get_tile_at_position(world_data, world_x, world_y);
        }
    }

    Some(ChunkData {
        position: chunk_pos,
        tiles,
    })
}

fn spawn_chunk(
    chunk_data: &ChunkData,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk_border_visible: &Res<ChunkBorderVisible>,
) {
    let chunk_world_x = chunk_data.position.x as f32 * CHUNK_SIZE as f32 * TILE_SIZE;
    let chunk_world_z = chunk_data.position.y as f32 * CHUNK_SIZE as f32 * TILE_SIZE;

    // --- Single Mesh with Vertex Colors ---
    // This approach creates one single mesh for the entire chunk.
    // Each tile is a quad with its vertices colored according to the tile type.
    // This is extremely performant as it results in only one draw call per chunk.
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();
    let mut vertex_count: u32 = 0;

    for y in 0..CHUNK_SIZE as usize {
        for x in 0..CHUNK_SIZE as usize {
            let tile_type = chunk_data.tiles[y][x];

            if tile_type == TileType::Water {
                continue; // Skip rendering water tiles
            }

            let tile_color = tile_type.get_color().as_rgba_f32();
            let x_pos = x as f32 * TILE_SIZE;
            let z_pos = y as f32 * TILE_SIZE;

            // Define the 4 vertices of the quad, with correct winding order
            positions.extend([
                // Bottom-left
                [x_pos, 0.0, z_pos],
                // Bottom-right
                [x_pos + TILE_SIZE, 0.0, z_pos],
                // Top-right
                [x_pos + TILE_SIZE, 0.0, z_pos + TILE_SIZE],
                // Top-left
                [x_pos, 0.0, z_pos + TILE_SIZE],
            ]);

            // All vertices of a tile have the same color and normal
            for _ in 0..4 {
                normals.push([0.0, 1.0, 0.0]);
                colors.push(tile_color);
            }

            // Add UVs for the quad
            uvs.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);

            // Add indices for the two triangles of the quad, with correct winding order
            indices.extend([
                vertex_count,
                vertex_count + 2,
                vertex_count + 1,
                vertex_count,
                vertex_count + 3,
                vertex_count + 2,
            ]);

            vertex_count += 4;
        }
    }

    if positions.is_empty() {
        // Spawn an empty parent entity even for water-only chunks to track them
        commands.spawn((
            SpatialBundle {
                transform: Transform::from_xyz(chunk_world_x, 0.0, chunk_world_z),
                ..default()
            },
            ChunkEntity {
                position: chunk_data.position,
            },
        ));
        return;
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));

    let mesh_handle = meshes.add(mesh);

    // Use a single material that respects vertex colors.
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE, // Set to white to not tint the vertex colors
        perceptual_roughness: 1.0,
        metallic: 0.0,
        ..default()
    });

    // Create a parent entity for the whole chunk and add the mesh as a child
    let parent_chunk_entity = commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(chunk_world_x, 0.0, chunk_world_z),
                ..default()
            },
            ChunkEntity {
                position: chunk_data.position,
            },
        ))
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: mesh_handle,
                material: material.clone(),
                ..default()
            });
        })
        .id();

    // Draw chunk border if enabled
    if chunk_border_visible.0 {
        let border_color = Color::BLACK;
        let border_thickness = 0.05;
        let border_length = CHUNK_SIZE as f32 * TILE_SIZE;
        let y = 0.01; // Slightly above ground to avoid z-fighting

        let border_material = materials.add(StandardMaterial::from(border_color));

        let borders = [
            // Top
            (
                Vec3::new(border_length / 2.0, y, 0.0),
                Quat::IDENTITY,
                Vec3::new(border_length, border_thickness, border_thickness),
            ),
            // Bottom
            (
                Vec3::new(border_length / 2.0, y, border_length),
                Quat::IDENTITY,
                Vec3::new(border_length, border_thickness, border_thickness),
            ),
            // Left
            (
                Vec3::new(0.0, y, border_length / 2.0),
                Quat::IDENTITY,
                Vec3::new(border_thickness, border_thickness, border_length),
            ),
            // Right
            (
                Vec3::new(border_length, y, border_length / 2.0),
                Quat::IDENTITY,
                Vec3::new(border_thickness, border_thickness, border_length),
            ),
        ];

        let border_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        commands.entity(parent_chunk_entity).with_children(|parent| {
            for (translation, _rotation, scale) in borders.iter() {
                parent.spawn(PbrBundle {
                    mesh: border_mesh.clone(),
                    material: border_material.clone(),
                    transform: Transform {
                        translation: *translation,
                        scale: *scale,
                        ..default()
                    },
                    ..default()
                });
            }
        });
    }

    // Add decorations
    spawn_chunk_decorations(
        chunk_data,
        commands,
        meshes,
        materials,
        parent_chunk_entity,
    );
}

fn spawn_chunk_decorations(
    chunk_data: &ChunkData,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent_chunk: Entity,
) {
    let cactus_mesh = meshes.add(Cuboid::new(0.2, 1.5, 0.2));
    let tree_mesh = meshes.add(Cuboid::new(0.3, 2.0, 0.3));
    let enemy_mesh = meshes.add(Cuboid::new(0.8, 1.8, 0.8));
    let cactus_material = materials.add(Color::rgb(0.2, 0.7, 0.2));
    let tree_material = materials.add(Color::rgb(0.4, 0.2, 0.1));
    let enemy_material = materials.add(Color::rgb(0.6, 0.2, 0.8)); // Purple enemies

    // Only add a few decorations per chunk to keep performance good
    for i in 0..CHUNK_SIZE as usize {
        for j in 0..CHUNK_SIZE as usize {
            let tile_type = chunk_data.tiles[j][i];

            // Use a hash of the position for deterministic "randomness"
            let mut hasher = DefaultHasher::new();
            hasher.write_i32(chunk_data.position.x * CHUNK_SIZE + i as i32);
            hasher.write_i32(chunk_data.position.y * CHUNK_SIZE + j as i32);
            let hash = hasher.finish();

            let world_x = (chunk_data.position.x * CHUNK_SIZE + i as i32) as f32 * TILE_SIZE;
            let world_z = (chunk_data.position.y * CHUNK_SIZE + j as i32) as f32 * TILE_SIZE;

            // Spawn enemies sporadically across all biomes (except water)
            if tile_type != TileType::Water && (hash % 200) == 0 {
                let enemy_entity = commands
                    .spawn((
                        PbrBundle {
                            mesh: enemy_mesh.clone(),
                            material: enemy_material.clone(),
                            transform: Transform::from_xyz(world_x, 0.9, world_z),
                            ..default()
                        },
                        Enemy,
                    ))
                    .id();
                commands.entity(parent_chunk).add_child(enemy_entity);
            }

            match tile_type {
                TileType::Desert => {
                    if (hash % 50) == 0 {
                        // More frequent cacti
                        let cactus_entity = commands
                            .spawn((
                                PbrBundle {
                                    mesh: cactus_mesh.clone(),
                                    material: cactus_material.clone(),
                                    transform: Transform::from_xyz(world_x, 0.75, world_z),
                                    ..default()
                                },
                                Cactus,
                            ))
                            .id();
                        commands.entity(parent_chunk).add_child(cactus_entity);
                    }
                }
                TileType::Grass => {
                    if (hash % 100) == 0 {
                        // Less frequent trees
                        let tree_entity = commands
                            .spawn((
                                PbrBundle {
                                    mesh: tree_mesh.clone(),
                                    material: tree_material.clone(),
                                    transform: Transform::from_xyz(world_x, 1.0, world_z),
                                    ..default()
                                },
                                Tree,
                            ))
                            .id();
                        commands.entity(parent_chunk).add_child(tree_entity);
                    }
                }
                _ => {} // No decorations for other types
            }
        }
    }
}

fn camera_zoom(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<&mut CameraController>,
    time: Res<Time>,
) {
    if let Ok(mut camera_controller) = camera_query.get_single_mut() {
        let zoom_speed = 15.0; // How fast to zoom with keyboard
        let scroll_zoom_speed = 2.0; // How fast to zoom with mouse wheel
        let min_zoom = 3.0; // Closest zoom
        let max_zoom = 25.0; // Farthest zoom

        // Keyboard zoom controls (Q to zoom out, E to zoom in)
        if keyboard_input.pressed(KeyCode::KeyQ) {
            camera_controller.zoom += zoom_speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            camera_controller.zoom -= zoom_speed * time.delta_seconds();
        }

        // Mouse wheel zoom
        for scroll in scroll_events.read() {
            camera_controller.zoom -= scroll.y * scroll_zoom_speed;
        }

        // Clamp zoom to min/max values
        camera_controller.zoom = camera_controller.zoom.clamp(min_zoom, max_zoom);
    }
}

fn gun_control(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut aim_state: ResMut<AimState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_query: Query<(&Transform, &mut PlayerState), With<Player>>,
    gun_query: Query<Entity, With<Gun>>,
) {
    let gun_drawn = mouse_input.pressed(MouseButton::Right);

    if gun_drawn != aim_state.gun_drawn {
        aim_state.gun_drawn = gun_drawn;

        if let Ok((player_transform, mut player_state)) = player_query.get_single_mut() {
            player_state.is_aiming = gun_drawn;

            if gun_drawn {
                // Draw gun
                let gun_mesh = meshes.add(Cuboid::new(0.1, 0.1, 0.6));
                let gun_material = materials.add(Color::rgb(0.3, 0.3, 0.3));

                commands.spawn((
                    PbrBundle {
                        mesh: gun_mesh,
                        material: gun_material,
                        transform: Transform::from_xyz(
                            player_transform.translation.x + 0.5,
                            player_transform.translation.y + 0.2,
                            player_transform.translation.z,
                        ),
                        ..default()
                    },
                    Gun,
                ));
            } else {
                // Holster gun
                for gun_entity in gun_query.iter() {
                    commands.entity(gun_entity).despawn();
                }
            }
        }
    }

    // Update gun position to follow player
    if gun_drawn {
        if let Ok((player_transform, _)) = player_query.get_single() {
            for gun_entity in gun_query.iter() {
                commands.entity(gun_entity).insert(Transform::from_xyz(
                    player_transform.translation.x + 0.5,
                    player_transform.translation.y + 0.2,
                    player_transform.translation.z,
                ));
            }
        }
    }
}

fn aim_system(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut aim_state: ResMut<AimState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    target_indicator_query: Query<Entity, With<TargetIndicator>>,
) {
    // Clear existing target indicator
    for indicator_entity in target_indicator_query.iter() {
        commands.entity(indicator_entity).despawn();
    }

    aim_state.target_enemy = None;

    if !aim_state.gun_drawn {
        return;
    }

    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    if let Some(cursor_position) = window.cursor_position() {
        // Cast ray from camera through cursor position
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            let mut closest_distance = f32::INFINITY;
            let mut closest_enemy = None;

            // Check for enemies within aim assist range
            for (enemy_entity, enemy_transform) in enemy_query.iter() {
                let enemy_pos = enemy_transform.translation;

                // Calculate distance from ray to enemy
                let ray_to_enemy = enemy_pos - ray.origin;
                let projection = ray_to_enemy.dot(*ray.direction);

                if projection > 0.0 {
                    let closest_point = ray.origin + *ray.direction * projection;
                    let distance_to_ray = (enemy_pos - closest_point).length();

                    // Aim assist: snap to enemy if cursor is close enough
                    if distance_to_ray < 2.0 && projection < closest_distance {
                        closest_distance = projection;
                        closest_enemy = Some(enemy_entity);
                    }
                }
            }

            // If we found a target, show indicator and set target
            if let Some(target_entity) = closest_enemy {
                aim_state.target_enemy = Some(target_entity);

                // Get target position for indicator
                if let Ok((_, target_transform)) = enemy_query.get(target_entity) {
                    let indicator_mesh = meshes.add(Torus::new(0.3, 0.1));
                    let indicator_material = materials.add(Color::rgb(1.0, 0.0, 0.0)); // Red target indicator

                    commands.spawn((
                        PbrBundle {
                            mesh: indicator_mesh,
                            material: indicator_material,
                            transform: Transform::from_xyz(
                                target_transform.translation.x,
                                target_transform.translation.y + 1.0,
                                target_transform.translation.z,
                            ),
                            ..default()
                        },
                        TargetIndicator,
                    ));
                }
            }
        }
    }
}

// Add a shooting timer component
#[derive(Component)]
struct ShootingTimer(Timer);

fn shooting_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    aim_state: Res<AimState>,
    mut commands: Commands,
    enemy_query: Query<Entity, With<Enemy>>,
    mut player_query: Query<(Entity, &mut PlayerState, Option<&mut ShootingTimer>), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((player_entity, mut player_state, shooting_timer)) = player_query.get_single_mut() {
        // Handle shooting timer
        if let Some(mut timer) = shooting_timer {
            timer.0.tick(time.delta());
            if timer.0.finished() {
                player_state.is_shooting = false;
                commands.entity(player_entity).remove::<ShootingTimer>();
            }
        }

        // Handle new shots
        if mouse_input.just_pressed(MouseButton::Left) && aim_state.gun_drawn {
            if let Some(target_entity) = aim_state.target_enemy {
                // Check if the target still exists (in case it was already shot)
                if enemy_query.get(target_entity).is_ok() {
                    // Despawn the enemy
                    commands.entity(target_entity).despawn();

                    // Set shooting state and timer
                    player_state.is_shooting = true;
                    commands
                        .entity(player_entity)
                        .insert(ShootingTimer(Timer::from_seconds(0.5, TimerMode::Once)));
                }
            }
        }
    }
}

fn spawn_player_when_loaded(
    mut commands: Commands,
    player_query: Query<Entity, (With<Player>, With<Handle<Mesh>>)>,
    player_assets: Res<PlayerAssets>,
    asset_server: Res<AssetServer>,
) {
    // Check if the cowboy scene is loaded
    if let Some(LoadState::Loaded) = asset_server.get_load_state(&player_assets.cowboy_scene) {
        // Replace cube with 3D model
        if let Ok(player_entity) = player_query.get_single() {
            commands.entity(player_entity).remove::<Handle<Mesh>>();
            commands
                .entity(player_entity)
                .remove::<Handle<StandardMaterial>>();

            // Spawn the cowboy scene as a child
            let scene_entity = commands
                .spawn(SceneBundle {
                    scene: player_assets.cowboy_scene.clone(),
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    ..default()
                })
                .id();

            // Make the scene a child of the player entity
            commands
                .entity(player_entity)
                .push_children(&[scene_entity]);
        }
    }
}

fn handle_animations(
    mut animation_players: Query<&mut AnimationPlayer>,
    player_query: Query<(Entity, &PlayerState, &Children), (With<Player>, Changed<PlayerState>)>,
    children_query: Query<&Children>,
    player_assets: Res<PlayerAssets>,
    asset_server: Res<AssetServer>,
) {
    for (player_entity, player_state, children) in player_query.iter() {
        // Find the AnimationPlayer in the scene hierarchy
        let mut animation_player_entity = None;

        // Check if player itself has AnimationPlayer
        if animation_players.get(player_entity).is_ok() {
            animation_player_entity = Some(player_entity);
        } else {
            // Recursively search through children for AnimationPlayer
            fn find_animation_player(
                entity: Entity,
                children_query: &Query<&Children>,
                animation_players: &Query<&mut AnimationPlayer>,
            ) -> Option<Entity> {
                if animation_players.get(entity).is_ok() {
                    return Some(entity);
                }

                if let Ok(children) = children_query.get(entity) {
                    for &child in children.iter() {
                        if let Some(found) =
                            find_animation_player(child, children_query, animation_players)
                        {
                            return Some(found);
                        }
                    }
                }
                None
            }

            for &child in children.iter() {
                if let Some(found) =
                    find_animation_player(child, &children_query, &animation_players)
                {
                    animation_player_entity = Some(found);
                    break;
                }
            }
        }

        if let Some(anim_entity) = animation_player_entity {
            if let Ok(mut animation_player) = animation_players.get_mut(anim_entity) {
                // Determine which animation to play based on priority
                let new_animation = if player_state.is_shooting {
                    &player_assets.shooting_animation
                } else if player_state.is_aiming {
                    &player_assets.aiming_animation
                } else if player_state.is_running {
                    &player_assets.running_animation
                } else if player_state.is_moving {
                    &player_assets.walking_animation
                } else {
                    &player_assets.idle_animation
                };

                // Check if all animations are loaded before playing
                if let Some(LoadState::Loaded) = asset_server.get_load_state(new_animation) {
                    // Only change animation if it's different from current
                    if !animation_player.is_playing_clip(new_animation) {
                        animation_player.play(new_animation.clone()).repeat();
                    }
                }
            }
        }
    }
}

fn toggle_camera_view(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_view: ResMut<CameraViewResource>,
    mut windows: Query<&mut Window>,
) {
    if keyboard_input.just_pressed(KeyCode::F5) {
        let new_view = match camera_view.0 {
            CameraView::Isometric => CameraView::FirstPerson,
            CameraView::FirstPerson => CameraView::Isometric,
        };
        
        camera_view.0 = new_view;
        
        // Handle mouse cursor grab based on camera mode
        if let Ok(mut window) = windows.get_single_mut() {
            match new_view {
                CameraView::FirstPerson => {
                    window.cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
                    window.cursor.visible = false;
                }
                CameraView::Isometric => {
                    window.cursor.grab_mode = bevy::window::CursorGrabMode::None;
                    window.cursor.visible = true;
                }
            }
        }
    }
}

fn toggle_chunk_borders(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut chunk_border_visible: ResMut<ChunkBorderVisible>,
) {
    if keyboard_input.just_pressed(KeyCode::F3) {
        chunk_border_visible.0 = !chunk_border_visible.0;
    }
}
