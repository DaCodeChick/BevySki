// BevySki - A modern rewrite of the classic MacSki game
// Original MacSki by David Rowbotham (mid-1990s)

mod components;
mod constants;
mod resources;
mod states;
mod systems;

use bevy::prelude::*;
use components::{CoursePosition, GameTimer, Score, Skier};
use resources::{Course, GameSettings};
use states::GameState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "BevySki - Modern rewrite of MacSki v1.7".to_string(),
                resolution: (800., 600.).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .init_resource::<GameSettings>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::Playing), start_game)
        .add_systems(
            Update,
            (
                systems::movement::skier_input,
                systems::movement::adjust_skier,
                systems::collision::check_collisions,
                systems::rendering::update_skier_transform,
                systems::rendering::camera_follow_skier,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .run();
}

fn setup(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    // Spawn 2D camera
    commands.spawn(Camera2d);

    info!("BevySki started - Modern rewrite of MacSki v1.7");
    info!("Original game by David Rowbotham");

    // Start in playing state for now (TODO: add main menu)
    next_state.set(GameState::Playing);
}

fn start_game(mut commands: Commands) {
    info!("Starting new game...");

    // Generate a random course (or load default)
    let course = Course::random();
    info!(
        "Course: {} - {} obstacles",
        course.name,
        course.obstacles.len()
    );

    // Spawn the skier
    commands.spawn((
        Skier::default(),
        CoursePosition::default(),
        Score::default(),
        Transform::from_xyz(0.0, -100.0, 10.0),
    ));

    // Spawn course obstacles
    let course_ref = course.clone();
    commands.insert_resource(course);

    // We'll need to spawn obstacles differently
    // TODO: Create a system for this instead of calling directly

    // Start game timer
    commands.spawn(GameTimer { elapsed: 0.0 });
}
