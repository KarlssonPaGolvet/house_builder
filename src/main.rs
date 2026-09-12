// ============================================================================
// MODULE DECLARATIONS
// ============================================================================
// Organizing the codebase into distinct modules for maintainability.
mod building; // Core logic for grid snapping, placing/deleting blocks, and scene setup
mod camera; // Camera controls (e.g., flying, orbiting, or zooming)
mod mechanics; // Player mechanics, physics, cursor state, and movement
mod save; // Serialization/Deserialization of the world state to JSON on disk
mod types; // Shared ECS components, resources, and state enums used across modules
mod ui; // User interface setup, cleanup, and interaction handling (both Bevy UI and Egui)

// ============================================================================
// IMPORTS
// ============================================================================
use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass}; // Immediate mode GUI for complex overlays (like color pickers)

use building::{
    animate_selection_system, cleanup_blocks, keyboard_shortcut_system, load_current_world_system,
    place_wall_system, setup_scene,
};

use camera::camera_movement_system;
use mechanics::{
    cursor_mode_system, human_mouse_look_system, human_movement_system, return_to_start_system,
};

use types::{CurrentWorld, GameMode, GameState, HumanPhysics, PlacedBlock, PlacementSettings};

use ui::{
    cleanup_controls_menu, cleanup_game_hud, cleanup_main_menu, cleanup_new_house_menu,
    cleanup_pause_menu, cleanup_saved_houses_menu, color_picker_system,
    main_menu_interaction_system, new_house_input_system, pause_menu_interaction_system,
    saved_houses_interaction_system, setup_main_menu, setup_new_house_menu, setup_pause_menu,
    setup_saved_houses_menu, setup_ui, ui_interaction_system,
    update_player_debug_coordinates_system, update_status_text_system,
};

// ============================================================================
// MAIN APPLICATION ENTRY POINT
// ============================================================================
fn main() {
    App::new()
        // --------------------------------------------------------------------
        // PLUGINS & WINDOW SETUP
        // --------------------------------------------------------------------
        // DefaultPlugins include core Bevy engine features (rendering, input, assets, etc.)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "3D House Builder".into(),
                // Start the application in borderless fullscreen on the primary monitor
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        // Inject Egui to allow rendering immediate-mode UI overlays alongside Bevy's native UI
        .add_plugins(EguiPlugin::default())
        // --------------------------------------------------------------------
        // STATES & RESOURCES INITIALIZATION
        // --------------------------------------------------------------------
        // init_state defines the core state machine (MainMenu, Playing, Paused, etc.)
        .init_state::<GameState>()
        // Resources are singleton data globally accessible by any system.
        .init_resource::<PlacementSettings>() // Current block size, rotation, and color
        .init_resource::<types::TextInputBuffer>() // Buffer for typing in new save names
        .init_resource::<types::ColorInputBuffer>() // Buffer for HEX/RGB color inputs
        .init_resource::<types::ColorPickerState>() // Visibility/state of the Egui color picker
        .init_resource::<types::BlockMenuState>() // Visibility/state of the block selection menu
        .init_resource::<types::EditPositionState>() // UI state for manually tweaking block coordinates
        .init_resource::<types::CurrentWorld>() // Tracks the name of the currently loaded save file
        .init_resource::<GameMode>() // e.g., Build Mode vs Play/Explore Mode
        .init_resource::<HumanPhysics>() // Player velocity, gravity, and collision data
        // --------------------------------------------------------------------
        // ONE-TIME STARTUP SYSTEMS
        // --------------------------------------------------------------------
        // Runs exactly once when the engine starts (spawns lights, ground plane, default camera)
        .add_systems(Startup, setup_scene)
        // --------------------------------------------------------------------
        // STATE TRANSITIONS (ON ENTER / ON EXIT)
        // --------------------------------------------------------------------
        // Main Menu Transitions
        .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
        .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
        // Saved Houses Menu Transitions
        .add_systems(OnEnter(GameState::SavedHouses), setup_saved_houses_menu)
        .add_systems(OnExit(GameState::SavedHouses), cleanup_saved_houses_menu)
        // New House Input Menu Transitions
        .add_systems(OnEnter(GameState::NewHouseInput), setup_new_house_menu)
        .add_systems(OnExit(GameState::NewHouseInput), cleanup_new_house_menu)
        // Playing State Transitions
        // When entering gameplay, spawn the HUD and load the JSON data for CurrentWorld
        .add_systems(
            OnEnter(GameState::Playing),
            (setup_ui, load_current_world_system),
        )
        // Pause Menu Transitions
        .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
        // NOTE: Saving and cleanup happens here on EXITING the pause menu.
        // This ensures that when the user clicks "Quit & Save" from the pause menu,
        // the world data is serialized and flushed to disk right before returning to the main menu.
        .add_systems(
            OnExit(GameState::Paused),
            (
                save_current_world_system,
                cleanup_blocks,
                cleanup_game_hud,
                cleanup_pause_menu,
                cleanup_controls_menu,
            ),
        )
        // --------------------------------------------------------------------
        // UPDATE SCHEDULE (PER-FRAME GAME LOOP SYSTEMS)
        // --------------------------------------------------------------------
        // These systems run every frame, but ONLY if the application is in the specified state.
        // Menu Interactions
        .add_systems(
            Update,
            main_menu_interaction_system.run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(
            Update,
            saved_houses_interaction_system.run_if(in_state(GameState::SavedHouses)),
        )
        .add_systems(
            Update,
            new_house_input_system.run_if(in_state(GameState::NewHouseInput)),
        )
        // Core Gameplay Loop
        .add_systems(
            Update,
            (
                camera_movement_system,
                human_mouse_look_system,
                human_movement_system,
                return_to_start_system,
                ui_interaction_system,
                keyboard_shortcut_system,
                update_status_text_system,
                update_player_debug_coordinates_system,
                place_wall_system,
                animate_selection_system,
            )
                .run_if(in_state(GameState::Playing)),
        )
        // Egui Context rendering (Separate from Bevy's standard Update schedule to sync with Egui's pass)
        .add_systems(
            EguiPrimaryContextPass,
            color_picker_system.run_if(in_state(GameState::Playing)),
        )
        // Pause Menu Interaction
        .add_systems(
            Update,
            pause_menu_interaction_system.run_if(in_state(GameState::Paused)),
        )
        // Global Systems (Run every frame regardless of state)
        .add_systems(Update, toggle_pause_system) // Listens for ESC key to switch states
        .add_systems(Update, cursor_mode_system) // Manages cursor lock/unlock based on mode
        // Start the engine
        .run();
}

// ============================================================================
// SYSTEM DEFINITIONS
// ============================================================================

/// Scrapes the current ECS world for all PlacedBlock entities and delegates
/// them to the `save` module to be serialized into a JSON file.
/// This is triggered primarily when exiting the Paused state.
fn save_current_world_system(
    current_world: Res<CurrentWorld>,
    blocks_query: Query<(&PlacedBlock, &MeshMaterial3d<StandardMaterial>), With<PlacedBlock>>,
    materials: Res<Assets<StandardMaterial>>,
) {
    // Only attempt to save if a world is actually loaded/named
    if !current_world.name.is_empty() {
        crate::save::save_house_to_disk(&current_world.name, &blocks_query, &materials);
    }
}

/// Global system that listens for the Escape key to toggle the active state
/// between Playing and Paused.
fn toggle_pause_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::Playing => {
                // Suspend gameplay and open pause menu
                next_state.set(GameState::Paused);
            }
            GameState::Paused => {
                // Resume gameplay
                next_state.set(GameState::Playing);
            }
            // Ignore the ESC key if the user is in any of the pre-game menus
            GameState::MainMenu | GameState::SavedHouses | GameState::NewHouseInput => {}
        }
    }
}
