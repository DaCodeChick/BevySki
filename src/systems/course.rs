// Course generation and management

use crate::components::{Obstacle, ObstacleType};
use crate::resources::Course;
use bevy::prelude::*;

/// Spawn obstacles from the current course
pub fn spawn_course_obstacles(mut commands: Commands, course: Res<Course>) {
    for course_obstacle in &course.obstacles {
        let obstacle_type = match course_obstacle.image_id {
            0..=5 => ObstacleType::Tree,
            6..=10 => ObstacleType::Rock,
            11..=13 => ObstacleType::Jump,
            14..=16 => ObstacleType::Flag,
            17 => ObstacleType::Yeti,
            _ => ObstacleType::Other(course_obstacle.image_id),
        };

        commands.spawn(Obstacle {
            obstacle_type,
            image_id: course_obstacle.image_id,
            course_distance: course_obstacle.distance,
            course_x: course_obstacle.x,
            bounds: Vec2::new(20.0, 20.0),
        });
    }
}

/// Load a course from file
pub fn load_course(path: &str) -> Result<Course, String> {
    // Use JSON for easier editing and portability

    let file_content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read course file: {}", e))?;

    let course: Course = serde_json::from_str(&file_content)
        .map_err(|e| format!("Failed to parse course: {}", e))?;

    Ok(course)
}

/// Save a course to file
pub fn save_course(course: &Course, path: &str) -> Result<(), String> {
    let json = serde_json::to_string_pretty(course)
        .map_err(|e| format!("Failed to serialize course: {}", e))?;

    std::fs::write(path, json).map_err(|e| format!("Failed to write course file: {}", e))?;

    Ok(())
}
