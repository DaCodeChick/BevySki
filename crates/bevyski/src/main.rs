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
use systems::GameSystemsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "BevySki - Modern rewrite of MacSki v1.7".to_string(),
                resolution: (800, 600).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GameSystemsPlugin)
        .init_state::<GameState>()
        .init_resource::<GameSettings>()
        .add_systems(Startup, setup)
        .add_systems(
            OnEnter(GameState::Playing),
            (start_game, systems::course::spawn_course_obstacles).chain(),
        )
        .add_systems(
            Update,
            (
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

    // Spawn the skier with a simple colored square
    commands.spawn((
        Skier::default(),
        CoursePosition::default(),
        Score::default(),
        Sprite {
            color: Color::srgb(0.2, 0.6, 1.0), // Blue skier
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -100.0, 10.0),
    ));

    // Insert course resource (obstacles will be spawned by course system)
    commands.insert_resource(course);

    // Start game timer
    commands.spawn(GameTimer { elapsed: 0.0 });
}
