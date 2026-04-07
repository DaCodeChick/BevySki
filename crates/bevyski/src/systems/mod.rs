//! Game systems for physics, collision, rendering, and course management.

pub mod collision;
pub mod course;
pub mod movement;
pub mod rendering;
pub mod settings_ui;
pub mod ski_lodge;

use crate::states::GameState;
use bevy::prelude::*;

/// Plugin that registers all core gameplay systems.
pub struct GameSystemsPlugin;

impl Plugin for GameSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<settings_ui::SettingsDialogState>();
        app.add_systems(
            Startup,
            (
                settings_ui::setup_settings_dialog,
                settings_ui::initialize_window_metrics,
            ),
        );

        app.add_systems(
            Update,
            (
                movement::skier_input,
                movement::adjust_skier,
                collision::check_collisions,
                rendering::update_skier_transform,
                rendering::render_obstacles,
                rendering::camera_follow_skier,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );

        app.add_systems(
            Update,
            (
                settings_ui::toggle_settings_dialog,
                settings_ui::handle_settings_button_interaction,
                settings_ui::sync_settings_dialog,
                settings_ui::update_window_metrics_from_resize,
            ),
        );

        app.add_systems(OnEnter(GameState::SkiLodge), ski_lodge::spawn_ski_lodge_ui);
        app.add_systems(
            Update,
            ski_lodge::handle_ski_lodge_actions.run_if(in_state(GameState::SkiLodge)),
        );
        app.add_systems(OnExit(GameState::SkiLodge), ski_lodge::cleanup_ski_lodge_ui);
        app.add_systems(
            Update,
            ski_lodge::return_to_ski_lodge_shortcut.run_if(in_state(GameState::Playing)),
        );
    }
}
