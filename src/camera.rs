// ============================================================================
// Camera Movement System
// ============================================================================

use crate::types::GameMode;
use bevy::prelude::*;

/// Handles all free-flying camera controls for the player.
///
/// This system provides a classic "noclip" or spectator style camera, allowing
/// the user to navigate the 3D space freely. It supports:
/// - Horizontal translation (WASD)
/// - Vertical elevation (Space/Shift)
/// - Pitch and Yaw rotation (Holding 'R' + WASD)
/// - Roll rotation (Q/E)
pub fn camera_movement_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mode: Res<GameMode>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    // ------------------------------------------------------------------------
    // Guard Clauses & Setup
    // ------------------------------------------------------------------------

    // The free camera should only be active if the player is in Free Mode.
    // In Human mode, a different system (likely player-attached) handles the camera.
    if *mode != GameMode::Free {
        return;
    }

    // Attempt to grab the single 3D camera in the scene. If it doesn't exist, exit early.
    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };

    // Base speeds scaled by delta time for frame-rate independent movement.
    let delta = time.delta_secs();
    let speed = 12.0 * delta; // Translation speed (units per second)
    let rotation_speed = 2.0 * delta; // Rotation speed (radians per second)

    // ------------------------------------------------------------------------
    // 1. View Rotation Handling (Pitch, Yaw, and Roll)
    // ------------------------------------------------------------------------

    // Holding the 'R' key shifts the WASD keys from translation (movement) to rotation.
    if keyboard.pressed(KeyCode::KeyR) {
        let mut yaw_change = 0.0; // Left/Right rotation (Y-axis)
        let mut pitch_change = 0.0; // Up/Down rotation (X-axis)

        // A/D controls Yaw (looking left/right)
        if keyboard.pressed(KeyCode::KeyA) {
            yaw_change += rotation_speed;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            yaw_change -= rotation_speed;
        }
        // Apply Yaw locally so it feels natural to the camera's current orientation
        if yaw_change != 0.0 {
            transform.rotate_local_y(yaw_change);
        }

        // W/S controls Pitch (looking up/down)
        if keyboard.pressed(KeyCode::KeyW) {
            pitch_change += rotation_speed;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            pitch_change -= rotation_speed;
        }
        // Apply Pitch locally
        if pitch_change != 0.0 {
            transform.rotate_local_x(pitch_change);
        }
    }

    // Q/E controls Roll (tilting the camera sideways around the Z-axis).
    // Note: This is independent of the 'R' key modifier.
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

    // ------------------------------------------------------------------------
    // 2. Translation Handling (Movement)
    // ------------------------------------------------------------------------

    // Extract the camera's current local forward and right vectors.
    // These tell us which way the camera is currently facing in the 3D world.
    let forward = transform.forward();
    let right = transform.right();

    // Accumulator vector for all movement input this frame
    let mut move_dir = Vec3::ZERO;

    // Only process WASD translation if the 'R' key (rotation modifier) is NOT pressed.
    if !keyboard.pressed(KeyCode::KeyR) {
        // A/D for left/right strafing.
        // We set the Y component to 0.0 and normalize so that looking down and
        // pressing 'W' or 'A' doesn't push the camera into the ground.
        // Movement is constrained to the horizontal plane.
        if keyboard.pressed(KeyCode::KeyA) {
            move_dir -= Vec3::new(right.x, 0.0, right.z).normalize_or_zero();
        }
        if keyboard.pressed(KeyCode::KeyD) {
            move_dir += Vec3::new(right.x, 0.0, right.z).normalize_or_zero();
        }

        // W/S for forward/backward movement.
        if keyboard.pressed(KeyCode::KeyW) {
            move_dir += Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        }
        if keyboard.pressed(KeyCode::KeyS) {
            move_dir -= Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        }
    }

    // Space and Shift control absolute vertical elevation (global Y-axis).
    // Space moves the camera up, ShiftLeft moves it down.
    if keyboard.pressed(KeyCode::Space) {
        move_dir.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        move_dir.y -= 1.0;
    }

    // Apply the accumulated movement direction, scaled by the calculated speed.
    // If multiple keys are pressed, move_dir might have a length > 1.0,
    // but typically in free-cams this slight diagonal speed boost is acceptable.
    transform.translation += move_dir * speed;
}
