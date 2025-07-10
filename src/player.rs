use bevy::prelude::*;
use crate::{components::*, constants::*, resources::*};

pub fn player_movement(
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

pub fn spawn_player_when_loaded(
    mut commands: Commands,
    player_query: Query<Entity, (With<Player>, With<Handle<Mesh>>)>,
    player_assets: Res<PlayerAssets>,
    asset_server: Res<AssetServer>,
) {
    // Check if the cowboy scene is loaded
    if let Some(bevy::asset::LoadState::Loaded) = asset_server.get_load_state(&player_assets.cowboy_scene) {
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

pub fn handle_animations(
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
                if let Some(bevy::asset::LoadState::Loaded) = asset_server.get_load_state(new_animation) {
                    // Only change animation if it's different from current
                    if !animation_player.is_playing_clip(new_animation) {
                        animation_player.play(new_animation.clone()).repeat();
                    }
                }
            }
        }
    }
}
