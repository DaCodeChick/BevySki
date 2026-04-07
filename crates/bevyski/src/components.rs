//! ECS components for BevySki.
//!
//! Defines all the entity components used in the game, including:
//! - Skier state and physics
//! - Course position tracking
//! - Obstacles and their types
//! - Score and timer tracking
//! - Visual elements like ski trails

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

/// Ski trail marks left by the skier.
///
/// Represents a visual trail segment showing where the skier has been.
#[derive(Component)]
pub struct SkiTrail {
    /// Starting position of the trail segment.
    pub start_pos: Vec2,
    /// Ending position of the trail segment.
    pub end_pos: Vec2,
    /// Angle of the skier when this trail was made.
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

/// Converts an image/sprite ID into an obstacle type.
///
/// Maps ranges of image IDs to specific obstacle types based on
/// the original MacSki asset organization.
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

/// Obstacle on the ski course.
///
/// Represents a physical object on the course that the skier can interact with.
#[derive(Component)]
pub struct Obstacle {
    /// Type classification of this obstacle.
    pub obstacle_type: ObstacleType,
    /// Image/sprite ID from original game assets.
    pub image_id: u8,
    /// Course distance where obstacle is located.
    pub course_distance: f32,
    /// Horizontal position on the course.
    pub course_x: f32,
    /// Collision bounds (width, height).
    pub bounds: Vec2,
}

/// Player score and statistics.
///
/// Tracks various gameplay metrics for the current run.
#[derive(Component, Default)]
pub struct Score {
    /// Total distance traveled down the course.
    pub distance: f32,
    /// Elapsed time in seconds.
    pub time: f32,
    /// Number of flags collected during this run.
    pub flags_collected: u32,
    /// Number of jumps performed.
    pub jumps_performed: u32,
    /// Number of times the skier has crashed.
    pub crashes: u32,
}

/// Game timer component.
///
/// Tracks elapsed time for the current game session or run.
#[derive(Component)]
pub struct GameTimer {
    /// Time elapsed in seconds.
    pub elapsed: f32,
}

/// Wind/weather effects component.
///
/// Represents environmental forces that affect skier movement.
#[derive(Component)]
pub struct Wind {
    /// Horizontal wind force (positive = push right, negative = push left).
    pub horizontal_force: f32,
    /// Vertical wind force affecting downhill speed.
    pub vertical_force: f32,
}

/// Camera that follows the skier.
///
/// Marker component for the camera entity that tracks the player.
#[derive(Component)]
pub struct SkiCamera;
