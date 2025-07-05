use bevy::prelude::*;
use bevy::ecs::system::ParamSet;
use crate::{components::*, constants::*, resources::*, world::get_tile_at_position};

pub fn toggle_map(keyboard_input: Res<ButtonInput<KeyCode>>, mut map_visible: ResMut<MapVisible>) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        map_visible.0 = !map_visible.0;
    }
}

pub fn update_map_display(
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

pub fn update_fps(
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

pub fn update_biome_display(
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
