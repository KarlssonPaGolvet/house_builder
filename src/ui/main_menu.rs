use bevy::app::AppExit;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::text::FontSize;

use crate::save;
use crate::types::{
    CurrentWorld, GameState, MainMenuNewHouseButton, MainMenuQuitButton, MainMenuRoot,
    MainMenuSavedHousesButton, NewHouseInputText, NewHouseRoot, PlacedBlock, SavedHousesRoot,
    TextInputBuffer,
};
use crate::ui::components::*;
use crate::ui::helpers::*;

/// Sets up the initial Main Menu screen.
pub fn setup_main_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            MainMenuRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("House Builder"),
                TextFont {
                    font_size: FontSize::Px(36.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            spawn_menu_button(parent, "Saved Houses", MainMenuSavedHousesButton);
            spawn_menu_button(parent, "New House", MainMenuNewHouseButton);
            spawn_menu_button(parent, "Quit", MainMenuQuitButton);
        });
}

/// Sets up the UI screen that lists out all saved houses on disk.
pub fn setup_saved_houses_menu(mut commands: Commands) {
    let saves = save::list_saved_houses();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(30.0)),
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            SavedHousesRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Saved Houses"),
                TextFont {
                    font_size: FontSize::Px(28.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            if saves.is_empty() {
                parent.spawn((
                    Text::new("No saved houses found."),
                    TextFont {
                        font_size: FontSize::Px(16.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
            } else {
                // Generate a row with a Load and Delete button for each save file found
                for save_name in saves {
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(10.0),
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_menu_button(
                                row,
                                &save_name,
                                SaveHouseListItem(save_name.clone()),
                            );
                            spawn_menu_button(
                                row,
                                "Delete",
                                DeleteSaveButton(save_name.to_string()),
                            );
                        });
                }
            }

            spawn_menu_button(parent, "Back", BackButton);
        });
}

/// Sets up the input menu to type in the name for a new house save.
pub fn setup_new_house_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            NewHouseRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Enter New World Name:"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new("_"), // Initial cursor representation
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.4)),
                NewHouseInputText,
            ));

            parent.spawn((
                Text::new("Type name & press Enter to confirm. Esc to go back."),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

/// Cleans up the Main Menu nodes when transitioning away from the state.
pub fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Tears down the saved houses menu upon exiting its state.
pub fn cleanup_saved_houses_menu(
    mut commands: Commands,
    query: Query<Entity, With<SavedHousesRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Tears down the new house menu upon exiting its state.
pub fn cleanup_new_house_menu(mut commands: Commands, query: Query<Entity, With<NewHouseRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Handles interactions for the primary main menu buttons (routing to states or exiting).
pub fn main_menu_interaction_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: MessageWriter<AppExit>,
    saved_query: Query<&Interaction, (Changed<Interaction>, With<MainMenuSavedHousesButton>)>,
    new_query: Query<&Interaction, (Changed<Interaction>, With<MainMenuNewHouseButton>)>,
    quit_query: Query<&Interaction, (Changed<Interaction>, With<MainMenuQuitButton>)>,
) {
    for interaction in saved_query.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::SavedHouses);
        }
    }
    for interaction in new_query.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::NewHouseInput);
        }
    }
    for interaction in quit_query.iter() {
        if *interaction == Interaction::Pressed {
            app_exit.write(AppExit::Success);
        }
    }
}

/// Processes clicks inside the Saved Houses menu (loading, deleting, and returning to Main Menu).
pub fn saved_houses_interaction_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut current_world: ResMut<CurrentWorld>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_blocks: Query<Entity, With<PlacedBlock>>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>,
    item_query: Query<(&Interaction, &SaveHouseListItem), Changed<Interaction>>,
    delete_query: Query<(&Interaction, &DeleteSaveButton), Changed<Interaction>>,
    root_query: Query<Entity, With<SavedHousesRoot>>,
) {
    // Back button
    for interaction in back_query.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::MainMenu);
        }
    }

    // Load save button
    for (interaction, item) in item_query.iter() {
        if *interaction == Interaction::Pressed {
            // Clear existing blocks before loading a save so it doesn't overlap or appear empty/bugged
            for entity in existing_blocks.iter() {
                commands.entity(entity).despawn();
            }

            current_world.name = item.0.clone();
            save::load_house_from_disk(&item.0, &mut commands, &mut meshes, &mut materials);
            next_state.set(GameState::Playing);
        }
    }

    // Delete save button
    for (interaction, del) in delete_query.iter() {
        if *interaction == Interaction::Pressed {
            save::delete_save_file(&del.0);

            // Refresh menu screen after delete
            if let Ok(entity) = root_query.single() {
                commands.entity(entity).despawn();
            }
            setup_saved_houses_menu(commands);
            return;
        }
    }
}

/// Reads native keyboard events to allow typing a name for a new house save.
pub fn new_house_input_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut current_world: ResMut<CurrentWorld>,
    mut text_input: ResMut<TextInputBuffer>,
    mut char_evr: MessageReader<KeyboardInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
    blocks_query: Query<(&PlacedBlock, &MeshMaterial3d<StandardMaterial>), With<PlacedBlock>>,
    existing_blocks: Query<Entity, With<PlacedBlock>>,
    materials: Res<Assets<StandardMaterial>>,
    mut text_query: Query<&mut Text, With<NewHouseInputText>>,
) {
    // Abort creation
    if keyboard.just_pressed(KeyCode::Escape) {
        text_input.text.clear();
        next_state.set(GameState::MainMenu);
        return;
    }

    // Read characters typing
    for ev in char_evr.read() {
        if let Some(text) = &ev.text {
            for c in text.chars() {
                if !c.is_control() {
                    text_input.text.push(c);
                }
            }
        }
    }

    // Handle backspacing
    if keyboard.just_pressed(KeyCode::Backspace) {
        text_input.text.pop();
    }

    // Handle confirmation and save creation
    if keyboard.just_pressed(KeyCode::Enter) && !text_input.text.is_empty() {
        let name = text_input.text.trim().to_string();
        let existing = save::list_saved_houses();
        if !name.is_empty() && !existing.contains(&name) {
            // Clear existing blocks for a fresh new house
            for entity in existing_blocks.iter() {
                commands.entity(entity).despawn();
            }

            current_world.name = name.clone();
            text_input.text.clear();
            save::save_house_to_disk(&name, &blocks_query, &materials);
            next_state.set(GameState::Playing);
        }
    }

    // Sync input buffer to the rendering text node, appending an underscore as a cursor
    if let Ok(mut text) = text_query.single_mut() {
        text.0 = if text_input.text.is_empty() {
            "_".to_string()
        } else {
            format!("{}_", text_input.text)
        };
    }
}