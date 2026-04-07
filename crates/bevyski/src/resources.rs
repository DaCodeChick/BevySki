//! Global game resources.

use bevy::prelude::*;
use bitflags::bitflags;
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
        use rand::RngExt;
        let mut rng = rand::rng();

        let mut course = Self {
            name: format!("Random Course {}", rng.random_range(1..1000)),
            slope_difficulty: rng.random_range(0..4),
            skill_level: rng.random_range(0..3),
            length: rng.random_range(50000.0..200000.0),
            obstacles: Vec::new(),
        };

        // Generate obstacles along the course
        let mut current_distance = 500.0;
        while current_distance < course.length {
            let obstacle = CourseObstacle {
                image_id: rng.random_range(0..20), // Different obstacle types
                distance: current_distance,
                x: rng.random_range(50.0..900.0),
            };
            course.obstacles.push(obstacle);
            current_distance += rng.random_range(200.0..800.0);
        }

        course
    }
}

/// Game configuration settings.
#[derive(Resource)]
pub struct GameSettings {
    /// Enabled runtime options stored as bit flags.
    pub flags: GameSettingFlags,
    /// Current logical window size in pixels.
    pub window_size: Vec2,
    /// Current game scale derived from window size.
    pub game_scale: f32,
}

bitflags! {
    /// Compact bit flags for runtime game options.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct GameSettingFlags: u32 {
        /// Sound effects and music are enabled.
        const SOUND = 1 << 0;
        /// Gameplay and visual animations are enabled.
        const ANIMATION = 1 << 1;
        /// Ski trails are rendered.
        const TRAILS = 1 << 2;
    }
}

impl GameSettings {
    /// Returns true when a specific setting is enabled.
    pub fn is_enabled(&self, flag: GameSettingFlags) -> bool {
        self.flags.contains(flag)
    }

    /// Enables or disables a specific setting.
    pub fn set_enabled(&mut self, flag: GameSettingFlags, enabled: bool) {
        if enabled {
            self.flags.insert(flag);
            return;
        }

        self.flags.remove(flag);
    }

    /// Toggles a specific setting flag.
    pub fn toggle(&mut self, flag: GameSettingFlags) {
        self.flags.toggle(flag);
    }

    /// Updates window metrics and derived scale.
    pub fn update_window_metrics(&mut self, width: f32, height: f32) {
        self.window_size = Vec2::new(width, height);
        let width_scale = width / crate::constants::VIEWPORT_WIDTH;
        let height_scale = height / crate::constants::VIEWPORT_HEIGHT;
        self.game_scale = width_scale.min(height_scale).max(0.1);
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            flags: GameSettingFlags::SOUND | GameSettingFlags::ANIMATION | GameSettingFlags::TRAILS,
            window_size: Vec2::new(
                crate::constants::VIEWPORT_WIDTH,
                crate::constants::VIEWPORT_HEIGHT,
            ),
            game_scale: 1.0,
        }
    }
}

/// Final outcome for a single ski run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RunOutcome {
    /// Run is still in progress or no run has completed yet.
    #[default]
    InProgress,
    /// Player reached the course end.
    Finished,
    /// Player exited the run manually.
    Aborted,
}

/// Tracks runtime lifecycle data for the currently active run.
#[derive(Resource, Debug, Default)]
pub struct RunLifecycle {
    /// Elapsed time for the active run in seconds.
    pub elapsed_seconds: f32,
    /// Elapsed time spent in crashed state in seconds.
    pub crashed_seconds: f32,
    /// Number of crashes since the last dog rescue sequence ended.
    pub crashes_since_dog: u32,
    /// True while the dog rescue sequence is active.
    pub dog_rescue_active: bool,
}

/// Summary of the most recently completed run.
#[derive(Resource, Debug)]
pub struct LastRunSummary {
    /// Last run outcome.
    pub outcome: RunOutcome,
    /// Distance reached on the run.
    pub distance: f32,
    /// Elapsed run time in seconds.
    pub time: f32,
    /// Total crashes recorded.
    pub crashes: u32,
    /// Total flags collected.
    pub flags_collected: u32,
    /// Total jumps performed.
    pub jumps_performed: u32,
}

impl Default for LastRunSummary {
    fn default() -> Self {
        Self {
            outcome: RunOutcome::InProgress,
            distance: 0.0,
            time: 0.0,
            crashes: 0,
            flags_collected: 0,
            jumps_performed: 0,
        }
    }
}
