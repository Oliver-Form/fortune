use bevy::asset::LoadState;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::collections::HashMap;

const TILE_SIZE: f32 = 1.0;
const WORLD_SIZE: i32 = 100; // 100x100 world
const CHUNK_SIZE: i32 = 16; // 16x16 tiles per chunk (larger chunks for better performance)
const RENDER_DISTANCE: i32 = 3; // Only render chunks within 3 chunk radius
struct COLORS;
impl COLORS {
    pub const TAN: Color = Color::rgb(0.8, 0.7, 0.4);
    pub const GREEN: Color = Color::rgb(0.2, 0.6, 0.2);
    pub const BLUE: Color = Color::rgb(0.2, 0.4, 0.8);
}

// Components
#[derive(Component)]
struct Player;

#[derive(Component)]
struct PlayerState {
    is_moving: bool,
    is_running: bool,
    is_aiming: bool,
    is_shooting: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            is_moving: false,
            is_running: false,
            is_aiming: false,
            is_shooting: false,
        }
    }
}

#[derive(Component)]
struct CameraController {
    zoom: f32,
}

#[derive(Component)]
struct WorldTile {
    chunk_pos: IVec2,
    tile_pos: IVec2,
}

#[derive(Component)]
struct ChunkEntity;

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

// Data structures
struct ChunkData {
    position: IVec2,
    color: Color,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TileType {
    Desert,    // Tan
    Grassland, // Green
    Water,     // Blue
}

impl TileType {
    fn get_name(&self) -> &'static str {
        match self {
            TileType::Desert => "Desert",
            TileType::Grassland => "Grassland",
            TileType::Water => "Water",
        }
    }

    fn get_color(&self) -> Color {
        match self {
            TileType::Desert => COLORS::TAN,
            TileType::Grassland => COLORS::GREEN,
            TileType::Water => COLORS::BLUE,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .insert_resource(MapVisible(false))
        .insert_resource(FpsTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .insert_resource(AimState {
            target_enemy: None,
            gun_drawn: false,
        })
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
    // Initialize empty world data
    let world_data = WorldData {
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
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::rgb(0.8, 0.2, 0.2)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
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
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut PlayerState), With<Player>>,
    camera_query: Query<&Transform, (With<CameraController>, Without<Player>)>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut player_state)) = player_query.get_single_mut() {
        let mut direction = Vec3::ZERO;
        let base_speed = 5.0;
        let run_multiplier = 2.0;

        // Check if running (Shift key held)
        let is_running = keyboard_input.pressed(KeyCode::ShiftLeft)
            || keyboard_input.pressed(KeyCode::ShiftRight);

        // Get camera transform to calculate relative movement
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
                _ => return, // Don't change rotation if no clear direction
            };

            // Convert degrees to radians and apply rotation
            if direction.length() > 0.0 {
                let target_angle_radians = target_angle_degrees.to_radians();
                let target_rotation = Quat::from_rotation_y(target_angle_radians);
                transform.rotation = transform.rotation.slerp(target_rotation, 10.0 * time.delta_seconds());
            }
        }

        // Update movement state
        player_state.is_moving = direction.length() > 0.0;
        player_state.is_running = player_state.is_moving && is_running;

        // Normalize direction and apply movement
        if direction.length() > 0.0 {
            direction = direction.normalize();
            
            let speed = if is_running { base_speed * run_multiplier } else { base_speed };
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
    mut camera_query: Query<
        (&mut Transform, &CameraController),
        (With<CameraController>, Without<Player>),
    >,
) {
    if let (Ok(player_transform), Ok((mut camera_transform, camera_controller))) =
        (player_query.get_single(), camera_query.get_single_mut())
    {
        // Use zoom value for offset distance
        let zoom_distance = camera_controller.zoom;
        let offset = Vec3::new(zoom_distance, zoom_distance, zoom_distance);
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

fn toggle_map(keyboard_input: Res<ButtonInput<KeyCode>>, mut map_visible: ResMut<MapVisible>) {
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

fn get_biome_at_position(_world_data: &WorldData, world_pos: Vec3) -> TileType {
    let chunk_x = (world_pos.x / (CHUNK_SIZE as f32 * TILE_SIZE)) as i32;
    let chunk_z = (world_pos.z / (CHUNK_SIZE as f32 * TILE_SIZE)) as i32;

    // Check if chunk is within world boundaries
    let max_chunk = (WORLD_SIZE / CHUNK_SIZE) - 1;
    if chunk_x < 0 || chunk_x > max_chunk || chunk_z < 0 || chunk_z > max_chunk {
        return TileType::Desert; // Default to desert outside world bounds
    }

    // Use the same noise logic as world generation
    let noise = Perlin::new(42);
    let noise_value = noise.get([chunk_x as f64 * 0.1, chunk_z as f64 * 0.1]);

    match noise_value {
        n if n < -0.3 => TileType::Water,
        n if n < 0.2 => TileType::Desert,
        _ => TileType::Grassland,
    }
}

fn manage_world_chunks(
    player_query: Query<&Transform, With<Player>>,
    mut world_data: ResMut<WorldData>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _chunk_query: Query<(Entity, &ChunkEntity)>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let player_chunk = IVec2::new(
            (player_transform.translation.x / (CHUNK_SIZE as f32 * TILE_SIZE)) as i32,
            (player_transform.translation.z / (CHUNK_SIZE as f32 * TILE_SIZE)) as i32,
        );

        // Calculate world boundaries in chunks
        let max_chunk = (WORLD_SIZE / CHUNK_SIZE) - 1;

        // Generate and spawn chunks around player
        for x in -RENDER_DISTANCE..=RENDER_DISTANCE {
            for z in -RENDER_DISTANCE..=RENDER_DISTANCE {
                let chunk_pos = player_chunk + IVec2::new(x, z);

                // Check if chunk is within world boundaries
                if chunk_pos.x >= 0
                    && chunk_pos.x <= max_chunk
                    && chunk_pos.y >= 0
                    && chunk_pos.y <= max_chunk
                {
                    if !world_data.chunks.contains_key(&chunk_pos) {
                        let chunk_data = generate_chunk(chunk_pos);
                        spawn_chunk(&chunk_data, &mut commands, &mut meshes, &mut materials);
                        world_data.chunks.insert(chunk_pos, chunk_data);
                    }
                }
            }
        }

        // TODO: Remove chunks that are too far away (implement later for infinite world)
    }
}

fn generate_chunk(chunk_pos: IVec2) -> ChunkData {
    let noise = Perlin::new(42);
    let noise_value = noise.get([chunk_pos.x as f64 * 0.1, chunk_pos.y as f64 * 0.1]);

    let tile_type = match noise_value {
        n if n < -0.3 => TileType::Water,
        n if n < 0.2 => TileType::Desert,
        _ => TileType::Grassland,
    };

    ChunkData {
        position: chunk_pos,
        color: tile_type.get_color(),
    }
}

fn spawn_chunk(
    chunk_data: &ChunkData,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Create a larger mesh for the entire chunk instead of individual tiles
    let chunk_mesh = meshes.add(
        Plane3d::default()
            .mesh()
            .size(CHUNK_SIZE as f32 * TILE_SIZE, CHUNK_SIZE as f32 * TILE_SIZE),
    );
    let chunk_material = materials.add(chunk_data.color);

    let world_x = chunk_data.position.x as f32 * CHUNK_SIZE as f32 * TILE_SIZE;
    let world_z = chunk_data.position.y as f32 * CHUNK_SIZE as f32 * TILE_SIZE;

    // Spawn single entity for entire chunk
    commands.spawn((
        PbrBundle {
            mesh: chunk_mesh,
            material: chunk_material,
            transform: Transform::from_xyz(
                world_x + (CHUNK_SIZE as f32 * TILE_SIZE) / 2.0,
                0.0,
                world_z + (CHUNK_SIZE as f32 * TILE_SIZE) / 2.0,
            ),
            ..default()
        },
        ChunkEntity,
    ));

    // Add decorations
    spawn_chunk_decorations(chunk_data, commands, meshes, materials);
}

fn spawn_chunk_decorations(
    chunk_data: &ChunkData,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let cactus_mesh = meshes.add(Cuboid::new(0.2, 1.5, 0.2));
    let tree_mesh = meshes.add(Cuboid::new(0.3, 2.0, 0.3));
    let enemy_mesh = meshes.add(Cuboid::new(0.8, 1.8, 0.8));
    let cactus_material = materials.add(Color::rgb(0.2, 0.7, 0.2));
    let tree_material = materials.add(Color::rgb(0.4, 0.2, 0.1));
    let enemy_material = materials.add(Color::rgb(0.6, 0.2, 0.8)); // Purple enemies

    let decoration_noise = Perlin::new(123);
    let enemy_noise = Perlin::new(456);

    // Determine chunk biome
    let noise = Perlin::new(42);
    let noise_value = noise.get([
        chunk_data.position.x as f64 * 0.1,
        chunk_data.position.y as f64 * 0.1,
    ]);
    let biome = match noise_value {
        n if n < -0.3 => TileType::Water,
        n if n < 0.2 => TileType::Desert,
        _ => TileType::Grassland,
    };

    // Only add a few decorations per chunk to keep performance good
    for i in 0..3 {
        for j in 0..3 {
            let sample_x = chunk_data.position.x * CHUNK_SIZE + i * (CHUNK_SIZE / 3);
            let sample_z = chunk_data.position.y * CHUNK_SIZE + j * (CHUNK_SIZE / 3);

            let decoration_value = decoration_noise.get([
                sample_x as f64 * 0.3 + 1000.0,
                sample_z as f64 * 0.3 + 1000.0,
            ]);

            let enemy_value = enemy_noise.get([
                sample_x as f64 * 0.25 + 2000.0,
                sample_z as f64 * 0.25 + 2000.0,
            ]);

            // Spawn enemies sporadically across all biomes (except water)
            if biome != TileType::Water && enemy_value > 0.8 {
                commands.spawn((
                    PbrBundle {
                        mesh: enemy_mesh.clone(),
                        material: enemy_material.clone(),
                        transform: Transform::from_xyz(
                            sample_x as f32 * TILE_SIZE,
                            0.9,
                            sample_z as f32 * TILE_SIZE,
                        ),
                        ..default()
                    },
                    Enemy,
                ));
            }

            match biome {
                TileType::Desert => {
                    if decoration_value > 0.7 {
                        commands.spawn((
                            PbrBundle {
                                mesh: cactus_mesh.clone(),
                                material: cactus_material.clone(),
                                transform: Transform::from_xyz(
                                    sample_x as f32 * TILE_SIZE,
                                    0.75,
                                    sample_z as f32 * TILE_SIZE,
                                ),
                                ..default()
                            },
                            Cactus,
                        ));
                    }
                }
                TileType::Grassland => {
                    if decoration_value > 0.85 {
                        commands.spawn((
                            PbrBundle {
                                mesh: tree_mesh.clone(),
                                material: tree_material.clone(),
                                transform: Transform::from_xyz(
                                    sample_x as f32 * TILE_SIZE,
                                    1.0,
                                    sample_z as f32 * TILE_SIZE,
                                ),
                                ..default()
                            },
                            Tree,
                        ));
                    }
                }
                TileType::Water => {}
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
