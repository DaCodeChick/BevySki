// Collision detection system

use crate::components::{CoursePosition, Obstacle, ObstacleType, Skier};
use bevy::prelude::*;

/// Check for collisions between skier and obstacles
pub fn check_collisions(
    mut skier_query: Query<(&mut Skier, &CoursePosition)>,
    obstacle_query: Query<&Obstacle>,
) {
    for (mut skier, skier_pos) in skier_query.iter_mut() {
        if skier.is_crashed {
            continue;
        }

        // Check obstacles within a distance range of the skier's current position
        for obstacle in obstacle_query.iter() {
            // Only check obstacles near the skier's current distance
            let distance_to_obstacle = (obstacle.course_distance - skier_pos.distance).abs();

            if distance_to_obstacle < 100.0 {
                // Check if skier's horizontal position intersects obstacle
                let horizontal_distance = (obstacle.course_x - skier_pos.x).abs();

                // Collision bounds vary by obstacle type
                let collision_threshold = match obstacle.obstacle_type {
                    ObstacleType::Tree => 20.0,
                    ObstacleType::Rock => 15.0,
                    ObstacleType::Jump => 30.0,
                    ObstacleType::Flag => 25.0,
                    ObstacleType::Yeti => 25.0,
                    ObstacleType::Other(_) => 20.0,
                };

                if horizontal_distance < collision_threshold && distance_to_obstacle < 20.0 {
                    handle_collision(&mut skier, obstacle.obstacle_type, skier_pos.distance);
                }
            }
        }
    }
}

/// Handle what happens when skier hits an obstacle
fn handle_collision(skier: &mut Skier, obstacle_type: ObstacleType, _distance: f32) {
    match obstacle_type {
        ObstacleType::Tree | ObstacleType::Rock => {
            // Crash
            skier.is_crashed = true;
            skier.speed *= 0.1;
            // TODO: Play crash sound
            // TODO: Trigger crash animation
            info!("Skier crashed into {:?}!", obstacle_type);
        }
        ObstacleType::Jump => {
            // Trigger jump
            skier.is_jumping = true;
            // TODO: Play jump sound
            info!("Skier hit a jump!");
        }
        ObstacleType::Flag => {
            // Collect flag
            // TODO: Increment score
            // TODO: Play collection sound
            info!("Flag collected!");
        }
        ObstacleType::Yeti => {
            // Yeti encounter
            skier.is_crashed = true;
            skier.speed = 0.0;
            // TODO: Play Yeti sound
            info!("Eaten by the Yeti!");
        }
        ObstacleType::Other(_) => {
            // Generic obstacle collision
            skier.speed *= 0.7;
        }
    }
}

/// Check if skier just passed an object (for scoring, gates, etc.)
pub fn check_passed_objects(
    skier_query: Query<&CoursePosition, With<Skier>>,
    obstacle_query: Query<&Obstacle>,
) {
    for _skier_pos in skier_query.iter() {
        for _obstacle in obstacle_query.iter() {
            // Check if we just passed this obstacle
            // This is used for gates and scoring zones

            // TODO: Track which objects have been passed
            // TODO: Award points for passing gates
        }
    }
}
