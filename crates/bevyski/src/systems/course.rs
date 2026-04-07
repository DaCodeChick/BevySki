//! Course generation and management.

use crate::components::{Obstacle, ObstacleType};
use crate::resources::Course;
use bevy::prelude::*;

/// Spawns obstacles from the current course into the game world.
pub fn spawn_course_obstacles(mut commands: Commands, course: Res<Course>) {
    for course_obstacle in &course.obstacles {
        let obstacle_type = ObstacleType::from(course_obstacle.image_id);

        // Choose color based on obstacle type
        let color = match obstacle_type {
            ObstacleType::Tree => Color::srgb(0.2, 0.6, 0.2), // Green
            ObstacleType::Rock => Color::srgb(0.5, 0.5, 0.5), // Gray
            ObstacleType::Jump => Color::srgb(0.9, 0.7, 0.3), // Yellow/Gold
            ObstacleType::Flag => Color::srgb(1.0, 0.2, 0.2), // Red
            ObstacleType::Other(_) => Color::srgb(0.7, 0.5, 0.8), // Purple for unknown
        };

        commands.spawn((
            Obstacle {
                obstacle_type,
                image_id: course_obstacle.image_id,
                course_distance: course_obstacle.distance,
                course_x: course_obstacle.x,
                bounds: Vec2::new(20.0, 20.0),
            },
            Sprite {
                color,
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            Transform::from_xyz(course_obstacle.x, 0.0, 5.0),
        ));
    }
}

/// Loads a course from a JSON file.
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn load_course(path: &str) -> Result<Course, String> {
    let file_content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read course file: {}", e))?;

    let course: Course = serde_json::from_str(&file_content)
        .map_err(|e| format!("Failed to parse course: {}", e))?;

    Ok(course)
}

/// Saves a course to a JSON file.
///
/// # Errors
///
/// Returns an error if the file cannot be written or the course cannot be serialized.
pub fn save_course(course: &Course, path: &str) -> Result<(), String> {
    let json = serde_json::to_string_pretty(course)
        .map_err(|e| format!("Failed to serialize course: {}", e))?;

    std::fs::write(path, json).map_err(|e| format!("Failed to write course file: {}", e))?;

    Ok(())
}
