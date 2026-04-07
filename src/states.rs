// Game states

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    MainMenu,
    CourseSelect,
    Playing,
    Paused,
    GameOver,
    Editor,
}
