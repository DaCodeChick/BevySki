//! Rendering system for translating course coordinates to screen space.

use crate::components::{CoursePosition, Obstacle, SkiTrail, Skier};
use crate::constants::*;
use bevy::prelude::*;

/// Updates skier's screen position based on course position.
pub fn update_skier_transform(mut query: Query<(&CoursePosition, &mut Transform), With<Skier>>) {
    for (course_pos, mut transform) in query.iter_mut() {
        // Convert course coordinates to screen coordinates
        // Keep the skier centered vertically while the course scrolls
        transform.translation.x = course_pos.x - (VIEWPORT_WIDTH / 2.0);
        // Skier is typically drawn at a fixed vertical position
        transform.translation.y = -100.0;
        transform.translation.z = 10.0; // Layer above course
    }
}

/// Renders ski trails behind the skier using debug lines.
#[allow(dead_code)]
pub fn render_ski_trails(mut gizmos: Gizmos, trail_query: Query<&SkiTrail>) {
    for trail in trail_query.iter() {
        // Draw the ski trail as a line
        gizmos.line_2d(
            trail.start_pos,
            trail.end_pos,
            Color::srgba(0.8, 0.8, 0.9, 0.5),
        );
    }
}

/// Renders obstacles on the course, culling those out of view.
#[allow(dead_code)]
pub fn render_obstacles(
    mut query: Query<(&Obstacle, &mut Transform)>,
    skier_query: Query<&CoursePosition, With<Skier>>,
) {
    if let Ok(skier_pos) = skier_query.single() {
        for (obstacle, mut transform) in query.iter_mut() {
            // Calculate screen position relative to skier
            let relative_distance = obstacle.course_distance - skier_pos.distance;

            // Only show obstacles within view range
            if relative_distance.abs() < 500.0 {
                transform.translation.x = obstacle.course_x - (VIEWPORT_WIDTH / 2.0);
                transform.translation.y = relative_distance * 0.5; // Scale for screen
                transform.translation.z = 5.0;
            } else {
                // Hide far-away obstacles
                transform.translation.z = -100.0;
            }
        }
    }
}

/// Makes camera follow the skier horizontally.
pub fn camera_follow_skier(
    skier_query: Query<&CoursePosition, With<Skier>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    if let Ok(skier_pos) = skier_query.single() {
        for mut camera_transform in camera_query.iter_mut() {
            // Smooth camera following
            camera_transform.translation.x = skier_pos.x - (VIEWPORT_WIDTH / 2.0);
        }
    }
}
