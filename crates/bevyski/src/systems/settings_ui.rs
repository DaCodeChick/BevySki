//! UI systems for in-game settings dialog.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::window::WindowResized;

use crate::resources::{GameSettingFlags, GameSettings};

#[derive(Component)]
pub(crate) struct SettingsDialogRoot;

#[derive(Resource, Default)]
pub(crate) struct SettingsDialogState {
    visible: bool,
}

#[derive(Component)]
pub(crate) struct SettingsToggleButton {
    flag: GameSettingFlags,
}

#[derive(Component)]
pub(crate) struct SettingsToggleLabel {
    flag: GameSettingFlags,
}

/// Spawns a hidden settings dialog with toggle buttons.
pub fn setup_settings_dialog(mut commands: Commands) {
    commands
        .spawn((
            SettingsDialogRoot,
            Node {
                position_type: PositionType::Absolute,
                top: px(20.0),
                right: px(20.0),
                width: px(300.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(10.0),
                padding: UiRect::all(px(14.0)),
                border: UiRect::all(px(2.0)),
                ..default()
            },
            Visibility::Hidden,
            BackgroundColor(Color::srgba(0.08, 0.12, 0.18, 0.92)),
            BorderColor::all(Color::srgb(0.4, 0.6, 0.8)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Settings"),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.96, 1.0)),
            ));

            spawn_toggle_row(parent, GameSettingFlags::SOUND, "Sound");
            spawn_toggle_row(parent, GameSettingFlags::ANIMATION, "Animation");
            spawn_toggle_row(parent, GameSettingFlags::TRAILS, "Trails");

            parent.spawn((
                Text::new("Press F1 to close"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.82, 0.92)),
            ));
        });
}

/// Initializes window metrics from the primary window at startup.
pub fn initialize_window_metrics(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut settings: ResMut<GameSettings>,
) {
    if let Ok(window) = window_query.single() {
        settings.update_window_metrics(window.width(), window.height());
    }
}

fn spawn_toggle_row(parent: &mut ChildSpawnerCommands, flag: GameSettingFlags, title: &str) {
    parent
        .spawn((Node {
            width: percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_children(|row| {
            row.spawn((
                Text::new(title),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.96, 1.0)),
            ));

            row.spawn((
                Button,
                SettingsToggleButton { flag },
                Node {
                    width: px(110.0),
                    height: px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.26, 0.36, 0.46)),
            ))
            .with_children(|button| {
                button.spawn((
                    SettingsToggleLabel { flag },
                    Text::new("OFF"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.95, 0.95)),
                ));
            });
        });
}

/// Toggles settings dialog visibility with F1.
pub fn toggle_settings_dialog(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut dialog_state: ResMut<SettingsDialogState>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        dialog_state.visible = !dialog_state.visible;
    }
}

/// Handles clicks on settings toggle buttons.
pub fn handle_settings_button_interaction(
    mut interactions: Query<
        (&Interaction, &SettingsToggleButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut settings: ResMut<GameSettings>,
) {
    for (interaction, toggle) in &mut interactions {
        if *interaction == Interaction::Pressed {
            settings.toggle(toggle.flag);
        }
    }
}

/// Synchronizes dialog visibility and button text with current settings.
pub fn sync_settings_dialog(
    settings: Res<GameSettings>,
    dialog_state: Res<SettingsDialogState>,
    mut dialog_visibility: Query<&mut Visibility, With<SettingsDialogRoot>>,
    mut labels: Query<(&SettingsToggleLabel, &mut Text, &mut TextColor)>,
    mut buttons: Query<(&SettingsToggleButton, &mut BackgroundColor)>,
) {
    if !settings.is_changed() && !dialog_state.is_changed() {
        return;
    }

    let dialog_visible = dialog_state.visible;
    for mut visibility in &mut dialog_visibility {
        *visibility = if dialog_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (label, mut text, mut color) in &mut labels {
        if settings.is_enabled(label.flag) {
            *text = Text::new("ON");
            *color = TextColor(Color::srgb(0.66, 1.0, 0.72));
            continue;
        }

        *text = Text::new("OFF");
        *color = TextColor(Color::srgb(1.0, 0.7, 0.7));
    }

    for (toggle, mut background) in &mut buttons {
        if settings.is_enabled(toggle.flag) {
            *background = BackgroundColor(Color::srgb(0.16, 0.52, 0.28));
            continue;
        }

        *background = BackgroundColor(Color::srgb(0.46, 0.2, 0.24));
    }
}

/// Tracks window resize events and updates settings metrics.
pub fn update_window_metrics_from_resize(
    mut resize_events: MessageReader<WindowResized>,
    mut settings: ResMut<GameSettings>,
) {
    for event in resize_events.read() {
        settings.update_window_metrics(event.width, event.height);
    }
}
