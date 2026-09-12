use bevy::app::AppExit;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::text::FontSize;
use bevy_egui::{EguiContexts, egui};

use crate::save;
use crate::types::PlacedBlock;
use crate::types::{
    ColorInputBuffer, ColorPickerState, ColourButton, ControlsButton, ControlsText, CurrentWorld,
    GameState, MainMenuNewHouseButton, MainMenuQuitButton, MainMenuRoot, MainMenuSavedHousesButton,
    NewHouseInputText, NewHouseRoot, PauseMenuRoot, PauseSaveAndQuitButton, PauseSaveButton,
    PlacementSettings, ResumeButton, SavedHousesRoot, SizeButton, StatusText, TextInputBuffer,
};

pub fn setup_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(15.0),
                top: Val::Px(15.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.85)),
            GameHudRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("3D House Builder Controls"),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new("Initializing..."),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.4)),
                StatusText,
            ));

            parent.spawn((
                Text::new("Choose Block Color:"),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            spawn_compact_button(parent, "Colour", ColourButton);

            parent.spawn((
                Text::new("Select Size (Z/X/C):"),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_button(
                        row,
                        "1x1 Wall",
                        SizeButton(Vec3::new(1.0, 1.0, 1.0), "Standard Wall (1x1x1)"),
                    );

                    spawn_button(
                        row,
                        "Tall 1x2",
                        SizeButton(Vec3::new(1.0, 2.0, 1.0), "Tall Wall (1x2x1)"),
                    );

                    spawn_button(
                        row,
                        "Wide 2x1",
                        SizeButton(Vec3::new(2.0, 1.0, 1.0), "Wide Wall (2x1x1)"),
                    );
                });
        });
}

fn spawn_button<C: Component>(parent: &mut ChildSpawnerCommands, label: &str, marker: C) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            BorderColor::all(Color::WHITE),
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_compact_button<C: Component>(parent: &mut ChildSpawnerCommands, label: &str, marker: C) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            BorderColor::all(Color::WHITE),
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_menu_button<C: Component>(parent: &mut ChildSpawnerCommands, label: &str, marker: C) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(220.0),
                height: Val::Px(45.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            BorderColor::all(Color::WHITE),
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn ui_interaction_system(
    mut settings: ResMut<PlacementSettings>,
    mut picker: ResMut<ColorPickerState>,
    colour_buttons: Query<&Interaction, (Changed<Interaction>, With<ColourButton>)>,
    size_buttons: Query<(&Interaction, &SizeButton), (Changed<Interaction>, With<Button>)>,
) {
    for interaction in colour_buttons.iter() {
        if *interaction == Interaction::Pressed {
            picker.is_open = true;
        }
    }

    for (interaction, size_btn) in size_buttons.iter() {
        if *interaction == Interaction::Pressed {
            settings.size = size_btn.0;
            settings.size_name = size_btn.1;
        }
    }
}

pub fn color_picker_system(
    mut contexts: EguiContexts,
    mut settings: ResMut<PlacementSettings>,
    mut color_input: ResMut<ColorInputBuffer>,
    mut picker: ResMut<ColorPickerState>,
) {
    if !picker.is_open {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let srgba = settings.color.to_srgba();
    let mut color = egui::Color32::from_rgba_unmultiplied(
        (srgba.red.clamp(0.0, 1.0) * 255.0) as u8,
        (srgba.green.clamp(0.0, 1.0) * 255.0) as u8,
        (srgba.blue.clamp(0.0, 1.0) * 255.0) as u8,
        (srgba.alpha.clamp(0.0, 1.0) * 255.0) as u8,
    );
    let mut hex = color_input.text.clone();

    let mut is_open = picker.is_open;
    egui::Window::new("Block Colour")
        .default_width(220.0)
        .open(&mut is_open)
        .show(ctx, |ui| {
            ui.label("Color wheel");
            if egui::color_picker::color_edit_button_srgba(
                ui,
                &mut color,
                egui::color_picker::Alpha::Opaque,
            )
            .changed()
            {
                let [red, green, blue, _] = color.to_array();
                settings.color = Color::srgba(
                    red as f32 / 255.0,
                    green as f32 / 255.0,
                    blue as f32 / 255.0,
                    1.0,
                );
                settings.color_name = format!("#{red:02X}{green:02X}{blue:02X}");
                color_input.text = settings.color_name.clone();
                add_color_to_history(&mut picker.history, &settings.color_name);
            }

            ui.label("Hexadecimal");
            if ui.text_edit_singleline(&mut hex).changed() {
                color_input.text = hex.clone();
                if let Some(parsed) = parse_hex_color(&hex) {
                    settings.color = parsed;
                    settings.color_name = normalize_hex(&hex);
                    color_input.text = settings.color_name.clone();
                    add_color_to_history(&mut picker.history, &settings.color_name);
                }
            }

            if ui.button("History").clicked() {
                picker.show_history = !picker.show_history;
            }

            if picker.show_history {
                ui.separator();
                ui.label("Recent colours");
                for history_color in picker.history.clone() {
                    ui.horizontal(|ui| {
                        if let Some(parsed) = parse_hex_color(&history_color) {
                            let srgba = parsed.to_srgba();
                            let swatch = egui::Color32::from_rgba_unmultiplied(
                                (srgba.red * 255.0) as u8,
                                (srgba.green * 255.0) as u8,
                                (srgba.blue * 255.0) as u8,
                                (srgba.alpha * 255.0) as u8,
                            );
                            let (rect, _) = ui
                                .allocate_exact_size(egui::Vec2::splat(14.0), egui::Sense::hover());
                            ui.painter().circle_filled(rect.center(), 6.0, swatch);
                        }

                        if ui.button(&history_color).clicked() {
                            if let Some(parsed) = parse_hex_color(&history_color) {
                                settings.color = parsed;
                                settings.color_name = history_color.clone();
                                color_input.text = history_color.clone();
                            }
                        }
                    });
                }
            }
        });

    picker.is_open = is_open;
}

fn add_color_to_history(history: &mut Vec<String>, color: &str) {
    history.retain(|entry| entry != color);
    history.insert(0, color.to_string());
    history.truncate(15);
}

fn parse_hex_color(value: &str) -> Option<Color> {
    let trimmed = value.trim();
    let value = trimmed.strip_prefix('#').unwrap_or(trimmed);
    if value.len() != 6 && value.len() != 8 {
        return None;
    }
    let red = u8::from_str_radix(&value[0..2], 16).ok()?;
    let green = u8::from_str_radix(&value[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&value[4..6], 16).ok()?;
    let alpha = if value.len() == 8 {
        u8::from_str_radix(&value[6..8], 16).ok()?
    } else {
        255
    };
    Some(Color::srgba(
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
        alpha as f32 / 255.0,
    ))
}

fn normalize_hex(value: &str) -> String {
    format!("#{}", value.trim().trim_start_matches('#').to_uppercase())
}

pub fn update_status_text_system(
    settings: Res<PlacementSettings>,
    mut text_query: Query<&mut Text, With<StatusText>>,
) {
    if settings.is_changed() {
        if let Ok(mut text) = text_query.single_mut() {
            let mode_str = if settings.is_deleting {
                "DELETE"
            } else {
                "BUILD"
            };
            text.0 = format!(
                "Mode: {}\nColor: {}\nSize: {}\nRotation: {}°",
                mode_str, settings.color_name, settings.size_name, settings.rotation_angle as i32
            );
        }
    }
}

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

pub fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn cleanup_game_hud(mut commands: Commands, query: Query<Entity, With<GameHudRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
pub struct BackButton;
#[derive(Component)]
pub struct SaveHouseListItem(pub String);
#[derive(Component)]
pub struct DeleteSaveButton(pub String);

#[derive(Component)]
pub struct GameHudRoot;

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

pub fn cleanup_saved_houses_menu(
    mut commands: Commands,
    query: Query<Entity, With<SavedHousesRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

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
                Text::new("_"),
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

pub fn cleanup_new_house_menu(mut commands: Commands, query: Query<Entity, With<NewHouseRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

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
    for interaction in back_query.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::MainMenu);
        }
    }

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

    for (interaction, del) in delete_query.iter() {
        if *interaction == Interaction::Pressed {
            save::delete_save_file(&del.0);
            if let Ok(entity) = root_query.single() {
                commands.entity(entity).despawn();
            }
            setup_saved_houses_menu(commands);
            return;
        }
    }
}

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
    if keyboard.just_pressed(KeyCode::Escape) {
        text_input.text.clear();
        next_state.set(GameState::MainMenu);
        return;
    }

    for ev in char_evr.read() {
        if let Some(text) = &ev.text {
            for c in text.chars() {
                if !c.is_control() {
                    text_input.text.push(c);
                }
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Backspace) {
        text_input.text.pop();
    }

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

    if let Ok(mut text) = text_query.single_mut() {
        text.0 = if text_input.text.is_empty() {
            "_".to_string()
        } else {
            format!("{}_", text_input.text)
        };
    }
}

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            PauseMenuRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: FontSize::Px(32.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

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
                    BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
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

            parent.spawn((
                Text::new(
                    "Move: WASD\nElevation: Space / Shift\nRotate view: Hold R + WASD\nSelect colour: Colour button\nDelete block: P\nChange size: Z / X / C\nPause: Esc",
                ),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    display: Display::None,
                    ..default()
                },
                ControlsText,
            ));

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
                    BackgroundColor(Color::srgb(0.2, 0.4, 0.7)),
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
                    BackgroundColor(Color::srgb(0.7, 0.2, 0.2)),
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

pub fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn pause_menu_interaction_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    current_world: Res<CurrentWorld>,
    blocks_query: Query<(&PlacedBlock, &MeshMaterial3d<StandardMaterial>), With<PlacedBlock>>,
    materials: Res<Assets<StandardMaterial>>,
    pause_menu_query: Query<Entity, With<PauseMenuRoot>>,
    controls_query: Query<&Interaction, (Changed<Interaction>, With<ControlsButton>)>,
    mut controls_text_query: Query<&mut Node, With<ControlsText>>,
    resume_query: Query<&Interaction, (Changed<Interaction>, With<ResumeButton>)>,
    save_query: Query<&Interaction, (Changed<Interaction>, With<PauseSaveButton>)>,
    save_quit_query: Query<&Interaction, (Changed<Interaction>, With<PauseSaveAndQuitButton>)>,
) {
    for interaction in resume_query.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Playing);
        }
    }

    for interaction in controls_query.iter() {
        if *interaction == Interaction::Pressed {
            for mut node in controls_text_query.iter_mut() {
                node.display = match node.display {
                    Display::None => Display::Flex,
                    _ => Display::None,
                };
            }
        }
    }

    for interaction in save_query.iter() {
        if *interaction == Interaction::Pressed {
            save::save_house_to_disk(&current_world.name, &blocks_query, &materials);
        }
    }

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
