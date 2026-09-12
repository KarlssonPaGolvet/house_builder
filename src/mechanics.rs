use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

use crate::types::{GameMode, GameState, HumanPhysics, PlacedBlock};

const HUMAN_HEIGHT: f32 = 1.75;
const HUMAN_HALF_WIDTH: f32 = 0.375;
const STEP_HEIGHT: f32 = 0.51;
const MIN_STEP_OFFSET: f32 = 0.051;
const CONTACT_EPSILON: f32 = 0.002;
const JUMP_HEIGHT: f32 = 3.0;
const GRAVITY: f32 = 24.0;
const WALK_SPEED: f32 = 5.0;
const MOUSE_SENSITIVITY: f32 = 0.0025;
const GROUND_HALF_SIZE: f32 = 100.0;

pub fn return_to_start_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut physics: ResMut<HumanPhysics>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    if !keyboard.just_pressed(KeyCode::F1) {
        return;
    }
    let Ok(mut camera) = camera_query.single_mut() else {
        return;
    };
    *camera = Transform::from_xyz(-8.0, 10.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y);
    physics.vertical_velocity = 0.0;
}

pub fn human_mouse_look_system(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mode: Res<GameMode>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    if *mode != GameMode::Human {
        return;
    }

    let Ok(mut camera) = camera_query.single_mut() else {
        return;
    };
    let motion = mouse_motion.delta;
    if motion == Vec2::ZERO {
        return;
    }

    camera.rotate_y(-motion.x * MOUSE_SENSITIVITY);
    camera.rotate_local_x(-motion.y * MOUSE_SENSITIVITY);
}

pub fn cursor_mode_system(
    mode: Res<GameMode>,
    state: Res<State<GameState>>,
    mut windows: Query<&mut CursorOptions>,
) {
    let Ok(mut cursor) = windows.single_mut() else {
        return;
    };
    let human_playing = *mode == GameMode::Human && *state.get() == GameState::Playing;
    cursor.visible = !human_playing;
    cursor.grab_mode = if human_playing {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
}

pub fn human_movement_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mode: Res<GameMode>,
    mut physics: ResMut<HumanPhysics>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    blocks: Query<&PlacedBlock>,
) {
    if *mode != GameMode::Human {
        return;
    }

    let Ok(mut camera) = camera_query.single_mut() else {
        return;
    };
    let delta = time.delta_secs();
    let forward = camera.forward();
    let right = camera.right();
    let horizontal_forward = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let horizontal_right = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

    let mut direction = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        direction += horizontal_forward;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction -= horizontal_forward;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += horizontal_right;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= horizontal_right;
    }
    let direction = direction.normalize_or_zero();
    let movement = direction * WALK_SPEED * delta;
    let mut feet_y = camera.translation.y - HUMAN_HEIGHT;
    let support = support_height(camera.translation, feet_y, &blocks);
    let grounded = physics.vertical_velocity.abs() < 0.01 && feet_y <= support + 0.05;

    let jumping = keyboard.just_pressed(KeyCode::Space) && grounded;
    if jumping {
        physics.vertical_velocity = (2.0 * GRAVITY * JUMP_HEIGHT).sqrt();
    }
    let next_feet_y = if grounded && !jumping {
        physics.vertical_velocity = 0.0;
        feet_y
    } else {
        physics.vertical_velocity -= GRAVITY * delta;
        feet_y + physics.vertical_velocity * delta
    };

    let mut candidate = camera.translation;
    for axis_movement in [
        Vec3::new(movement.x, 0.0, 0.0),
        Vec3::new(0.0, 0.0, movement.z),
    ] {
        let next = candidate + axis_movement;
        let step_target = step_support(next, feet_y, axis_movement, &blocks);
        let step = step_target.map(|(top, _)| top - feet_y).unwrap_or(0.0);
        let can_step = grounded && step > 0.0 && step <= STEP_HEIGHT;
        let clear_to_step = can_step
            && step_target.is_some_and(|(_, support_center)| {
                step_clear(next, feet_y + step, support_center, axis_movement, &blocks)
            });
        let can_move =
            !can_step && !intersects_swept(candidate, next, next_feet_y, HUMAN_HEIGHT, &blocks);
        if clear_to_step || can_move {
            candidate = next;
            if clear_to_step {
                feet_y += step;
            }
        }
    }

    if physics.vertical_velocity <= 0.0 {
        let landing = highest_support(candidate, feet_y, next_feet_y, &blocks);
        if next_feet_y <= landing {
            feet_y = landing;
            physics.vertical_velocity = 0.0;
        } else {
            feet_y = next_feet_y;
        }
    } else {
        feet_y = next_feet_y;
    }

    camera.translation = Vec3::new(candidate.x, feet_y + HUMAN_HEIGHT, candidate.z);
    camera.translation.x = camera.translation.x.clamp(
        -GROUND_HALF_SIZE + HUMAN_HALF_WIDTH,
        GROUND_HALF_SIZE - HUMAN_HALF_WIDTH,
    );
    camera.translation.z = camera.translation.z.clamp(
        -GROUND_HALF_SIZE + HUMAN_HALF_WIDTH,
        GROUND_HALF_SIZE - HUMAN_HALF_WIDTH,
    );
}

fn support_height(position: Vec3, feet_y: f32, blocks: &Query<&PlacedBlock>) -> f32 {
    let mut support: f32 = 0.0;
    for block in blocks.iter() {
        let half_extents = world_half_extents(block);
        let top = block.center.y + half_extents.y;
        if overlaps_xz(position, block.center, half_extents * 2.0)
            && top <= feet_y + STEP_HEIGHT + 0.001
        {
            support = support.max(top);
        }
    }
    support
}

fn step_support(
    position: Vec3,
    feet_y: f32,
    movement: Vec3,
    blocks: &Query<&PlacedBlock>,
) -> Option<(f32, Vec3)> {
    let mut support: Option<(f32, Vec3)> = None;
    for block in blocks.iter() {
        let half_extents = world_half_extents(block);
        let top = block.center.y + half_extents.y;
        if top > feet_y
            && top <= feet_y + STEP_HEIGHT + 0.001
            && overlaps_xz(position, block.center, half_extents * 2.0)
            && (support.is_none() || top > support.unwrap().0)
        {
            support = Some((top, block.center));
        }
    }
    if movement == Vec3::ZERO {
        None
    } else {
        support
    }
}

fn step_clear(
    position: Vec3,
    target_feet_y: f32,
    support_center: Vec3,
    movement: Vec3,
    blocks: &Query<&PlacedBlock>,
) -> bool {
    let axis = if movement.x.abs() >= movement.z.abs() {
        0
    } else {
        2
    };
    let direction = if axis == 0 {
        movement.x.signum()
    } else {
        movement.z.signum()
    };

    for block in blocks.iter() {
        let half_extents = world_half_extents(block);
        let block_bottom = block.center.y - half_extents.y;
        let block_top = block.center.y + half_extents.y;
        if block_bottom >= target_feet_y + HUMAN_HEIGHT
            || block_top <= target_feet_y + CONTACT_EPSILON
            || !overlaps_xz(position, block.center, half_extents * 2.0)
        {
            continue;
        }

        let center_offset = if axis == 0 {
            (block.center.x - support_center.x) * direction
        } else {
            (block.center.z - support_center.z) * direction
        };
        if center_offset < MIN_STEP_OFFSET {
            return false;
        }
    }
    true
}

fn highest_support(
    position: Vec3,
    current_feet_y: f32,
    next_feet_y: f32,
    blocks: &Query<&PlacedBlock>,
) -> f32 {
    let mut support: f32 = 0.0;
    for block in blocks.iter() {
        let half_extents = world_half_extents(block);
        let top = block.center.y + half_extents.y;
        if overlaps_xz(position, block.center, half_extents * 2.0)
            && top <= current_feet_y + 0.001
            && top >= next_feet_y - 0.001
            && top > support
        {
            support = top;
        }
    }
    support
}

fn intersects_solid(
    position: Vec3,
    feet_y: f32,
    height: f32,
    blocks: &Query<&PlacedBlock>,
) -> bool {
    for block in blocks.iter() {
        let half_extents = world_half_extents(block);
        let block_bottom = block.center.y - half_extents.y;
        let block_top = block.center.y + half_extents.y;
        if overlaps_xz(position, block.center, half_extents * 2.0)
            && block_bottom < feet_y + height
            && block_top > feet_y + 0.001
        {
            return true;
        }
    }
    false
}

fn world_half_extents(block: &PlacedBlock) -> Vec3 {
    Mat3::from_quat(block.rotation).abs() * (block.size * 0.5)
}

fn intersects_swept(
    start: Vec3,
    end: Vec3,
    feet_y: f32,
    height: f32,
    blocks: &Query<&PlacedBlock>,
) -> bool {
    let distance = start.distance(end);
    let sample_count = ((distance / 0.02).ceil() as usize).max(1);
    for sample in 1..=sample_count {
        let fraction = sample as f32 / sample_count as f32;
        let position = start.lerp(end, fraction);
        if intersects_solid(position, feet_y, height, blocks) {
            return true;
        }
    }
    false
}

fn overlaps_xz(position: Vec3, center: Vec3, size: Vec3) -> bool {
    position.x + HUMAN_HALF_WIDTH > center.x - size.x * 0.5
        && position.x - HUMAN_HALF_WIDTH < center.x + size.x * 0.5
        && position.z + HUMAN_HALF_WIDTH > center.z - size.z * 0.5
        && position.z - HUMAN_HALF_WIDTH < center.z + size.z * 0.5
}
