use crate::types::PlacedBlock;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockSaveData {
    pub center: [f32; 3],
    pub size: [f32; 3],
    pub color: [f32; 4],
    pub rotation_angle: f32,
    #[serde(default)]
    pub rotation_x: Option<f32>,
    #[serde(default)]
    pub rotation_y: Option<f32>,
    #[serde(default)]
    pub rotation_quat: Option<[f32; 4]>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HouseSaveFile {
    pub name: String,
    pub blocks: Vec<BlockSaveData>,
}

pub fn get_save_path(world_name: &str) -> std::path::PathBuf {
    let dir = "saves";
    let _ = fs::create_dir_all(dir);
    std::path::Path::new(dir).join(format!("{}.json", world_name))
}

pub fn list_saved_houses() -> Vec<String> {
    let dir = "saves";
    if let Ok(entries) = fs::read_dir(dir) {
        entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let path = e.path();
                if path.extension()?.to_str()? == "json" {
                    Some(path.file_stem()?.to_str()?.to_string())
                } else {
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}

pub fn save_house_to_disk(
    world_name: &str,
    blocks_query: &Query<(&PlacedBlock, &MeshMaterial3d<StandardMaterial>), With<PlacedBlock>>,
    materials: &Res<Assets<StandardMaterial>>,
) {
    if world_name.is_empty() {
        return;
    }

    let mut block_data = Vec::new();
    for (block, mat_handle) in blocks_query.iter() {
        let color = if let Some(material) = materials.get(&mat_handle.0) {
            let srgba = material.base_color.to_srgba();
            [srgba.red, srgba.green, srgba.blue, srgba.alpha]
        } else {
            [0.8, 0.7, 0.6, 1.0] // Fallback color if material handle isn't found
        };

        block_data.push(BlockSaveData {
            center: [block.center.x, block.center.y, block.center.z],
            size: [block.size.x, block.size.y, block.size.z],
            color,
            rotation_angle: block.rotation_y,
            rotation_x: Some(block.rotation_x),
            rotation_y: Some(block.rotation_y),
            rotation_quat: Some([
                block.rotation.x,
                block.rotation.y,
                block.rotation.z,
                block.rotation.w,
            ]),
        });
    }

    let save_file = HouseSaveFile {
        name: world_name.to_string(),
        blocks: block_data,
    };

    if let Ok(json) = serde_json::to_string_pretty(&save_file) {
        let path = get_save_path(world_name);
        let _ = fs::write(path, json);
    }
}

pub fn load_house_from_disk(
    world_name: &str,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let path = get_save_path(world_name);
    if let Ok(json) = fs::read_to_string(path) {
        if let Ok(save_file) = serde_json::from_str::<HouseSaveFile>(&json) {
            for b in save_file.blocks {
                let center = Vec3::new(b.center[0], b.center[1], b.center[2]);
                let size = Vec3::new(b.size[0], b.size[1], b.size[2]);
                let color = Color::srgba(b.color[0], b.color[1], b.color[2], b.color[3]);
                let rotation_x = b.rotation_x.unwrap_or(0.0);
                let rotation_y = b.rotation_y.unwrap_or(b.rotation_angle);
                let rotation = b
                    .rotation_quat
                    .map(|q| Quat::from_xyzw(q[0], q[1], q[2], q[3]))
                    .unwrap_or_else(|| {
                        Quat::from_rotation_y(rotation_y.to_radians())
                            * Quat::from_rotation_x(rotation_x.to_radians())
                    });

                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
                    MeshMaterial3d(materials.add(color)),
                    Transform::from_translation(center).with_rotation(rotation),
                    PlacedBlock {
                        center,
                        size,
                        rotation_x,
                        rotation_y,
                        rotation,
                    },
                ));
            }
        }
    }
}

pub fn delete_save_file(world_name: &str) {
    let path = get_save_path(world_name);
    let _ = fs::remove_file(path);
}
