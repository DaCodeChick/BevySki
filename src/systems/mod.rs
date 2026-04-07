// Game systems

pub mod collision;
pub mod course;
pub mod movement;
pub mod rendering;

use bevy::prelude::*;

pub struct GameSystemsPlugin;

impl Plugin for GameSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                movement::skier_input,
                movement::adjust_skier,
                collision::check_collisions,
            )
                .chain(),
        );
    }
}
