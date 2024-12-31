use bevy::{prelude::*, window::PrimaryWindow};

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
pub struct MouseWorldCoords(pub Vec2);

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

pub fn mouse_world_pos_update(
    mut mycoords: ResMut<MouseWorldCoords>,
    // Query to get the primary window
    q_window: Query<&Window, With<PrimaryWindow>>,
    // Query to get camera + its transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // Single camera => we can use .single()
    let (camera, camera_transform) = q_camera.single();

    // Single primary window => we can also use .single()
    let window = q_window.single();

    // If the cursor is inside the window, convert its position to world coordinates
    if let Some(world_position) = window
        .cursor_position()
        .map(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.unwrap().origin.truncate())
    {
        mycoords.0 = world_position;
    }
}
