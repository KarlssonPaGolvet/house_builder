// ============================================================================
// SAVE & LOAD SYSTEM
// ============================================================================
// This module handles serializing the ECS (Entity Component System) world state
// into JSON files, and deserializing those files back into Bevy entities.

use crate::types::PlacedBlock;
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use serde::{Deserialize, Serialize};
use std::fs;

// ============================================================================
// DATA STRUCTURES FOR SERIALIZATION
// ============================================================================

/// Represents a single block as it exists on disk.
/// We use this intermediate struct instead of directly serializing `PlacedBlock`
/// because it allows us to handle file-format versioning, default values, and
/// decouple our game logic from our save file format.
#[derive(Serialize, Deserialize, Clone)]
pub struct BlockSaveData {
    // Basic physical properties
    pub center: [f32; 3],
    pub size: [f32; 3],
    pub color: [f32; 4], // RGBA representation

    // Legacy rotation field (used in older versions of the builder)
    pub rotation_angle: f32,

    // Modern rotation fields.
    // The `#[serde(default)]` attribute provides backward compatibility!
    // If a user loads an older save file that doesn't have these fields,
    // Serde will automatically fill them with `None` instead of crashing.
    #[serde(default)]
    pub rotation_x: Option<f32>,
    #[serde(default)]
    pub rotation_y: Option<f32>,
    #[serde(default)]
    pub rotation_quat: Option<[f32; 4]>,
}

/// The root JSON object for a save file.
#[derive(Serialize, Deserialize, Clone)]
pub struct HouseSaveFile {
    pub name: String,
    pub blocks: Vec<BlockSaveData>,
}

// ============================================================================
// FILE SYSTEM HELPERS
// ============================================================================

/// Resolves a world name into a standard OS path (e.g., "saves/MyHouse.json").
/// Safely ensures the "saves" directory exists before attempting to access it.
pub fn get_save_path(world_name: &str) -> std::path::PathBuf {
    let dir = "saves";
    // Ignore the Result here; if it already exists, that's fine.
    let _ = fs::create_dir_all(dir);
    std::path::Path::new(dir).join(format!("{}.json", world_name))
}

/// Scans the "saves" directory and returns a list of all valid world names.
/// Used to populate the "Saved Houses" UI menu.
pub fn list_saved_houses() -> Vec<String> {
    let dir = "saves";
    if let Ok(entries) = fs::read_dir(dir) {
        entries
            .filter_map(|e| e.ok()) // Ignore unreadable files/directories
            .filter_map(|e| {
                let path = e.path();
                // Only include files that end with exactly ".json"
                if path.extension()?.to_str()? == "json" {
                    // Extract just the filename without the .json extension
                    Some(path.file_stem()?.to_str()?.to_string())
                } else {
                    None
                }
            })
            .collect() // Gather into a neat Vec<String>
    } else {
        Vec::new() // If directory doesn't exist or can't be read, return empty list
    }
}

// ============================================================================
// CORE SAVE / LOAD LOGIC
// ============================================================================

/// Reads the active Bevy world and writes it asynchronously to a JSON file.
/// Extraction occurs on the main thread, while serialization and disk I/O are
/// offloaded to Bevy's background task pool to prevent frame hitches.
pub fn save_house_to_disk(
    world_name: &str,
    blocks_query: &Query<(&PlacedBlock, &MeshMaterial3d<StandardMaterial>), With<PlacedBlock>>,
    materials: &Res<Assets<StandardMaterial>>,
) {
    // Prevent accidental creation of an unnamed file (e.g., ".json")
    if world_name.is_empty() {
        return;
    }

    let mut block_data = Vec::new();

    // Iterate over every PlacedBlock currently spawned in the world
    for (block, mat_handle) in blocks_query.iter() {
        // Attempt to extract the true RGBA color from the block's material.
        let color = if let Some(material) = materials.get(&mat_handle.0) {
            let srgba = material.base_color.to_srgba();
            [srgba.red, srgba.green, srgba.blue, srgba.alpha]
        } else {
            // Fallback in case the material handle became invalid or hasn't loaded
            [0.8, 0.7, 0.6, 1.0]
        };

        // Map the ECS component data into our Serializable struct
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

    // Wrap the block array in the top-level save file struct
    let save_file = HouseSaveFile {
        name: world_name.to_string(),
        blocks: block_data,
    };

    let path = get_save_path(world_name);

    // Offload JSON serialization and file writing to a background task
    IoTaskPool::get()
        .spawn(async move {
            if let Ok(json) = serde_json::to_string_pretty(&save_file) {
                if let Err(e) = fs::write(&path, json) {
                    eprintln!("Failed to write save file asynchronously: {}", e);
                }
            }
        })
        .detach(); // Fire-and-forget task
}

/// Reads a JSON file from disk and spawns the blocks into the Bevy ECS.
/// This gets called when transitioning into the `Playing` state.
pub fn load_house_from_disk(
    world_name: &str,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let path = get_save_path(world_name);

    // Attempt to read the file. If it fails (e.g., new house, file doesn't exist),
    // we silently do nothing and load an empty world.
    if let Ok(json) = fs::read_to_string(path) {
        if let Ok(save_file) = serde_json::from_str::<HouseSaveFile>(&json) {
            // File successfully parsed, recreate each block
            for b in save_file.blocks {
                let center = Vec3::new(b.center[0], b.center[1], b.center[2]);
                let size = Vec3::new(b.size[0], b.size[1], b.size[2]);
                let color = Color::srgba(b.color[0], b.color[1], b.color[2], b.color[3]);

                // --- BACKWARD COMPATIBILITY LOGIC ---
                // If the save file is older, it might only have `rotation_angle`.
                // We resolve X/Y rotations first, defaulting to older fallbacks if modern keys are missing.
                let rotation_x = b.rotation_x.unwrap_or(0.0);
                let rotation_y = b.rotation_y.unwrap_or(b.rotation_angle);

                // Determine the true Quat. If we saved a true quaternion, use it.
                // Otherwise, mathematically reconstruct it from the Euler angles.
                let rotation = b
                    .rotation_quat
                    .map(|q| Quat::from_xyzw(q[0], q[1], q[2], q[3]))
                    .unwrap_or_else(|| {
                        Quat::from_rotation_y(rotation_y.to_radians())
                            * Quat::from_rotation_x(rotation_x.to_radians())
                    });

                // Spawn the actual 3D Entity with its Mesh, Material, and game logic (PlacedBlock)
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

/// Completely removes a save file from the disk.
/// Triggered by UI buttons (like a delete button) in the Saved Houses menu.
pub fn delete_save_file(world_name: &str) {
    let path = get_save_path(world_name);
    let _ = fs::remove_file(path);
}