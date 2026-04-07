// Global resources

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Course data structure
#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct Course {
    pub name: String,
    pub slope_difficulty: u8,
    pub skill_level: u8,
    pub length: f32,
    pub obstacles: Vec<CourseObstacle>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CourseObstacle {
    pub image_id: u8,
    pub distance: f32,
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

/// Game settings
#[derive(Resource)]
pub struct GameSettings {
    pub sound_enabled: bool,
    pub animation_enabled: bool,
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
