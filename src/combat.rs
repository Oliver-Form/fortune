use bevy::prelude::*;
use crate::{components::*, constants::*, resources::*};

pub fn gun_control(
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

pub fn aim_system(
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

pub fn shooting_system(
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
