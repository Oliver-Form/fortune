use bevy::prelude::*;
use crate::{components::*, constants::*, resources::*};

pub fn camera_follow(
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

pub fn camera_zoom(
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

pub fn toggle_camera_view(
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
