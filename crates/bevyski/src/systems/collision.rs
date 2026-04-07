//! Collision detection system.

use crate::components::{CoursePosition, Obstacle, ObstacleType, Skier};
use bevy::prelude::*;

/// Maximum vertical distance to check for collisions.
const VERTICAL_CHECK_DISTANCE: f32 = 100.0;

/// Maximum vertical distance for actual collision.
const COLLISION_VERTICAL_THRESHOLD: f32 = 20.0;

/// Returns the collision radius for an obstacle type.
const fn get_collision_threshold(obstacle_type: ObstacleType) -> f32 {
    match obstacle_type {
        ObstacleType::Tree => 20.0,
        ObstacleType::Rock => 15.0,
        ObstacleType::Jump => 30.0,
        ObstacleType::Flag => 25.0,
        ObstacleType::Other(_) => 20.0,
    }
}

/// Checks if skier is close enough to an obstacle for collision.
fn is_collision(skier_pos: &CoursePosition, obstacle: &Obstacle) -> bool {
    let vertical_distance = (obstacle.course_distance - skier_pos.distance).abs();
    if vertical_distance >= COLLISION_VERTICAL_THRESHOLD {
        return false;
    }

    let horizontal_distance = (obstacle.course_x - skier_pos.x).abs();
    let threshold = get_collision_threshold(obstacle.obstacle_type);

    horizontal_distance < threshold
}

/// Checks for collisions between skier and obstacles.
pub fn check_collisions(
    mut skier_query: Query<(&mut Skier, &CoursePosition)>,
    obstacle_query: Query<&Obstacle>,
) {
    for (mut skier, skier_pos) in skier_query.iter_mut() {
        if skier.is_crashed {
            continue;
        }

        for obstacle in obstacle_query.iter() {
            // Only check obstacles within reasonable range
            let distance_to_obstacle = (obstacle.course_distance - skier_pos.distance).abs();
            if distance_to_obstacle >= VERTICAL_CHECK_DISTANCE {
                continue;
            }

            if is_collision(skier_pos, obstacle) {
                handle_collision(&mut skier, obstacle.obstacle_type, skier_pos.distance);
            }
        }
    }
}

/// Handles what happens when skier hits an obstacle.
fn handle_collision(skier: &mut Skier, obstacle_type: ObstacleType, _distance: f32) {
    match obstacle_type {
        ObstacleType::Tree | ObstacleType::Rock => {
            skier.is_crashed = true;
            skier.speed *= 0.1;
            // TODO: Play crash sound
            // TODO: Trigger crash animation
            info!("Skier crashed into {:?}!", obstacle_type);
        }
        ObstacleType::Jump => {
            skier.is_jumping = true;
            // TODO: Play jump sound
            info!("Skier hit a jump!");
        }
        ObstacleType::Flag => {
            // TODO: Increment score
            // TODO: Play collection sound
            info!("Flag collected!");
        }
        ObstacleType::Other(_) => {
            skier.speed *= 0.7;
        }
    }
}

/// Checks if skier just passed an object (for scoring, gates, etc.).
///
/// # Note
///
/// This is a placeholder for future gate/scoring system.
#[allow(dead_code)]
pub fn check_passed_objects(
    skier_query: Query<&CoursePosition, With<Skier>>,
    obstacle_query: Query<&Obstacle>,
) {
    for _skier_pos in skier_query.iter() {
        for _obstacle in obstacle_query.iter() {
            // TODO: Track which objects have been passed
            // TODO: Award points for passing gates
        }
    }
}
