//! Game state management.

use bevy::prelude::*;

/// Represents the current state of the game.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    /// Ski lodge (hub/menu) screen.
    #[default]
    SkiLodge,
    /// Main menu screen.
    MainMenu,
    /// Course selection screen.
    CourseSelect,
    /// Actively playing/skiing.
    Playing,
    /// Game is paused.
    Paused,
    /// Game over screen.
    GameOver,
    /// Course editor mode.
    Editor,
}
