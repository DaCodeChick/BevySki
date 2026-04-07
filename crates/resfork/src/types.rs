//! MacSki-specific resource type definitions and parsers.

use crate::ResType;

/// MacSki resource types found in the game files.
pub mod macski {
    use super::*;

    /// Color resources ("COLRiSki").
    pub const COLR: ResType = ResType::new(b"COLR");

    /// Hill/course terrain ("HILLiSki").
    pub const HILL: ResType = ResType::new(b"HILL");

    /// Obstacle/sprite graphics.
    pub const PICT: ResType = ResType::new(b"PICT");

    /// Sound resources.
    pub const SND: ResType = ResType::new(b"snd ");

    /// String resources.
    pub const STR: ResType = ResType::new(b"STR ");

    /// Version information.
    pub const VERS: ResType = ResType::new(b"vers");
}

/// Course file structure parsed from MacSki course resources.
#[derive(Debug, Clone)]
pub struct CourseData {
    /// Course name.
    pub name: String,
    /// Course obstacles.
    pub obstacles: Vec<CourseObstacle>,
}

/// A single obstacle in a course.
#[derive(Debug, Clone)]
pub struct CourseObstacle {
    /// Image/sprite ID.
    pub image_id: u8,
    /// Horizontal position on course.
    pub x: f32,
    /// Vertical distance down course.
    pub distance: f32,
}
