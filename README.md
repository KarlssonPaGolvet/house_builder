# 3D House Builder

A voxel-style 3D architectural building simulator written in **Rust** using the **Bevy Engine** and **`bevy_egui`**. 

Design structures in a free-flying editor mode, customize block materials and orientations, and switch into a first-person mode to explore your creations with physics, collision, and step-climbing.

---

## Features

* **Dual Gameplay Modes:**
  * **Free Flying Mode ("God Mode"):** Fly freely around the scene with noclip controls to place walls, edit blocks, and inspect designs from any angle.
  * **Human Mode ("Walk Mode"):** Walk through your built world with first-person controls, realistic physics, gravity, jumping, block collision, and smooth step-climbing.
* **Precision Grid Building:**
  * Raycast target detection with grid snapping on the floor plane ($100 \times 100$ ground bounds) and adjacent block face attachment.
  * Axis-aligned and rotated bounding box (AABB) raycast intersection algorithms for precise block selection and deletion.
* **Block Editing & Manipulation:**
  * **Rotations:** Snap rotations on world axes using keybinds or numerical tweaking.
  * **Color Customization:** `bevy_egui` integrated color picker supporting HEX/RGB input and historical color palettes.
  * **Position Fine-Tuning:** Precise numerical coordinate inputs to position individual blocks down to decimal units.
  * **Visual Selection Outlines:** Animated pulsing edge highlights when selecting blocks for editing.
* **Asynchronous Save & Load System:**
  * Saves world state into JSON files inside the `saves/` directory.
  * Non-blocking file I/O offloaded to Bevy's background `IoTaskPool` to maintain high frame rates during saves.
  * Built-in backward compatibility for older save file schema formats (handling rotation migration automatically).

---

## Controls

### General & Navigation
| Key | Action |
| :--- | :--- |
| `Esc` | Toggle Pause Menu / Resume |
| `F1` | Reset camera/player back to spawn coordinates |

### Free Camera Mode
| Key | Action |
| :--- | :--- |
| `W` / `A` / `S` / `D` | Move Camera (Forward / Left / Backward / Right) |
| `Space` / `Left Shift` | Move Camera Up / Down |
| `Hold R` + `W` / `A` / `S` / `D` | Pitch and Yaw (Look Up / Left / Down / Right) |
| `Q` / `E` | Roll Camera Left / Right |

### Human Walk Mode
| Input | Action |
| :--- | :--- |
| `Mouse Move` | First-Person Look (Cursor is automatically locked) |
| `W` / `A` / `S` / `D` | Walk |
| `Space` | Jump |

### Building & Editing
| Input / Key | Action |
| :--- | :--- |
| `Left Mouse Click` | Place block / Select target block (or Delete if in Delete Mode) |
| `Arrow Keys` | Rotate selected / active placement block by 90° on X/Y axes |
| `P` | Toggle Demolition / Delete Mode |
| `B` | Toggle Block Selection & Edit Mode |

---

## Tech Stack & Architecture

* **Engine:** [Bevy](https://bevyengine.org/) (Entity Component System architecture)
* **GUI Overlay:** `bevy_egui` (Immediate-mode user interface for color pickers, menu overlays, and coordinate tweaking)
* **Serialization:** `serde` & `serde_json`
* **Language:** Rust (2021 Edition)

### Code Base Overview

* `src/main.rs` — App entry point, state machine setup (`GameState`), and plugin initializations.
* `src/building.rs` — Grid snapping, block raycasting, selection outline animations, and building placement systems.
* `src/mechanics.rs` — First-person human mechanics (gravity calculation, collision checks, step-climbing, and mouse look).
* `src/camera.rs` — Spectator "noclip" camera translation and rotation logic.
* `src/save.rs` — Asynchronous JSON save/load systems and file path management.
* `src/types.rs` — Shared ECS components (`PlacedBlock`), resources (`PlacementSettings`), and state definitions.
* `src/ui.rs` — Modular export definitions for UI components and HUD interactions.

---

## Getting Started

### Prerequisites

Ensure you have the Rust toolchain installed (Cargo, `rustc`).

```bash
# Verify installation
rustc --version
cargo --version
```

### Running the App

1. Clone the repository:
   ```bash
   git clone [https://github.com/KarlssonPaGolvet/house_builder.git](https://github.com/KarlssonPaGolvet/house_builder.git)
   cd house_builder
   ```

2. Run the project in release mode for best performance:
   ```bash
   cargo run --release
   ```

---

## Save File Location

Saves are automatically stored as JSON files inside the root `./saves/` directory (e.g., `./saves/MyHouse.json`). You can share save files simply by copying the `.json` files into another installation's `saves/` folder.