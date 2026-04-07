//! Ski Lodge (hub/menu) UI and transitions.

use bevy::prelude::*;

use crate::states::GameState;
use crate::systems::settings_ui::SettingsDialogState;

#[derive(Component)]
pub(crate) struct SkiLodgeRoot;

#[derive(Component)]
pub(crate) enum SkiLodgeAction {
    StartRun,
    Settings,
    Quit,
}

#[derive(Component)]
pub(crate) struct SkiLodgeButtonLabel;

/// Spawns Ski Lodge UI when entering SkiLodge state.
pub fn spawn_ski_lodge_ui(mut commands: Commands) {
    commands
        .spawn((
            SkiLodgeRoot,
            Node {
                width: percent(100.0),
                height: percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.88, 0.93, 0.98)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Brew's Ski Lodge"),
                TextFont {
                    font_size: 42.0,
                    ..default()
                },
                TextColor(Color::srgb(0.12, 0.2, 0.32)),
            ));

            parent.spawn((
                Text::new("Welcome back. Pick your next run."),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.24, 0.32, 0.45)),
            ));

            spawn_button(parent, "Start Run", SkiLodgeAction::StartRun);
            spawn_button(parent, "Settings", SkiLodgeAction::Settings);
            spawn_button(parent, "Quit", SkiLodgeAction::Quit);

            parent.spawn((
                Text::new("Esc returns to Ski Lodge while skiing."),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.4, 0.54)),
            ));
        });
}

fn spawn_button(parent: &mut ChildSpawnerCommands, label: &str, action: SkiLodgeAction) {
    parent
        .spawn((
            Button,
            action,
            Node {
                width: px(240.0),
                height: px(46.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.18, 0.36, 0.62)),
        ))
        .with_children(|button| {
            button.spawn((
                SkiLodgeButtonLabel,
                Text::new(label),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// Handles Ski Lodge button interactions.
pub fn handle_ski_lodge_actions(
    mut interactions: Query<(&Interaction, &SkiLodgeAction), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut settings_dialog: ResMut<SettingsDialogState>,
) {
    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            SkiLodgeAction::StartRun => next_state.set(GameState::Playing),
            SkiLodgeAction::Settings => {
                settings_dialog.visible = !settings_dialog.visible;
            }
            SkiLodgeAction::Quit => std::process::exit(0),
        }
    }
}

/// Despawns Ski Lodge UI on state exit.
pub fn cleanup_ski_lodge_ui(mut commands: Commands, roots: Query<Entity, With<SkiLodgeRoot>>) {
    for entity in roots.iter() {
        commands.entity(entity).despawn();
    }
}

/// Lets player return to Ski Lodge from gameplay with Escape.
pub fn return_to_ski_lodge_shortcut(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::SkiLodge);
    }
}
