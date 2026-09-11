use bevy::prelude::*;
use crate::types::{CurrentWorld, PlacedBlock, PlacementSettings};

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
        crate::save::load_house_from_disk(&current_world.name, &mut commands, &mut meshes, &mut materials);
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
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        settings.color = Color::srgb(0.8, 0.7, 0.6);
        settings.color_name = "Wood / Beige";
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        settings.color = Color::srgb(0.7, 0.2, 0.2);
        settings.color_name = "Red Brick";
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        settings.color = Color::srgb(0.5, 0.5, 0.5);
        settings.color_name = "Gray Stone";
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        settings.color = Color::srgb(0.2, 0.5, 0.8);
        settings.color_name = "Blue Roof";
    }

    if keyboard.just_pressed(KeyCode::KeyZ) {
        settings.size = Vec3::new(1.0, 1.0, 1.0);
        settings.size_name = "Standard Wall (1x1x1)";
    }
    if keyboard.just_pressed(KeyCode::KeyX) {
        settings.size = Vec3::new(1.0, 2.0, 1.0);
        settings.size_name = "Tall Wall (1x2x1)";
    }
    if keyboard.just_pressed(KeyCode::KeyC) {
        settings.size = Vec3::new(2.0, 1.0, 1.0);
        settings.size_name = "Wide Wall (2x1x1)";
    }

    // Toggle delete mode with 'P' (leaving 'R' completely untouched)
    if keyboard.just_pressed(KeyCode::KeyP) {
        settings.is_deleting = !settings.is_deleting;
    }
}

// Ray-AABB intersection returning (distance, surface_normal)
fn intersect_ray_box(ray_origin: Vec3, ray_dir: Vec3, box_center: Vec3, box_size: Vec3) -> Option<(f32, Vec3)> {
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

    if (hit_point.x - min.x).abs() < eps { normal = -Vec3::X; }
    else if (hit_point.x - max.x).abs() < eps { normal = Vec3::X; }
    else if (hit_point.y - min.y).abs() < eps { normal = -Vec3::Y; }
    else if (hit_point.y - max.y).abs() < eps { normal = Vec3::Y; }
    else if (hit_point.z - min.z).abs() < eps { normal = -Vec3::Z; }
    else if (hit_point.z - max.z).abs() < eps { normal = Vec3::Z; }

    Some((t, normal))
}

pub fn place_wall_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    settings: Res<PlacementSettings>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    button_interactions: Query<&Interaction, With<Button>>,
    placed_blocks_query: Query<(Entity, &PlacedBlock)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    for interaction in button_interactions.iter() {
        if *interaction != Interaction::None {
            return;
        }
    }

    let Ok(window) = window_query.single() else { return; };
    let Ok((camera, camera_transform)) = camera_query.single() else { return; };

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let ray_origin = ray.origin;
            let ray_dir: Vec3 = ray.direction.into();

            if settings.is_deleting {
                let mut closest_block: Option<(f32, Entity)> = None;
                let mut min_t = f32::MAX;

                for (entity, block) in placed_blocks_query.iter() {
                    if let Some((t, _)) = intersect_ray_box(ray_origin, ray_dir, block.center, block.size) {
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
                Ground { hit_point: Vec3, normal: Vec3 },
                Block { _t: f32, normal: Vec3, center: Vec3, size: Vec3 },
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
                if let Some((t, normal)) = intersect_ray_box(ray_origin, ray_dir, block.center, block.size) {
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
                    HitTarget::Block { normal, size: hit_size, .. } => {
                        let hit_center = match target {
                            HitTarget::Block { center, .. } => center,
                            _ => unreachable!(),
                        };
                        hit_center + normal * ((hit_size + settings.size) * 0.5)
                    }
                };

                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(settings.size.x, settings.size.y, settings.size.z))),
                    MeshMaterial3d(materials.add(settings.color)),
                    Transform::from_translation(block_center),
                    PlacedBlock {
                        center: block_center,
                        size: settings.size,
                        rotation_angle: settings.rotation_angle,
                    },
                ));
            }
        }
    }
}