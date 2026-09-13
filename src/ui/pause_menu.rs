use bevy::prelude::*;
use bevy::text::FontSize;

use crate::save;
use crate::types::{
    ControlsBackButton, ControlsButton, ControlsMenuRoot, CurrentWorld, GameMode, GameState,
    ModeConfirmationText, PauseMenuRoot, PauseSaveAndQuitButton, PauseSaveButton,
    PlacedBlock, ResumeButton, SwitchModeButton,
};
use crate::ui::helpers::*;

/// Spawns the Pause Menu UI screen when the game is paused.
/// This includes buttons to resume, check controls, switch camera modes, save, and quit.
pub fn setup_pause_menu(mut commands: Commands) {
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)), // Semi-transparent dark overlay
            PauseMenuRoot,
        ))
        .with_children(|parent| {
            // "PAUSED" Title Text
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: FontSize::Px(32.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Resume Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(45.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
                    BorderColor::all(Color::WHITE),
                    ResumeButton,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Resume"),
                        TextFont {
                            font_size: FontSize::Px(16.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Controls Menu Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(45.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
                    BorderColor::all(Color::WHITE),
                    ControlsButton,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Controls"),
                        TextFont {
                            font_size: FontSize::Px(16.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Switch Mode Button (uses helper function spawned externally)
            spawn_pause_action_button(parent, "Switch Mode", SwitchModeButton);

            // Mode Confirmation Text (displays what mode the user just switched to)
            parent.spawn((
                Text::new("Press Switch Mode to change mode"),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.95, 1.0)),
                ModeConfirmationText,
            ));

            // Save Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(45.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
                    BorderColor::all(Color::WHITE),
                    PauseSaveButton,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Save"),
                        TextFont {
                            font_size: FontSize::Px(16.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Save & Quit Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(45.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
                    BorderColor::all(Color::WHITE),
                    PauseSaveAndQuitButton,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Save & Quit"),
                        TextFont {
                            font_size: FontSize::Px(16.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

/// Spawns the Controls Menu UI, displaying the keybindings and instructions for different game modes.
pub fn setup_controls_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)), // Darker overlay for readability
            ControlsMenuRoot,
        ))
        .with_children(|parent| {
            // "CONTROLS" Title Text
            parent.spawn((
                Text::new("CONTROLS"),
                TextFont {
                    font_size: FontSize::Px(32.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Controls list container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(24.0),
                    align_items: AlignItems::Start,
                    ..default()
                })
                .with_children(|columns| {
                    // Build/Free Mode Controls
                    spawn_controls_column(
                        columns,
                        "Free Mode",
                        "WASD: Move\nSpace / Shift: Elevation\nHold R + WASD: Rotate view\nArrow keys: Rotate block\nB: Edit a block\nP: Delete mode\nBlock Types: Shape and size\nColour: Choose colour",
                    );
                    // Walk/Human Mode Controls
                    spawn_controls_column(
                        columns,
                        "Human Mode",
                        "WASD: Walk\nMouse: Look around\nSpace: Jump\nSwitch Mode: Return to Free Mode\nF1: Return to start",
                    );
                });

            // Back button to return to the Pause Menu
            spawn_pause_action_button(parent, "Back", ControlsBackButton);
        });
}

/// Cleans up the Pause Menu nodes when transitioning away from the paused state.
pub fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Cleans up the Controls Menu nodes when navigating back to the pause menu.
pub fn cleanup_controls_menu(mut commands: Commands, query: Query<Entity, With<ControlsMenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Processes clicks inside the Pause Menu and Controls Menu.
/// Handles resuming the game, opening the controls screen, switching modes, saving, and quitting.
pub fn pause_menu_interaction_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    current_world: Res<CurrentWorld>,
    mut mode: ResMut<GameMode>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    blocks_query: Query<(&PlacedBlock, &MeshMaterial3d<StandardMaterial>), With<PlacedBlock>>,
    materials: Res<Assets<StandardMaterial>>,
    pause_menu_query: Query<Entity, With<PauseMenuRoot>>,
    controls_menu_query: Query<Entity, With<ControlsMenuRoot>>,
    controls_query: Query<&Interaction, (Changed<Interaction>, With<ControlsButton>)>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<ControlsBackButton>)>,
    resume_query: Query<&Interaction, (Changed<Interaction>, With<ResumeButton>)>,
    save_query: Query<&Interaction, (Changed<Interaction>, With<PauseSaveButton>)>,
    save_quit_query: Query<&Interaction, (Changed<Interaction>, With<PauseSaveAndQuitButton>)>,
    switch_mode_query: Query<&Interaction, (Changed<Interaction>, With<SwitchModeButton>)>,
    mut mode_confirmation_query: Query<&mut Text, With<ModeConfirmationText>>,
) {
    // Handle Resume Button
    for interaction in resume_query.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Playing);
        }
    }

    // Handle Controls Button (Switch to controls menu)
    for interaction in controls_query.iter() {
        if *interaction == Interaction::Pressed {
            for entity in pause_menu_query.iter() {
                commands.entity(entity).despawn();
            }
            setup_controls_menu(commands);
            return;
        }
    }

    // Handle Back Button (Return to pause menu from controls menu)
    for interaction in back_query.iter() {
        if *interaction == Interaction::Pressed {
            for entity in controls_menu_query.iter() {
                commands.entity(entity).despawn();
            }
            setup_pause_menu(commands);
            return;
        }
    }

    // Handle Switch Mode Button (Toggle between Free Mode and Human Mode)
    for interaction in switch_mode_query.iter() {
        if *interaction == Interaction::Pressed {
            *mode = match *mode {
                GameMode::Free => {
                    // Level out the camera when switching to Human Mode
                    if let Ok(mut camera) = camera_query.single_mut() {
                        let forward = camera.forward();
                        camera.look_to(forward, Vec3::Y);
                    }
                    GameMode::Human
                }
                GameMode::Human => GameMode::Free,
            };

            // Update the UI text to confirm the new mode
            let message = match *mode {
                GameMode::Free => "Switched to Free Mode",
                GameMode::Human => "Switched to Human Mode",
            };
            for mut text in mode_confirmation_query.iter_mut() {
                text.0 = message.to_string();
            }
        }
    }

    // Handle Save Button (Save without exiting)
    for interaction in save_query.iter() {
        if *interaction == Interaction::Pressed {
            save::save_house_to_disk(&current_world.name, &blocks_query, &materials);
        }
    }

    // Handle Save & Quit Button (Save data and return to Main Menu)
    for interaction in save_quit_query.iter() {
        if *interaction == Interaction::Pressed {
            save::save_house_to_disk(&current_world.name, &blocks_query, &materials);
            // Explicitly despawn the pause menu root immediately upon selecting Save & Quit
            for entity in pause_menu_query.iter() {
                commands.entity(entity).despawn();
            }
            next_state.set(GameState::MainMenu);
        }
    }
}