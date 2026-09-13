use bevy::prelude::*;
use bevy::text::FontSize;
use bevy_egui::{egui, EguiContexts};

use crate::types::{
    BlockMenuButton, BlockMenuState, ColorInputBuffer, ColorPickerState, ColourButton,
    CoordinateText, EditCoordinatesButton, EditPositionState, GameMode, PlacedBlock,
    PlacementSettings, PlayerDebugCoordinates, StatusText,
};
use crate::ui::components::*;
use crate::ui::helpers::*;

/// Spawns the main in-game Heads-Up Display (HUD), including status text, color/block menus,
/// and player debug coordinates.
pub fn setup_ui(mut commands: Commands) {
    // Spawn the left-side UI panel (Controls, Status, Block settings)
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
            // HUD Title
            parent.spawn((
                Text::new("3D House Builder Controls"),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Status indicator (e.g., current mode, rotations, size)
            parent.spawn((
                Text::new("Initializing..."),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.4)),
                StatusText,
            ));

            // Block color section
            parent.spawn((
                Text::new("Choose Block Color:"),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            spawn_compact_button(parent, "Colour", ColourButton);

            // Block shape section
            parent.spawn((
                Text::new("Choose Block Shape:"),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            spawn_button(parent, "Block Types", BlockMenuButton);

            // Coordinates readout and edit button
            parent.spawn((
                Text::new("Coordinates: none"),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                CoordinateText,
            ));
            spawn_compact_button(parent, "Edit", EditCoordinatesButton);
        });

    // Spawn the right-side debug panel for player coordinates
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(15.0),
            top: Val::Px(15.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.8)),
        Text::new("Player: --"),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.95, 1.0)),
        PlayerDebugCoordinates,
        GameHudRoot,
    ));
}

/// Cleans up the Game HUD nodes when exiting the play state.
pub fn cleanup_game_hud(mut commands: Commands, query: Query<Entity, With<GameHudRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// System to update the player's debug coordinates text on the screen.
pub fn update_player_debug_coordinates_system(
    mode: Res<GameMode>,
    camera_query: Query<&Transform, With<Camera3d>>,
    mut text_query: Query<&mut Text, With<PlayerDebugCoordinates>>,
) {
    let Ok(camera) = camera_query.single() else {
        return;
    };
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    // Calculate player's feet position based on the camera position in Human mode
    if *mode == GameMode::Human {
        let player_feet = camera.translation - Vec3::Y * 1.75;
        text.0 = format!(
            "Player\nX: {:.3}\nY: {:.3}\nZ: {:.3}",
            camera.translation.x, player_feet.y, camera.translation.z
        );
    } else {
        text.0 = "Player\nFree Mode".to_string();
    }
}

/// Handles standard HUD button clicks to toggle Egui windows (Color Picker, Block Menu, Edit Coord).
pub fn ui_interaction_system(
    mode: Res<GameMode>,
    mut picker: ResMut<ColorPickerState>,
    mut block_menu: ResMut<BlockMenuState>,
    mut edit_position: ResMut<EditPositionState>,
    colour_buttons: Query<&Interaction, (Changed<Interaction>, With<ColourButton>)>,
    block_menu_buttons: Query<&Interaction, (Changed<Interaction>, With<BlockMenuButton>)>,
    edit_buttons: Query<&Interaction, (Changed<Interaction>, With<EditCoordinatesButton>)>,
) {
    // Disable interactions if the player is in Human mode (e.g., mouse is captured)
    if *mode == GameMode::Human {
        return;
    }

    if picker.is_open || block_menu.is_open || edit_position.is_open {
        return;
    }

    // Toggle color picker window
    for interaction in colour_buttons.iter() {
        if *interaction == Interaction::Pressed {
            picker.is_open = true;
            block_menu.is_open = false;
            edit_position.is_open = false;
        }
    }

    // Toggle block presets window
    for interaction in block_menu_buttons.iter() {
        if *interaction == Interaction::Pressed {
            block_menu.is_open = true;
            picker.is_open = false;
            edit_position.is_open = false;
        }
    }

    // Toggle edit position window (only if a block is currently selected for editing)
    for interaction in edit_buttons.iter() {
        if *interaction == Interaction::Pressed && edit_position.entity.is_some() {
            edit_position.is_open = true;
            picker.is_open = false;
            block_menu.is_open = false;
        }
    }
}

/// Renders and handles logic for all Egui overlay windows (Color Picker, Block Shapes, Coordinates).
pub fn color_picker_system(
    mut contexts: EguiContexts,
    mut settings: ResMut<PlacementSettings>,
    mode: Res<GameMode>,
    mut color_input: ResMut<ColorInputBuffer>,
    mut picker: ResMut<ColorPickerState>,
    mut block_menu: ResMut<BlockMenuState>,
    mut edit_position: ResMut<EditPositionState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    block_materials: Query<&MeshMaterial3d<StandardMaterial>, With<PlacedBlock>>,
    mut blocks_query: Query<(&mut Transform, &mut PlacedBlock)>,
    mut coordinate_text: Query<&mut Text, With<CoordinateText>>,
) {
    if *mode == GameMode::Human {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // ----------------------------------------------------
    // Color Picker Egui Window
    // ----------------------------------------------------
    if picker.is_open {
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
                // Native Egui Color Wheel
                if egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut color,
                    egui::color_picker::Alpha::Opaque,
                )
                .changed()
                {
                    // Apply newly picked color from the wheel
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
                    apply_edit_color(&settings, &mut materials, &block_materials);
                }

                ui.label("Hexadecimal");
                // Text input for Hex codes
                if ui.text_edit_singleline(&mut hex).changed() {
                    color_input.text = hex.clone();
                    if let Some(parsed) = parse_hex_color(&hex) {
                        settings.color = parsed;
                        settings.color_name = normalize_hex(&hex);
                        color_input.text = settings.color_name.clone();
                        add_color_to_history(&mut picker.history, &settings.color_name);
                        apply_edit_color(&settings, &mut materials, &block_materials);
                    }
                }

                // History toggle
                if ui.button("History").clicked() {
                    picker.show_history = !picker.show_history;
                }

                // Render history items if expanded
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
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::Vec2::splat(14.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().circle_filled(rect.center(), 6.0, swatch);
                            }

                            // Click history item to apply it
                            if ui.button(&history_color).clicked() {
                                if let Some(parsed) = parse_hex_color(&history_color) {
                                    settings.color = parsed;
                                    settings.color_name = history_color.clone();
                                    color_input.text = history_color.clone();
                                    apply_edit_color(&settings, &mut materials, &block_materials);
                                }
                            }
                        });
                    }
                }
            });
        picker.is_open = is_open;
    }

    // ----------------------------------------------------
    // Block Shapes Egui Window
    // ----------------------------------------------------
    if block_menu.is_open {
        let mut is_open = block_menu.is_open;
        egui::Window::new("Block Types")
            .default_width(260.0)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.label("Choose a shape and size for the next block");
                // Iterate through predefined sizes
                for (label, size) in block_presets() {
                    if ui
                        .button(format!("{label}  ({})", format_size(size)))
                        .clicked()
                    {
                        settings.size = size;
                        settings.size_name = label;
                        settings.rotation_x = 0.0;
                        settings.rotation_y = 0.0;
                        settings.rotation = Quat::IDENTITY;
                        block_menu.is_open = false;
                    }
                }
            });
        block_menu.is_open = is_open && block_menu.is_open;
    }

    // ----------------------------------------------------
    // Edit Coordinate Egui Window
    // ----------------------------------------------------
    if edit_position.is_open {
        let Some(entity) = edit_position.entity else {
            edit_position.is_open = false;
            return;
        };
        let mut position = EditPositionState {
            entity: edit_position.entity,
            is_open: edit_position.is_open,
            x: edit_position.x,
            y: edit_position.y,
            z: edit_position.z,
            x_text: edit_position.x_text.clone(),
            y_text: edit_position.y_text.clone(),
            z_text: edit_position.z_text.clone(),
            warning: edit_position.warning.clone(),
        };

        // Sync internal state if the entity changed or if texts are empty
        if position.entity != Some(entity)
            || position.x_text.is_empty()
            || position.y_text.is_empty()
            || position.z_text.is_empty()
        {
            if let Ok((transform, _)) = blocks_query.get(entity) {
                position.entity = Some(entity);
                position.x = transform.translation.x;
                position.y = transform.translation.y;
                position.z = transform.translation.z;
                position.x_text = format_coordinate(position.x);
                position.y_text = format_coordinate(position.y);
                position.z_text = format_coordinate(position.z);
                position.warning = None;
            }
        }

        let mut edit_open = true;
        egui::Window::new("Edit Block Position")
            .default_width(220.0)
            .open(&mut edit_open)
            .show(ctx, |ui| {
                ui.label("Coordinates");
                coordinate_field(
                    ui,
                    "X",
                    &mut position.x,
                    &mut position.x_text,
                    &mut position.warning,
                );
                coordinate_field(
                    ui,
                    "Y",
                    &mut position.y,
                    &mut position.y_text,
                    &mut position.warning,
                );
                coordinate_field(
                    ui,
                    "Z",
                    &mut position.z,
                    &mut position.z_text,
                    &mut position.warning,
                );

                // Show warning if a non-number is inputted
                if let Some(warning) = &position.warning {
                    ui.colored_label(egui::Color32::from_rgb(255, 170, 80), warning);
                }
            });

        // Apply coordinate changes to the actual block component
        if edit_open {
            let translation = Vec3::new(position.x, position.y, position.z);
            *edit_position = position;
            if let Ok((mut transform, mut block)) = blocks_query.get_mut(entity) {
                transform.translation = translation;
                block.center = transform.translation;
            }
        } else {
            edit_position.is_open = false;
        }
    }

    // Update the native Bevy UI text to reflect currently targeted block's coordinates
    for mut text in coordinate_text.iter_mut() {
        if let Some(entity) = edit_position.entity {
            if let Ok((transform, _)) = blocks_query.get(entity) {
                text.0 = format!(
                    "Coordinates: X {:.3}, Y {:.3}, Z {:.3}",
                    transform.translation.x, transform.translation.y, transform.translation.z
                );
            }
        } else {
            text.0 = "Coordinates: none".to_string();
        }
    }
}

/// Refreshes the on-screen text that shows what mode/color/size the player is currently using.
pub fn update_status_text_system(
    settings: Res<PlacementSettings>,
    mut text_query: Query<&mut Text, With<StatusText>>,
) {
    if settings.is_changed() {
        if let Ok(mut text) = text_query.single_mut() {
            let mode_str = if settings.editing_mode {
                "EDIT"
            } else if settings.is_deleting {
                "DELETE"
            } else {
                "BUILD"
            };
            text.0 = format!(
                "Mode: {}\nColor: {}\nSize: {}\nRotation: {}",
                mode_str,
                settings.color_name,
                settings.size_name,
                format!(
                    "X {}° / Y {}°",
                    settings.rotation_x as i32, settings.rotation_y as i32
                )
            );
        }
    }
}