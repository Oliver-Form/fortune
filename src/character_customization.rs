use bevy::prelude::*;
use crate::{components::*, resources::*};

#[derive(Component)]
pub struct CharacterCustomizationUI;

#[derive(Component)]
pub struct BackToMenuButton;

#[derive(Component)]
pub struct CustomizationCamera;

#[derive(Component)]
pub struct CustomizationCharacter;

pub fn setup_character_customization(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
) {
    // Create a black background
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: Color::BLACK.into(),
            z_index: ZIndex::Global(500),
            ..default()
        },
        CharacterCustomizationUI,
    ));

    // Create UI for character customization
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexStart,
                    flex_direction: FlexDirection::Row,
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                z_index: ZIndex::Global(600),
                ..default()
            },
            CharacterCustomizationUI,
        ))
        .with_children(|parent| {
            // Left side - UI controls
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(50.0), // Use 50% of screen width
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // Title
                    parent.spawn(TextBundle::from_section(
                        "Character Customization",
                        TextStyle {
                            font_size: 32.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ).with_style(Style {
                        margin: UiRect::bottom(Val::Px(30.0)),
                        ..default()
                    }));

                    // Placeholder for customization options
                    parent.spawn(TextBundle::from_section(
                        "Customization options will be added here...",
                        TextStyle {
                            font_size: 16.0,
                            color: Color::GRAY,
                            ..default()
                        },
                    ).with_style(Style {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    }));

                    // Back to Menu Button
                    parent
                        .spawn((
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(200.0),
                                    height: Val::Px(50.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::top(Val::Px(30.0)),
                                    ..default()
                                },
                                background_color: Color::rgb(0.5, 0.5, 0.5).into(),
                                ..default()
                            },
                            BackToMenuButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Back to Menu",
                                TextStyle {
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ));
                        });
                });

            // Right side - Character display area (empty node to reserve space)
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(50.0), // Use the remaining 50% of screen width
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            });
        });

    // Create a camera for character viewing (positioned to show character on right side)
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(2.0, 1.5, 3.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            camera: Camera {
                order: 1, // Higher order so it renders on top
                ..default()
            },
            ..default()
        },
        CustomizationCamera,
    ));

    // Spawn the character for customization
    commands.spawn((
        SceneBundle {
            scene: player_assets.cowboy_scene.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        CustomizationCharacter,
    ));

    // Add lighting for the character
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                illuminance: 10000.0,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(2.0, 2.0, 2.0),
                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)
                    * Quat::from_rotation_y(-std::f32::consts::FRAC_PI_4),
                ..default()
            },
            ..default()
        },
        CustomizationCharacter, // Tag it so we can clean it up
    ));
}

pub fn cleanup_character_customization(
    mut commands: Commands,
    customization_query: Query<Entity, Or<(With<CharacterCustomizationUI>, With<CustomizationCamera>, With<CustomizationCharacter>)>>,
) {
    for entity in customization_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn handle_character_customization_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&BackToMenuButton>),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color, back_button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if back_button.is_some() {
                    next_state.set(GameState::Paused);
                }
            }
            Interaction::Hovered => {
                *color = Color::rgb(0.7, 0.7, 0.7).into();
            }
            Interaction::None => {
                *color = Color::rgb(0.5, 0.5, 0.5).into();
            }
        }
    }
}

pub fn play_idle_animation(
    mut animation_players: Query<&mut AnimationPlayer>,
    customization_character_query: Query<Entity, (With<CustomizationCharacter>, With<Children>)>,
    children_query: Query<&Children>,
    player_assets: Res<PlayerAssets>,
    asset_server: Res<AssetServer>,
) {
    for character_entity in customization_character_query.iter() {
        // Find the AnimationPlayer in the scene hierarchy
        let mut animation_player_entity = None;

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

        if let Ok(children) = children_query.get(character_entity) {
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
                // Check if idle animation is loaded before playing
                if let Some(bevy::asset::LoadState::Loaded) = asset_server.get_load_state(&player_assets.idle_animation) {
                    // Only start animation if it's not already playing
                    if !animation_player.is_playing_clip(&player_assets.idle_animation) {
                        animation_player.play(player_assets.idle_animation.clone()).repeat();
                    }
                }
            }
        }
    }
}
