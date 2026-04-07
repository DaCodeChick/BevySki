// ECS Components for BevySki

use bevy::prelude::*;

/// The player's skier
#[derive(Component)]
pub struct Skier {
    /// Current speed (0-1000+)
    pub speed: f32,
    /// Angle/direction (10-130 degrees, 70 is straight down)
    pub angle: i16,
    /// Animation state (0-5 for different skiing poses)
    pub animation_frame: u8,
    /// Is the skier currently in a jump/trick state
    pub is_jumping: bool,
    /// Is the skier currently crashed
    pub is_crashed: bool,
}

impl Default for Skier {
    fn default() -> Self {
        Self {
            speed: 0.0,
            angle: crate::constants::CENTER_ANGLE,
            animation_frame: 0,
            is_jumping: false,
            is_crashed: false,
        }
    }
}

/// Skier's position on the course (different from Transform)
#[derive(Component)]
pub struct CoursePosition {
    /// Horizontal position on the course (10 - 950 approx)
    pub x: f32,
    /// Vertical distance down the course (increases as player moves down)
    pub distance: f32,
}

impl Default for CoursePosition {
    fn default() -> Self {
        Self {
            x: crate::constants::COURSE_WIDTH / 2.0,
            distance: 0.0,
        }
    }
}

/// Ski trail marks left by the skier
#[derive(Component)]
pub struct SkiTrail {
    pub start_pos: Vec2,
    pub end_pos: Vec2,
    pub angle: i16,
}

/// Types of obstacles found on ski courses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObstacleType {
    /// Tree obstacle (causes crash).
    Tree,
    /// Rock obstacle (causes crash).
    Rock,
    /// Jump ramp.
    Jump,
    /// Flag to collect.
    Flag,
    /// Other obstacle type.
    Other(u8),
}

impl From<u8> for ObstacleType {
    fn from(image_id: u8) -> Self {
        match image_id {
            0..=5 => ObstacleType::Tree,
            6..=10 => ObstacleType::Rock,
            11..=13 => ObstacleType::Jump,
            14..=16 => ObstacleType::Flag,
            _ => ObstacleType::Other(image_id),
        }
    }
}

/// Obstacle on the ski course
#[derive(Component)]
pub struct Obstacle {
    pub obstacle_type: ObstacleType,
    /// Image/sprite ID
    pub image_id: u8,
    /// Course distance where obstacle is located
    pub course_distance: f32,
    /// Horizontal position
    pub course_x: f32,
    /// Collision bounds
    pub bounds: Vec2,
}

/// Player score and statistics
#[derive(Component, Default)]
pub struct Score {
    pub distance: f32,
    pub time: f32,
    pub flags_collected: u32,
    pub jumps_performed: u32,
    pub crashes: u32,
}

/// Game timer
#[derive(Component)]
pub struct GameTimer {
    pub elapsed: f32,
}

/// Wind/weather effects
#[derive(Component)]
pub struct Wind {
    pub horizontal_force: f32,
    pub vertical_force: f32,
}

/// Camera that follows the skier
#[derive(Component)]
pub struct SkiCamera;
