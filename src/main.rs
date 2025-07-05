mod constants;
mod components;
mod resources;
mod player;
mod camera;
mod world;
mod ui;
mod combat;
mod setup;

use constants::*;
use components::*;
use resources::*;
use player::*;
use camera::*;
use world::*;
use ui::*;
use combat::*;
use setup::*;

use bevy::prelude::*;
use bevy::window::PresentMode;

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
        .init_state::<GameState>()
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
                toggle_pause_menu,
                handle_pause_menu_buttons,
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
                    spawn_player_when_loaded,
                    handle_animations,
                    toggle_camera_view,
                    toggle_chunk_borders,
                ).run_if(in_state(GameState::Playing)),
                update_fps,
                update_biome_display,
            ),
        )
        .run();
}
