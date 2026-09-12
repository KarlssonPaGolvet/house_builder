use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    SavedHouses,
    NewHouseInput,
    Playing,
    Paused,
}

#[derive(Resource)]
pub struct PlacementSettings {
    pub color: Color,
    pub color_name: String,
    pub size: Vec3,
    pub size_name: &'static str,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub is_deleting: bool,
    pub editing_mode: bool,
    pub editing_entity: Option<Entity>,
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
            is_deleting: false,
            editing_mode: false,
            editing_entity: None,
        }
    }
}

#[derive(Resource, Default)]
pub struct CurrentWorld {
    pub name: String,
}

#[derive(Resource, Default)]
pub struct TextInputBuffer {
    pub text: String,
}

#[derive(Resource)]
pub struct ColorInputBuffer {
    pub text: String,
}

impl Default for ColorInputBuffer {
    fn default() -> Self {
        Self {
            text: "#CCB399".to_string(),
        }
    }
}

#[derive(Resource, Default)]
pub struct ColorPickerState {
    pub is_open: bool,
    pub show_history: bool,
    pub history: Vec<String>,
}

#[derive(Component)]
pub struct PlacedBlock {
    pub center: Vec3,
    pub size: Vec3,
    pub rotation_x: f32,
    pub rotation_y: f32,
}

#[derive(Component)]
pub struct SelectionHighlight;

#[derive(Component)]
pub struct ColourButton;

#[derive(Component)]
pub struct SizeButton(pub Vec3, pub &'static str);

#[derive(Component)]
pub struct StatusText;

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct QuitButton;

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct SavedHousesRoot;

#[derive(Component)]
pub struct NewHouseRoot;

#[derive(Component)]
pub struct MainMenuSavedHousesButton;

#[derive(Component)]
pub struct MainMenuNewHouseButton;

#[derive(Component)]
pub struct MainMenuQuitButton;

#[derive(Component)]
pub struct PauseSaveButton;

#[derive(Component)]
pub struct PauseSaveAndQuitButton;

#[derive(Component)]
pub struct ControlsButton;

#[derive(Component)]
pub struct ControlsText;

#[derive(Component)]
pub struct NewHouseInputText;
