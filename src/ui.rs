use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::text::FontSize;

use crate::types::{
    ColorButton,
    GameState,
    PauseMenuRoot,
    PlacementSettings,
    QuitButton,
    ResumeButton,
    SizeButton,
    StatusText,
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
                Text::new("Select Color (1-4):"),
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
                        "Wood",
                        ColorButton(
                            Color::srgb(0.8, 0.7, 0.6),
                            "Wood / Beige",
                        ),
                    );

                    spawn_button(
                        row,
                        "Brick",
                        ColorButton(
                            Color::srgb(0.7, 0.2, 0.2),
                            "Red Brick",
                        ),
                    );

                    spawn_button(
                        row,
                        "Stone",
                        ColorButton(
                            Color::srgb(0.5, 0.5, 0.5),
                            "Gray Stone",
                        ),
                    );

                    spawn_button(
                        row,
                        "Blue",
                        ColorButton(
                            Color::srgb(0.2, 0.5, 0.8),
                            "Blue Roof",
                        ),
                    );
                });

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
                        SizeButton(
                            Vec3::new(1.0, 1.0, 1.0),
                            "Standard Wall (1x1x1)",
                        ),
                    );

                    spawn_button(
                        row,
                        "Tall 1x2",
                        SizeButton(
                            Vec3::new(1.0, 2.0, 1.0),
                            "Tall Wall (1x2x1)",
                        ),
                    );

                    spawn_button(
                        row,
                        "Wide 2x1",
                        SizeButton(
                            Vec3::new(2.0, 1.0, 1.0),
                            "Wide Wall (2x1x1)",
                        ),
                    );
                });

            parent.spawn((
                Text::new(
                    "Delete: P | Rotate: R | Move: WASD | Elevation: Q/E | Pause: Esc",
                ),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn spawn_button<C: Component>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: C,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(
                    Val::Px(10.0),
                    Val::Px(6.0),
                ),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(
                0.25,
                0.25,
                0.25,
            )),
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

pub fn ui_interaction_system(
    mut settings: ResMut<PlacementSettings>,
    color_buttons: Query<
        (&Interaction, &ColorButton),
        (Changed<Interaction>, With<Button>),
    >,
    size_buttons: Query<
        (&Interaction, &SizeButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, color_btn) in color_buttons.iter() {
        if *interaction == Interaction::Pressed {
            settings.color = color_btn.0;
            settings.color_name = color_btn.1;
        }
    }

    for (interaction, size_btn) in size_buttons.iter() {
        if *interaction == Interaction::Pressed {
            settings.size = size_btn.0;
            settings.size_name = size_btn.1;
        }
    }
}

pub fn update_status_text_system(
    settings: Res<PlacementSettings>,
    mut text_query: Query<&mut Text, With<StatusText>>,
) {
    if settings.is_changed() {
        if let Ok(mut text) = text_query.single_mut() {
            text.0 = format!(
                "Color: {}\nSize: {}\nRotation: {}°",
                settings.color_name,
                settings.size_name,
                settings.rotation_angle as i32
            );
        }
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
            BackgroundColor(Color::srgba(
                0.0,
                0.0,
                0.0,
                0.75,
            )),
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
                    BackgroundColor(Color::srgb(
                        0.2,
                        0.6,
                        0.2,
                    )),
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
                    BackgroundColor(Color::srgb(
                        0.7,
                        0.2,
                        0.2,
                    )),
                    BorderColor::all(Color::WHITE),
                    QuitButton,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Quit"),
                        TextFont {
                            font_size: FontSize::Px(16.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

pub fn cleanup_pause_menu(
    mut commands: Commands,
    query: Query<Entity, With<PauseMenuRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn pause_menu_interaction_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: MessageWriter<AppExit>,
    resume_query: Query<
        &Interaction,
        (Changed<Interaction>, With<ResumeButton>),
    >,
    quit_query: Query<
        &Interaction,
        (Changed<Interaction>, With<QuitButton>),
    >,
) {
    for interaction in resume_query.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Playing);
        }
    }

    for interaction in quit_query.iter() {
        if *interaction == Interaction::Pressed {
            app_exit.write(AppExit::Success);
        }
    }
}
