use crate::types::{
    ColorPickerState, CurrentWorld, EditPositionState, PlacedBlock, PlacementSettings,
    SelectionHighlight,
};
use bevy::prelude::*;
use bevy_egui::EguiContexts;

const GRID_SIZE: f32 = 1.0;

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(30.0, 30.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.6, 0.2))),
        Transform::default(),
    ));

    // Light
    commands.spawn((
        PointLight {
            intensity: 2_500_000.0,
            range: 100.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 12.0, 8.0),
    ));

    // 3D Camera
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
    // Clear any existing blocks first to prevent duplication on reload
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

pub fn keyboard_shortcut_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<PlacementSettings>,
    color_picker: Res<ColorPickerState>,
    mut blocks_query: Query<(&mut Transform, &mut PlacedBlock)>,
    mut commands: Commands,
    highlight_query: Query<Entity, With<SelectionHighlight>>,
) {
    if color_picker.is_open {
        return;
    }

    // Toggle delete mode with 'P' (leaving 'R' completely untouched)
    if keyboard.just_pressed(KeyCode::KeyP) {
        settings.is_deleting = !settings.is_deleting;
    }

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
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        settings.rotation_y = wrap_angle(settings.rotation_y - 90.0);
        rotation_changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        settings.rotation_y = wrap_angle(settings.rotation_y + 90.0);
        rotation_changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        settings.rotation_x = wrap_angle(settings.rotation_x + 90.0);
        rotation_changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        settings.rotation_x = wrap_angle(settings.rotation_x - 90.0);
        rotation_changed = true;
    }

    if rotation_changed {
        if let Some(entity) = settings.editing_entity {
            if let Ok((mut transform, mut block)) = blocks_query.get_mut(entity) {
                block.rotation_x = settings.rotation_x;
                block.rotation_y = settings.rotation_y;
                transform.rotation = rotation_quat(settings.rotation_x, settings.rotation_y);
            } else {
                settings.editing_entity = None;
            }
        }
    }
}

fn wrap_angle(angle: f32) -> f32 {
    angle.rem_euclid(360.0)
}

fn rotation_quat(rotation_x: f32, rotation_y: f32) -> Quat {
    Quat::from_rotation_y(rotation_y.to_radians()) * Quat::from_rotation_x(rotation_x.to_radians())
}

// Ray-AABB intersection returning (distance, surface_normal)
fn intersect_ray_box(
    ray_origin: Vec3,
    ray_dir: Vec3,
    box_center: Vec3,
    box_size: Vec3,
) -> Option<(f32, Vec3)> {
    let min = box_center - box_size * 0.5;
    let max = box_center + box_size * 0.5;

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
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    button_interactions: Query<&Interaction, With<Button>>,
    placed_blocks_query: Query<(Entity, &PlacedBlock)>,
    selection_highlights: Query<Entity, With<SelectionHighlight>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if color_picker.is_open {
        return;
    }

    if let Ok(context) = egui_contexts.ctx_mut() {
        if context.egui_wants_pointer_input() {
            return;
        }
    }

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
                let mut closest_block: Option<(f32, Entity, f32, f32, Vec3)> = None;
                let mut min_t = f32::MAX;

                for (entity, block) in placed_blocks_query.iter() {
                    if let Some((t, _)) =
                        intersect_ray_box(ray_origin, ray_dir, block.center, block.size)
                    {
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
                let mut closest_block: Option<(f32, Entity)> = None;
                let mut min_t = f32::MAX;

                for (entity, block) in placed_blocks_query.iter() {
                    if let Some((t, _)) =
                        intersect_ray_box(ray_origin, ray_dir, block.center, block.size)
                    {
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

            // 1. Check ground plane intersection (y = 0)
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

            // 2. Check existing placed blocks
            for (_, block) in placed_blocks_query.iter() {
                if let Some((t, normal)) =
                    intersect_ray_box(ray_origin, ray_dir, block.center, block.size)
                {
                    if t > 0.0 && t < min_t {
                        min_t = t;
                        closest_hit = Some(HitTarget::Block {
                            _t: t,
                            normal,
                            center: block.center,
                            size: block.size,
                        });
                    }
                }
            }

            // 3. Compute final block center based on what was hit
            if let Some(target) = closest_hit {
                let block_center = match target {
                    HitTarget::Ground { hit_point, normal } => {
                        let mut center = hit_point + normal * (settings.size * 0.5);
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
                        hit_center + normal * ((hit_size + settings.size) * 0.5)
                    }
                };

                let placed_entity = commands
                    .spawn((
                        Mesh3d(meshes.add(Cuboid::new(
                            settings.size.x,
                            settings.size.y,
                            settings.size.z,
                        ))),
                        MeshMaterial3d(materials.add(settings.color)),
                        Transform::from_translation(block_center)
                            .with_rotation(rotation_quat(settings.rotation_x, settings.rotation_y)),
                        PlacedBlock {
                            center: block_center,
                            size: settings.size,
                            rotation_x: settings.rotation_x,
                            rotation_y: settings.rotation_y,
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
    if !settings.editing_mode || settings.editing_entity.is_none() {
        return;
    }

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
