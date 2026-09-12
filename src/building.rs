use crate::types::{
    BlockMenuState, ColorPickerState, CurrentWorld, EditPositionState, GameMode, PlacedBlock,
    PlacementSettings, SelectionHighlight,
};
use bevy::prelude::*;
use bevy_egui::EguiContexts;

// Building-related gameplay systems live here so the scene setup, block
// placement, editing, deletion, and selection feedback share the same spatial
// calculations.
const GRID_SIZE: f32 = 1.0;
// The visual ground plane spans -100..100 on both horizontal axes. Placement
// uses the same limit so the player cannot create blocks on the invisible
// mathematical continuation of the ground plane.
const FLOOR_HALF_SIZE: f32 = 100.0;

/// Creates the persistent environment shared by every saved world: ground,
/// lighting, and the camera. Placed blocks are created separately when a world
/// is loaded or when the player clicks to build.
pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Keep the mesh size and FLOOR_HALF_SIZE in sync. The raycast can still
    // intersect an infinite y=0 plane, so the placement system checks bounds.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(200.0, 200.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.6, 0.2))),
        Transform::default(),
    ));

    // A single point light keeps the block colours readable without requiring
    // each block to carry its own lighting setup.
    commands.spawn((
        PointLight {
            intensity: 2_500_000.0,
            range: 100.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 12.0, 8.0),
    ));

    // This is also the F1 reset transform and the initial Free Mode viewpoint.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-8.0, 10.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn load_current_world_system(
    current_world: Res<CurrentWorld>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_blocks: Query<Entity, With<PlacedBlock>>,
) {
    // OnEnter(Playing) can happen after another world was active. Remove the
    // old block entities before loading the selected save to avoid duplicates.
    for entity in existing_blocks.iter() {
        commands.entity(entity).despawn();
    }

    if !current_world.name.is_empty() {
        crate::save::load_house_from_disk(
            &current_world.name,
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }
}

pub fn cleanup_blocks(mut commands: Commands, query: Query<Entity, With<PlacedBlock>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Handles keyboard-only build tools. Rotation is stored as a quaternion for
/// rendering and collision, while rotation_x/rotation_y remain as readable
/// angle values for the HUD and compatibility with older save data.
pub fn keyboard_shortcut_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<PlacementSettings>,
    color_picker: Res<ColorPickerState>,
    mode: Res<GameMode>,
    mut blocks_query: Query<(&mut Transform, &mut PlacedBlock)>,
    mut commands: Commands,
    highlight_query: Query<Entity, With<SelectionHighlight>>,
) {
    // Human Mode is intentionally read-only: it must not respond to build,
    // delete, edit, or rotation shortcuts. An open colour picker also owns
    // keyboard input, so arrow keys must not rotate a block underneath it.
    if color_picker.is_open || *mode == GameMode::Human {
        return;
    }

    // P toggles deletion rather than placing a new block.
    if keyboard.just_pressed(KeyCode::KeyP) {
        settings.is_deleting = !settings.is_deleting;
    }

    // B toggles edit-selection mode. The next world click selects a block and
    // creates the pulsing outline; it does not create a new block.
    if keyboard.just_pressed(KeyCode::KeyB) {
        settings.editing_mode = !settings.editing_mode;
        settings.editing_entity = None;
        if !settings.editing_mode {
            for entity in highlight_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }

    let mut rotation_changed = false;
    let mut rotation_delta = Quat::IDENTITY;
    // These deltas deliberately use world axes. The block orientation should
    // be stable in the world even if the camera has been rotated or rolled.
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        rotation_delta = Quat::from_rotation_y(-90.0_f32.to_radians());
        settings.rotation_y = wrap_angle(settings.rotation_y - 90.0);
        rotation_changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        rotation_delta = Quat::from_rotation_y(90.0_f32.to_radians());
        settings.rotation_y = wrap_angle(settings.rotation_y + 90.0);
        rotation_changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        rotation_delta = Quat::from_rotation_x(90.0_f32.to_radians());
        settings.rotation_x = wrap_angle(settings.rotation_x + 90.0);
        rotation_changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        rotation_delta = Quat::from_rotation_x(-90.0_f32.to_radians());
        settings.rotation_x = wrap_angle(settings.rotation_x - 90.0);
        rotation_changed = true;
    }

    if rotation_changed {
        // Pre-multiplication applies the delta in world space.
        settings.rotation = rotation_delta * settings.rotation;
        if let Some(entity) = settings.editing_entity {
            if let Ok((mut transform, mut block)) = blocks_query.get_mut(entity) {
                transform.rotation = rotation_delta * transform.rotation;
                block.rotation = transform.rotation;
            } else {
                settings.editing_entity = None;
            }
        }
    }
}

fn wrap_angle(angle: f32) -> f32 {
    angle.rem_euclid(360.0)
}

/// Returns the axis-aligned half-extents of a rotated block in world space.
/// A rotated box is represented by an AABB for the lightweight raycast and
/// collision code, so each local extent must be projected onto world axes.
fn world_half_extents(block: &PlacedBlock) -> Vec3 {
    world_half_extents_for(block.size, block.rotation)
}

fn world_half_extents_for(size: Vec3, rotation: Quat) -> Vec3 {
    Mat3::from_quat(rotation).abs() * (size * 0.5)
}

/// Intersects a ray with an axis-aligned box.
///
/// The returned distance selects the closest hit and the normal identifies
/// which face was hit, allowing a new block to be attached to that face.
fn intersect_ray_box(
    ray_origin: Vec3,
    ray_dir: Vec3,
    box_center: Vec3,
    box_size: Vec3,
) -> Option<(f32, Vec3)> {
    let min = box_center - box_size * 0.5;
    let max = box_center + box_size * 0.5;

    // Slab intersection: each axis contributes an entering and exiting t
    // value; the ray hits the box only when all three intervals overlap.
    let t1 = (min.x - ray_origin.x) / ray_dir.x;
    let t2 = (max.x - ray_origin.x) / ray_dir.x;
    let t3 = (min.y - ray_origin.y) / ray_dir.y;
    let t4 = (max.y - ray_origin.y) / ray_dir.y;
    let t5 = (min.z - ray_origin.z) / ray_dir.z;
    let t6 = (max.z - ray_origin.z) / ray_dir.z;

    let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
    let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

    if tmax < 0.0 || tmin > tmax {
        return None;
    }

    let t = if tmin < 0.0 { tmax } else { tmin };
    if t < 0.0 {
        return None;
    }

    // Reconstruct the hit point to determine the face normal. The epsilon
    // avoids missing a face because of floating-point rounding.
    let hit_point = ray_origin + ray_dir * t;
    let mut normal = Vec3::Y;
    let eps = 0.001;

    if (hit_point.x - min.x).abs() < eps {
        normal = -Vec3::X;
    } else if (hit_point.x - max.x).abs() < eps {
        normal = Vec3::X;
    } else if (hit_point.y - min.y).abs() < eps {
        normal = -Vec3::Y;
    } else if (hit_point.y - max.y).abs() < eps {
        normal = Vec3::Y;
    } else if (hit_point.z - min.z).abs() < eps {
        normal = -Vec3::Z;
    } else if (hit_point.z - max.z).abs() < eps {
        normal = Vec3::Z;
    }

    Some((t, normal))
}

pub fn place_wall_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut egui_contexts: EguiContexts,
    mut settings: ResMut<PlacementSettings>,
    mut edit_position: ResMut<EditPositionState>,
    color_picker: Res<ColorPickerState>,
    block_menu: Res<BlockMenuState>,
    mode: Res<GameMode>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    button_interactions: Query<&Interaction, With<Button>>,
    placed_blocks_query: Query<(Entity, &PlacedBlock)>,
    selection_highlights: Query<Entity, With<SelectionHighlight>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // All overlays and Human Mode are modal. Returning before reading the
    // mouse prevents clicks on menus, their surrounding area, or the human
    // view from changing the world.
    if color_picker.is_open
        || block_menu.is_open
        || edit_position.is_open
        || *mode == GameMode::Human
    {
        return;
    }

    if let Ok(context) = egui_contexts.ctx_mut() {
        if context.egui_wants_pointer_input() {
            return;
        }
    }

    // Placement is click-driven; holding the button must not place blocks
    // every frame.
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    for interaction in button_interactions.iter() {
        if *interaction != Interaction::None {
            return;
        }
    }

    let Ok(window) = window_query.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let ray_origin = ray.origin;
            let ray_dir: Vec3 = ray.direction.into();

            if settings.editing_mode {
                // Find the closest visible block under the cursor. The raycast
                // uses world-space extents so rotated/wide blocks are selectable
                // across their complete visible footprint.
                let mut closest_block: Option<(f32, Entity, f32, f32, Vec3)> = None;
                let mut min_t = f32::MAX;

                for (entity, block) in placed_blocks_query.iter() {
                    if let Some((t, _)) = intersect_ray_box(
                        ray_origin,
                        ray_dir,
                        block.center,
                        world_half_extents(block) * 2.0,
                    ) {
                        if t > 0.0 && t < min_t {
                            min_t = t;
                            closest_block =
                                Some((t, entity, block.rotation_x, block.rotation_y, block.size));
                        }
                    }
                }

                if let Some((_, entity, rotation_x, rotation_y, size)) = closest_block {
                    settings.editing_entity = Some(entity);
                    settings.rotation_x = rotation_x;
                    settings.rotation_y = rotation_y;
                    if let Ok((_, selected_block)) = placed_blocks_query.get(entity) {
                        settings.rotation = selected_block.rotation;
                    }
                    edit_position.entity = Some(entity);
                    for highlight in selection_highlights.iter() {
                        commands.entity(highlight).despawn();
                    }
                    spawn_selection_highlight(
                        &mut commands,
                        entity,
                        size,
                        &mut meshes,
                        &mut materials,
                    );
                } else {
                    settings.editing_entity = None;
                    for highlight in selection_highlights.iter() {
                        commands.entity(highlight).despawn();
                    }
                }
                return;
            }

            if settings.is_deleting {
                // Delete mode uses the same closest-hit rule as edit mode, but
                // removes the entity instead of selecting it.
                let mut closest_block: Option<(f32, Entity)> = None;
                let mut min_t = f32::MAX;

                for (entity, block) in placed_blocks_query.iter() {
                    if let Some((t, _)) = intersect_ray_box(
                        ray_origin,
                        ray_dir,
                        block.center,
                        world_half_extents(block) * 2.0,
                    ) {
                        if t > 0.0 && t < min_t {
                            min_t = t;
                            closest_block = Some((t, entity));
                        }
                    }
                }

                if let Some((_, entity)) = closest_block {
                    commands.entity(entity).despawn();
                }
                return;
            }

            // Normal build mode first finds either the ground or the closest
            // existing block. Existing blocks take precedence when they are
            // closer than the ground intersection.
            enum HitTarget {
                Ground {
                    hit_point: Vec3,
                    normal: Vec3,
                },
                Block {
                    _t: f32,
                    normal: Vec3,
                    center: Vec3,
                    size: Vec3,
                },
            }

            let mut closest_hit: Option<HitTarget> = None;
            let mut min_t = f32::MAX;

            // 1. Check the mathematical ground plane (y = 0).
            if ray_dir.y < 0.0 {
                let t_ground = -ray_origin.y / ray_dir.y;
                if t_ground > 0.0 {
                    min_t = t_ground;
                    closest_hit = Some(HitTarget::Ground {
                        hit_point: ray_origin + ray_dir * t_ground,
                        normal: Vec3::Y,
                    });
                }
            }

            // 2. Check existing blocks using their world-space AABBs.
            for (_, block) in placed_blocks_query.iter() {
                if let Some((t, normal)) = intersect_ray_box(
                    ray_origin,
                    ray_dir,
                    block.center,
                    world_half_extents(block) * 2.0,
                ) {
                    if t > 0.0 && t < min_t {
                        min_t = t;
                        closest_hit = Some(HitTarget::Block {
                            _t: t,
                            normal,
                            center: block.center,
                            size: world_half_extents(block) * 2.0,
                        });
                    }
                }
            }

            // 3. Offset the new block by both boxes' world half-extents, then
            // snap ground placement to the global grid.
            if let Some(target) = closest_hit {
                let block_center = match target {
                    HitTarget::Ground { hit_point, normal } => {
                        // The ray can hit the infinite plane outside the mesh;
                        // reject that case before spawning an entity.
                        if hit_point.x.abs() > FLOOR_HALF_SIZE
                            || hit_point.z.abs() > FLOOR_HALF_SIZE
                        {
                            return;
                        }
                        let new_half_extents =
                            world_half_extents_for(settings.size, settings.rotation);
                        let mut center = hit_point + normal * new_half_extents;
                        center.x = (center.x / GRID_SIZE).round() * GRID_SIZE;
                        center.z = (center.z / GRID_SIZE).round() * GRID_SIZE;
                        center
                    }
                    HitTarget::Block {
                        normal,
                        size: hit_size,
                        ..
                    } => {
                        let hit_center = match target {
                            HitTarget::Block { center, .. } => center,
                            _ => unreachable!(),
                        };
                        let hit_half_extents = hit_size * 0.5;
                        let new_half_extents =
                            world_half_extents_for(settings.size, settings.rotation);
                        hit_center + normal * (hit_half_extents + new_half_extents)
                    }
                };

                // The ECS entity becomes the active coordinate target so the
                // HUD Edit button can immediately move the new block.
                let placed_entity = commands
                    .spawn((
                        Mesh3d(meshes.add(Cuboid::new(
                            settings.size.x,
                            settings.size.y,
                            settings.size.z,
                        ))),
                        MeshMaterial3d(materials.add(settings.color)),
                        Transform::from_translation(block_center).with_rotation(settings.rotation),
                        PlacedBlock {
                            center: block_center,
                            size: settings.size,
                            rotation_x: settings.rotation_x,
                            rotation_y: settings.rotation_y,
                            rotation: settings.rotation,
                        },
                    ))
                    .id();
                edit_position.entity = Some(placed_entity);
            }
        }
    }
}

fn spawn_selection_highlight(
    commands: &mut Commands,
    entity: Entity,
    size: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // The highlight is made from twelve thin cuboids, one for each edge of
    // the selected block. Parenting them to the block makes the outline follow
    // its position and world rotation automatically.
    let thickness = 0.035;
    let x_mesh = meshes.add(Cuboid::new(thickness, size.y + thickness, thickness));
    let y_mesh = meshes.add(Cuboid::new(size.x + thickness, thickness, thickness));
    let z_mesh = meshes.add(Cuboid::new(thickness, thickness, size.z + thickness));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.55, 1.0),
        unlit: true,
        ..default()
    });
    let half = size * 0.5;
    let offset = thickness * 0.5;

    commands.entity(entity).with_children(|parent| {
        // Four vertical edges.
        for x in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                parent.spawn((
                    Mesh3d(x_mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(Vec3::new(
                        x * (half.x + offset),
                        0.0,
                        z * (half.z + offset),
                    )),
                    SelectionHighlight,
                ));
            }
        }
        // Four edges along the local X direction.
        for y in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                parent.spawn((
                    Mesh3d(y_mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(Vec3::new(
                        0.0,
                        y * (half.y + offset),
                        z * (half.z + offset),
                    )),
                    SelectionHighlight,
                ));
            }
        }
        // Four edges along the local Z direction.
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                parent.spawn((
                    Mesh3d(z_mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(Vec3::new(
                        x * (half.x + offset),
                        y * (half.y + offset),
                        0.0,
                    )),
                    SelectionHighlight,
                ));
            }
        }
    });
}

pub fn animate_selection_system(
    time: Res<Time>,
    settings: Res<PlacementSettings>,
    highlights: Query<&MeshMaterial3d<StandardMaterial>, With<SelectionHighlight>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // No selected block means there is no reason to touch highlight materials.
    if !settings.editing_mode || settings.editing_entity.is_none() {
        return;
    }

    // A sine wave gives a soft breathing effect rather than a distracting
    // binary blink.
    let pulse = (time.elapsed_secs() * 3.0).sin() * 0.5 + 0.5;
    let color = Color::srgb(
        0.15 + pulse * 0.55,
        0.35 + pulse * 0.55,
        0.75 + pulse * 0.25,
    );
    for material_handle in highlights.iter() {
        if let Some(mut material) = materials.get_mut(&material_handle.0) {
            material.base_color = color;
        }
    }
}
