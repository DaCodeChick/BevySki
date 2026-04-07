//! Global game resources.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A ski course with obstacles and difficulty settings.
#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct Course {
    /// Name of the course.
    pub name: String,
    /// Slope difficulty level (0-3).
    pub slope_difficulty: u8,
    /// Required skill level (0-2).
    pub skill_level: u8,
    /// Total length of the course in units.
    pub length: f32,
    /// Obstacles placed along the course.
    pub obstacles: Vec<CourseObstacle>,
}

/// An obstacle on the ski course.
#[derive(Serialize, Deserialize, Clone)]
pub struct CourseObstacle {
    /// Sprite/image identifier for this obstacle.
    pub image_id: u8,
    /// Distance down the course where this obstacle is located.
    pub distance: f32,
    /// Horizontal position on the course.
    pub x: f32,
}

impl Default for Course {
    fn default() -> Self {
        Self {
            name: "Default Course".to_string(),
            slope_difficulty: 1,
            skill_level: 1,
            length: 100000.0,
            obstacles: Vec::new(),
        }
    }
}

impl Course {
    /// Generate a random course
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut course = Self {
            name: format!("Random Course {}", rng.gen_range(1..1000)),
            slope_difficulty: rng.gen_range(0..4),
            skill_level: rng.gen_range(0..3),
            length: rng.gen_range(50000.0..200000.0),
            obstacles: Vec::new(),
        };

        // Generate obstacles along the course
        let mut current_distance = 500.0;
        while current_distance < course.length {
            let obstacle = CourseObstacle {
                image_id: rng.gen_range(0..20), // Different obstacle types
                distance: current_distance,
                x: rng.gen_range(50.0..900.0),
            };
            course.obstacles.push(obstacle);
            current_distance += rng.gen_range(200.0..800.0);
        }

        course
    }
}

/// Game configuration settings.
#[derive(Resource)]
pub struct GameSettings {
    /// Whether sound effects are enabled.
    pub sound_enabled: bool,
    /// Whether animations are enabled.
    pub animation_enabled: bool,
    /// Whether ski trails should be shown.
    pub show_trails: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            sound_enabled: true,
            animation_enabled: true,
            show_trails: true,
        }
    }
}
