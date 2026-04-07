//! Game systems for physics, collision, rendering, and course management.

pub mod collision;
pub mod course;
pub mod movement;
pub mod rendering;

use bevy::prelude::*;

/// Plugin that registers all core gameplay systems.
pub struct GameSystemsPlugin;

impl Plugin for GameSystemsPlugin {
    fn build(&self, app: &mut App) {
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
                .chain(),
        );
    }
}
