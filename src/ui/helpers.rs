use bevy::prelude::*;
use bevy::text::FontSize;
use bevy_egui::egui;

use crate::types::PlacedBlock;
use crate::types::PlacementSettings;

/// Helper to spawn a standard-sized button with text and a custom marker component.
pub fn spawn_button<C: Component>(parent: &mut ChildSpawnerCommands, label: &str, marker: C) {
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

/// Helper to spawn a more compact button (smaller padding/font) with a marker component.
pub fn spawn_compact_button<C: Component>(parent: &mut ChildSpawnerCommands, label: &str, marker: C) {
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

/// Helper to spawn a large main menu button.
pub fn spawn_menu_button<C: Component>(parent: &mut ChildSpawnerCommands, label: &str, marker: C) {
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

/// Helper to spawn action buttons for the pause menu.
pub fn spawn_pause_action_button<C: Component>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: C,
) {
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

/// Defines predefined sizes for block building.
pub fn block_presets() -> [(&'static str, Vec3); 9] {
    [
        ("Cube", Vec3::new(1.0, 1.0, 1.0)),
        ("Small Cube", Vec3::new(0.5, 0.5, 0.5)),
        ("Tall Wall", Vec3::new(1.0, 2.0, 1.0)),
        ("Wide Wall", Vec3::new(2.0, 1.0, 1.0)),
        ("Long Beam", Vec3::new(3.0, 0.5, 0.5)),
        ("Floor Slab", Vec3::new(3.0, 0.25, 3.0)),
        ("Pillar", Vec3::new(0.75, 3.0, 0.75)),
        ("Large Block", Vec3::new(2.0, 2.0, 2.0)),
        ("Wide Slab", Vec3::new(4.0, 0.5, 2.0)),
    ]
}

/// Formats a Vec3 size into a readable string (e.g., "1.00 x 2.00 x 1.00").
pub fn format_size(size: Vec3) -> String {
    format!("{:.2} x {:.2} x {:.2}", size.x, size.y, size.z)
}

/// Formats a coordinate float for the UI text.
pub fn format_coordinate(value: f32) -> String {
    format!("{value:.3}")
}

/// Renders a single coordinate input row in Egui (Label + TextBox), handling parsing and validation.
pub fn coordinate_field(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    text: &mut String,
    warning: &mut Option<String>,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let response = ui.text_edit_singleline(text);
        if response.changed() {
            match text.trim().parse::<f32>() {
                Ok(parsed) if parsed.is_finite() => {
                    *value = parsed;
                    *warning = None;
                }
                _ => {
                    *warning = Some(format!("{label} must be a valid number."));
                }
            }
        }

        // Restore previous value if input is lost focus and invalid
        let valid = text
            .trim()
            .parse::<f32>()
            .map(|parsed| parsed.is_finite())
            .unwrap_or(false);
        if response.lost_focus() && !valid {
            *text = format_coordinate(*value);
            *warning = Some(format!("{label} was invalid and has been restored."));
        }
    });
}

/// Modifies the standard material of the block currently being edited.
pub fn apply_edit_color(
    settings: &PlacementSettings,
    materials: &mut Assets<StandardMaterial>,
    block_materials: &Query<&MeshMaterial3d<StandardMaterial>, With<PlacedBlock>>,
) {
    let Some(entity) = settings.editing_entity else {
        return;
    };
    let Ok(material_handle) = block_materials.get(entity) else {
        return;
    };
    if let Some(mut material) = materials.get_mut(&material_handle.0) {
        material.base_color = settings.color;
    }
}

/// Ensures unique colors are saved to the history array and capped at 15 items.
pub fn add_color_to_history(history: &mut Vec<String>, color: &str) {
    history.retain(|entry| entry != color);
    history.insert(0, color.to_string());
    history.truncate(15);
}

/// Safely parses a hex color string (e.g. "#FF0000" or "00FF00FF") into a Bevy `Color`.
pub fn parse_hex_color(value: &str) -> Option<Color> {
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

/// Forces hex string to standard `#XXXXXX` uppercase format.
pub fn normalize_hex(value: &str) -> String {
    format!("#{}", value.trim().trim_start_matches('#').to_uppercase())
}

/// Helper function to spawn a column of instructions with a title for the controls menu.
pub fn spawn_controls_column(parent: &mut ChildSpawnerCommands, title: &str, controls: &str) {
    parent
        .spawn(Node {
            width: Val::Px(260.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|column| {
            // Column Title (e.g., "Free Mode")
            column.spawn((
                Text::new(title),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.95, 1.0)),
            ));
            // Controls List
            column.spawn((
                Text::new(controls),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}