use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Playing,
    Paused,
}

#[derive(Resource)]
pub struct PlacementSettings {
    pub color: Color,
    pub color_name: &'static str,
    pub size: Vec3,
    pub size_name: &'static str,
    pub rotation_angle: f32, // 0.0, 90.0, 180.0, 270.0
    pub is_deleting: bool,
}

impl Default for PlacementSettings {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.8, 0.7, 0.6),
            color_name: "Wood / Beige",
            size: Vec3::new(1.0, 1.0, 1.0),
            size_name: "Standard Wall (1x1x1)",
            rotation_angle: 0.0,
            is_deleting: false,
        }
    }
}

#[derive(Component)]
pub struct PlacedBlock {
    pub center: Vec3,
    pub size: Vec3,
    pub rotation_angle: f32,
}

#[derive(Component)]
pub struct ColorButton(pub Color, pub &'static str);

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