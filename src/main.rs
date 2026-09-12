mod building;
mod camera;
mod save;
mod types;
mod ui;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

use building::{
    cleanup_blocks, keyboard_shortcut_system, load_current_world_system, place_wall_system,
    setup_scene,
};

use camera::camera_movement_system;

use types::{CurrentWorld, GameState, PlacedBlock, PlacementSettings};

use ui::{
    cleanup_game_hud, cleanup_main_menu, cleanup_new_house_menu, cleanup_pause_menu,
    cleanup_saved_houses_menu, color_picker_system, main_menu_interaction_system,
    new_house_input_system, pause_menu_interaction_system, saved_houses_interaction_system,
    setup_main_menu, setup_new_house_menu, setup_pause_menu, setup_saved_houses_menu, setup_ui,
    ui_interaction_system, update_status_text_system,
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
        .add_plugins(EguiPlugin::default())
        .init_state::<GameState>()
        .init_resource::<PlacementSettings>()
        .init_resource::<types::TextInputBuffer>()
        .init_resource::<types::ColorInputBuffer>()
        .init_resource::<types::ColorPickerState>()
        .init_resource::<types::CurrentWorld>()
        .add_systems(Startup, setup_scene)
        .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
        .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
        .add_systems(OnEnter(GameState::SavedHouses), setup_saved_houses_menu)
        .add_systems(OnExit(GameState::SavedHouses), cleanup_saved_houses_menu)
        .add_systems(OnEnter(GameState::NewHouseInput), setup_new_house_menu)
        .add_systems(OnExit(GameState::NewHouseInput), cleanup_new_house_menu)
        .add_systems(
            OnEnter(GameState::Playing),
            (setup_ui, load_current_world_system),
        )
        .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
        .add_systems(
            OnExit(GameState::Paused),
            (
                save_current_world_system,
                cleanup_blocks,
                cleanup_game_hud,
                cleanup_pause_menu,
            ),
        )
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
            EguiPrimaryContextPass,
            color_picker_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            pause_menu_interaction_system.run_if(in_state(GameState::Paused)),
        )
        .add_systems(Update, toggle_pause_system)
        .run();
}

fn save_current_world_system(
    current_world: Res<CurrentWorld>,
    blocks_query: Query<(&PlacedBlock, &MeshMaterial3d<StandardMaterial>), With<PlacedBlock>>,
    materials: Res<Assets<StandardMaterial>>,
) {
    if !current_world.name.is_empty() {
        crate::save::save_house_to_disk(&current_world.name, &blocks_query, &materials);
    }
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
            GameState::MainMenu | GameState::SavedHouses | GameState::NewHouseInput => {}
        }
    }
}
