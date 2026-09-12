use crate::types::GameMode;
use bevy::prelude::*;

pub fn camera_movement_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mode: Res<GameMode>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    if *mode != GameMode::Free {
        return;
    }
    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };
    let delta = time.delta_secs();
    let speed = 12.0 * delta;
    let rotation_speed = 2.0 * delta;

    // 1. View Rotation Handling (Hold R and press A/D to rotate left/right)
    if keyboard.pressed(KeyCode::KeyR) {
        let mut yaw_change = 0.0;
        let mut pitch_change = 0.0;
        if keyboard.pressed(KeyCode::KeyA) {
            yaw_change += rotation_speed;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            yaw_change -= rotation_speed;
        }
        if yaw_change != 0.0 {
            transform.rotate_local_y(yaw_change);
        }
        if keyboard.pressed(KeyCode::KeyW) {
            pitch_change += rotation_speed;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            pitch_change -= rotation_speed;
        }
        if pitch_change != 0.0 {
            transform.rotate_local_x(pitch_change);
        }
    }
    let mut cam_change = 0.0;
    if keyboard.pressed(KeyCode::KeyE) {
        cam_change += rotation_speed;
    }
    if keyboard.pressed(KeyCode::KeyQ) {
        cam_change -= rotation_speed;
    }
    if cam_change != 0.0 {
        transform.rotate_local_z(cam_change);
    }

    let forward = transform.forward();
    let right = transform.right();

    let mut move_dir = Vec3::ZERO;

    // 2. Translation Handling (WASD for horizontal plane, Q/E for vertical elevation)
    // Note: If R is held for rotation, WASD still moves you relative to your facing direction.
    // Only use A/D for strafing if R isn't held (otherwise A/D is used for rotation)
    if !keyboard.pressed(KeyCode::KeyR) {
        if keyboard.pressed(KeyCode::KeyA) {
            move_dir -= Vec3::new(right.x, 0.0, right.z).normalize_or_zero();
        }
        if keyboard.pressed(KeyCode::KeyD) {
            move_dir += Vec3::new(right.x, 0.0, right.z).normalize_or_zero();
        }
        if keyboard.pressed(KeyCode::KeyW) {
            move_dir += Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        }
        if keyboard.pressed(KeyCode::KeyS) {
            move_dir -= Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        }
    }
    if keyboard.pressed(KeyCode::Space) {
        move_dir.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        move_dir.y -= 1.0;
    }

    transform.translation += move_dir * speed;
}
