use bevy::prelude::*;

// ============================================================================
// CORE STATES & MODES
// ============================================================================

/// Dictates the current high-level state of the application.
/// This enum powers the state machine defined in `main.rs`, determining which
/// systems run and which UI menus are rendered on the screen.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu, // The initial landing screen of the game
    SavedHouses,   // Menu displaying a list of previously saved worlds/houses
    NewHouseInput, // Prompt where the user types the name for a new world
    Playing,       // Active 3D gameplay/building environment
    Paused,        // The pause overlay (accessible only from `Playing`)
}

/// Determines the current movement and interaction mechanics of the player.
#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Free,  // "God mode" / Noclip: Free flying camera, ideal for building high walls
    Human, // "Walk mode": Constrained by gravity, height, and collision
}

impl Default for GameMode {
    fn default() -> Self {
        Self::Free
    }
}

// ============================================================================
// GLOBAL RESOURCES (SINGLETON DATA)
// ============================================================================
// Resources hold global state that isn't tied to a specific entity in the world.

/// Tracks physics state specifically for the `Human` GameMode.
/// Currently used to apply gravity and handle jumping arcs.
#[derive(Resource, Default)]
pub struct HumanPhysics {
    pub vertical_velocity: f32, // Current speed of the player on the Y-axis
}

/// The "Brush" tool of the game. Holds all the configurations for the next block
/// the player is going to place, as well as tracking active interaction modes.
#[derive(Resource)]
pub struct PlacementSettings {
    pub color: Color,            // Bevy Color object for rendering
    pub color_name: String,      // Hex/RGB string for UI display and saving
    pub size: Vec3,              // The physical dimensions of the block
    pub size_name: &'static str, // Human-readable name (e.g., "Standard Wall")

    // Rotation tracking: We keep separate X/Y floats for easier UI manipulation
    // and a cached Quaternion for actual 3D math and rendering.
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation: Quat,

    // Tool modes
    pub is_deleting: bool,  // True if the user is in demolition mode
    pub editing_mode: bool, // True if the user is modifying an existing block
    pub editing_entity: Option<Entity>, // Which specific block entity is currently being edited
}

impl Default for PlacementSettings {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.8, 0.7, 0.6),
            color_name: "#CCB399".to_string(),
            size: Vec3::new(1.0, 1.0, 1.0),
            size_name: "Standard Wall (1x1x1)",
            rotation_x: 0.0,
            rotation_y: 0.0,
            rotation: Quat::IDENTITY,
            is_deleting: false,
            editing_mode: false,
            editing_entity: None,
        }
    }
}

/// Tracks the active save file being played. Used by the serialization system
/// in `save_current_world_system` to know where to write the JSON data.
#[derive(Resource, Default)]
pub struct CurrentWorld {
    pub name: String,
}

// ----------------------------------------------------------------------------
// UI Input Buffers & Egui States
// ----------------------------------------------------------------------------

/// Temporarily holds the text typed by the user when creating a new save file.
#[derive(Resource, Default)]
pub struct TextInputBuffer {
    pub text: String,
}

/// Temporarily holds text typed into the custom Hex/RGB color input field.
#[derive(Resource)]
pub struct ColorInputBuffer {
    pub text: String,
}

impl Default for ColorInputBuffer {
    fn default() -> Self {
        Self {
            text: "#CCB399".to_string(), // Matches default PlacementSettings color
        }
    }
}

/// Manages the state of the Egui-based color picker overlay.
#[derive(Resource, Default)]
pub struct ColorPickerState {
    pub is_open: bool,        // Is the window currently visible?
    pub show_history: bool,   // Toggle for previously used colors panel
    pub history: Vec<String>, // List of recently used Hex codes
}

/// Manages the state of the Egui-based block dimensions menu.
#[derive(Resource, Default)]
pub struct BlockMenuState {
    pub is_open: bool,
}

/// State for the Egui window that allows fine-tuning a block's exact XYZ coordinates.
#[derive(Resource, Default)]
pub struct EditPositionState {
    pub entity: Option<Entity>, // The block being edited
    pub is_open: bool,          // Window visibility
    // Parsed numeric coordinates
    pub x: f32,
    pub y: f32,
    pub z: f32,
    // Raw string buffers for the input fields (allows typing "1.5" before parsing)
    pub x_text: String,
    pub y_text: String,
    pub z_text: String,
    pub warning: Option<String>, // Error message if the user types invalid text
}

// ============================================================================
// GAMEPLAY COMPONENTS
// ============================================================================
// Components are attached to Entities in the ECS (Entity Component System).

/// The fundamental building block of the game.
/// Attached to every wall/floor/object the player builds.
/// This component contains the purely logical data required to recreate the mesh,
/// making it the source of truth for saving and loading the world.
#[derive(Component)]
pub struct PlacedBlock {
    pub center: Vec3,
    pub size: Vec3,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation: Quat,
}

/// Attached temporarily to the holographic "preview" block when placing a block,
/// or to highlight an existing block that is currently being aimed at for editing/deletion.
#[derive(Component)]
pub struct SelectionHighlight;

// ============================================================================
// UI MARKER COMPONENTS
// ============================================================================
// In Bevy, UI is often managed by attaching empty struct "Markers" to UI nodes.
// This allows interaction systems to query specifically for `Query<(&Interaction, &MyButton)>`.
// It also allows cleanup systems to easily find and delete whole UI trees.

// --- In-Game HUD Markers ---
#[derive(Component)]
pub struct ColourButton;
#[derive(Component)]
pub struct BlockMenuButton;
#[derive(Component)]
pub struct EditCoordinatesButton;
#[derive(Component)]
pub struct CoordinateText; // Displays XYZ of looked-at block
#[derive(Component)]
pub struct PlayerDebugCoordinates; // Displays player's XYZ
#[derive(Component)]
pub struct StatusText; // Displays current tool (Build/Delete)
#[derive(Component)]
pub struct SwitchModeButton; // Toggles Free/Human mode
#[derive(Component)]
pub struct ModeConfirmationText; // Visual feedback for mode switch

// --- Main Menu Markers ---
#[derive(Component)]
pub struct MainMenuRoot; // Root node (despawned on exit)
#[derive(Component)]
pub struct MainMenuSavedHousesButton;
#[derive(Component)]
pub struct MainMenuNewHouseButton;
#[derive(Component)]
pub struct MainMenuQuitButton;

// --- Saved Houses Menu Markers ---
#[derive(Component)]
pub struct SavedHousesRoot;

// --- New House Prompt Markers ---
#[derive(Component)]
pub struct NewHouseRoot;
#[derive(Component)]
pub struct NewHouseInputText; // Visual representation of TextInputBuffer

// --- Pause Menu Markers ---
#[derive(Component)]
pub struct PauseMenuRoot; // Root node (despawned on exit)
#[derive(Component)]
pub struct ResumeButton;
#[derive(Component)]
pub struct PauseSaveButton; // Save without quitting
#[derive(Component)]
pub struct PauseSaveAndQuitButton; // Save and return to Main Menu
#[derive(Component)]
pub struct QuitButton; // Quit without saving
#[derive(Component)]
pub struct ControlsButton; // Opens controls sub-menu

// --- Controls Overlay Markers ---
#[derive(Component)]
pub struct ControlsMenuRoot;
#[derive(Component)]
pub struct ControlsBackButton;
