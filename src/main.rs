mod building;
mod camera;
mod types;
mod ui;

use bevy::prelude::*;

use building::{
    keyboard_shortcut_system,
    place_wall_system,
    setup_scene,
};

use camera::camera_movement_system;

use types::{
    GameState,
    PlacementSettings,
};

use ui::{
    cleanup_pause_menu,
    pause_menu_interaction_system,
    setup_pause_menu,
    setup_ui,
    ui_interaction_system,
    update_status_text_system,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "3D House Builder".into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .init_resource::<PlacementSettings>()
        .add_systems(Startup, (setup_scene, setup_ui))
        .add_systems(
            OnEnter(GameState::Paused),
            setup_pause_menu,
        )
        .add_systems(
            OnExit(GameState::Paused),
            cleanup_pause_menu,
        )
        .add_systems(
            Update,
            (
                camera_movement_system,
                ui_interaction_system,
                keyboard_shortcut_system,
                update_status_text_system,
                place_wall_system,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            pause_menu_interaction_system
                .run_if(in_state(GameState::Paused)),
        )
        .add_systems(Update, toggle_pause_system)
        .run();
}

fn toggle_pause_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::Playing => {
                next_state.set(GameState::Paused);
            }
            GameState::Paused => {
                next_state.set(GameState::Playing);
            }
        }
    }
}
