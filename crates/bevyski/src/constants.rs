//! Game constants for physics, rendering, and gameplay.

// Physics constants

/// Maximum speed the skier can achieve (in units per second).
pub const MAX_SPEED: f32 = 1000.0;

/// Rate at which the skier accelerates when skiing downhill.
pub const ACCELERATION_RATE: f32 = 2.0;

/// Rate at which the skier decelerates when braking.
pub const DECELERATION_RATE: f32 = 7.0;

/// Sensitivity of turning controls.
pub const TURN_SENSITIVITY: f32 = 0.1;

/// Multiplier for gravity effects on the skier.
pub const GRAVITY_MULTIPLIER: f32 = 1.0;

// Skier angles (degrees, 0-180)

/// Minimum skiing angle (hard left).
pub const MIN_SKIER_ANGLE: i16 = 10;

/// Maximum skiing angle (hard right).
pub const MAX_SKIER_ANGLE: i16 = 130;

/// Center skiing angle (straight down the slope).
pub const CENTER_ANGLE: i16 = 70;

// Collision detection

/// Distance threshold for collision detection with obstacles.
pub const COLLISION_DISTANCE_THRESHOLD: f32 = 20.0;

// Animation frame timing

/// Time between animation frame updates.
pub const ANIMATION_SPEED: f32 = 0.1;

// Course dimensions

/// Width of the ski course.
pub const COURSE_WIDTH: f32 = 950.0;

/// Height of each course segment for generation.
pub const COURSE_SEGMENT_HEIGHT: f32 = 80.0;

// Display

/// Viewport width for rendering.
pub const VIEWPORT_WIDTH: f32 = 800.0;

/// Viewport height for rendering.
pub const VIEWPORT_HEIGHT: f32 = 600.0;
