//! Game systems for physics, collision, rendering, and course management.

pub mod collision;
pub mod course;
pub mod movement;
pub mod rendering;
pub mod run_flow;
pub mod settings_ui;
pub mod ski_lodge;

use crate::states::GameState;
use bevy::prelude::*;

/// Plugin that registers all core gameplay systems.
pub struct GameSystemsPlugin;

impl Plugin for GameSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<settings_ui::SettingsDialogState>();
        app.init_resource::<run_flow::ExtractedSpriteAtlas>();
        app.add_systems(
            Startup,
            (
                settings_ui::setup_settings_dialog,
                settings_ui::initialize_window_metrics,
                run_flow::refresh_extracted_sprite_atlas,
            ),
        );

        app.add_systems(
            Update,
            (
                movement::skier_input,
                movement::adjust_skier,
                collision::check_collisions,
                run_flow::update_run_stats,
                run_flow::check_run_completion,
                run_flow::handle_crash_recovery,
                run_flow::animate_rescue_dog,
                run_flow::update_run_hud,
                rendering::update_skier_transform,
                rendering::render_obstacles,
                rendering::camera_follow_skier,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );

        app.add_systems(OnEnter(GameState::Playing), run_flow::spawn_run_hud);

        app.add_systems(
            OnExit(GameState::Playing),
            run_flow::cleanup_playing_entities,
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
            run_flow::return_to_ski_lodge_shortcut.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            run_flow::handle_game_over_input.run_if(in_state(GameState::GameOver)),
        );
        app.add_systems(OnEnter(GameState::GameOver), run_flow::spawn_game_over_ui);
        app.add_systems(OnExit(GameState::GameOver), run_flow::cleanup_game_over_ui);
    }
}
