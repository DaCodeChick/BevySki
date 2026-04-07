// Skier movement and physics system

use crate::components::{CoursePosition, Skier};
use crate::constants::*;
use bevy::prelude::*;

/// Handle keyboard input for skier control
pub fn skier_input(keyboard: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Skier>) {
    for mut skier in query.iter_mut() {
        if skier.is_crashed {
            continue;
        }

        // Left/Right arrow keys or A/D to turn
        let mut turn_input = 0;
        if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
            turn_input -= 1;
        }
        if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
            turn_input += 1;
        }

        // Adjust angle based on input
        if turn_input != 0 {
            skier.angle = (skier.angle + turn_input * 2).clamp(MIN_SKIER_ANGLE, MAX_SKIER_ANGLE);
        }

        // Space for jump (if not already jumping)
        if keyboard.just_pressed(KeyCode::Space) && !skier.is_jumping {
            skier.is_jumping = true;
        }

        // Brake (slow down)
        if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
            skier.speed = (skier.speed - DECELERATION_RATE).max(0.0);
        }
    }
}

/// Calculate skier velocity vector from angle
fn skier_vector(speed: f32, angle: i16) -> Vec2 {
    // Angle 70 is straight down, smaller angles go left, larger go right
    // We'll convert this to a standard coordinate system
    let normalized_angle = angle - CENTER_ANGLE;
    let side_speed = (normalized_angle as f32 * speed) / 100.0;
    let forward_speed = speed;

    Vec2::new(side_speed, forward_speed)
}

/// Main skier physics update
pub fn adjust_skier(time: Res<Time>, mut query: Query<(&mut Skier, &mut CoursePosition)>) {
    let delta = time.delta_secs();

    for (mut skier, mut pos) in query.iter_mut() {
        // Handle jumping state
        if skier.is_jumping {
            // Jumps reduce speed temporarily
            skier.speed *= 0.9;
            // TODO: Full jump animation sequence
        }

        // Calculate acceleration based on angle
        // Skiing at extreme angles causes slowdown
        let angle_efficiency = if skier.angle < 20 || skier.angle > 130 {
            0.5 // Inefficient angle, lose speed
        } else if skier.angle > 60 && skier.angle < 80 {
            1.2 // Optimal downhill angle, gain speed
        } else {
            1.0
        };

        // Apply gravity/slope acceleration
        skier.speed += ACCELERATION_RATE * angle_efficiency * delta * 60.0;

        // Cap maximum speed
        skier.speed = skier.speed.min(MAX_SPEED);

        // Calculate movement vector
        let velocity = skier_vector(skier.speed, skier.angle);

        // Update position
        pos.x += velocity.x * delta * 0.1;
        pos.distance += velocity.y * delta * 0.1;

        // Keep skier within course bounds
        if pos.x < 10.0 {
            pos.x = 10.0;
            skier.speed *= 0.8; // Hit edge, slow down
        } else if pos.x > COURSE_WIDTH - 10.0 {
            pos.x = COURSE_WIDTH - 10.0;
            skier.speed *= 0.8;
        }

        // Update animation frame based on speed and state
        if !skier.is_jumping && !skier.is_crashed {
            update_skiing_animation(&mut skier, delta);
        }

        // Auto-center tendency
        if skier.angle < CENTER_ANGLE - 2 {
            skier.angle += 1;
        } else if skier.angle > CENTER_ANGLE + 2 {
            skier.angle -= 1;
        }
    }
}

/// Update skiing animation frame
fn update_skiing_animation(skier: &mut Skier, delta: f32) {
    // Animation speed increases with skier speed
    let anim_speed = (skier.speed / 100.0).min(10.0);

    // Cycle through animation frames (0-5 for different skiing poses)
    static mut ANIM_TIMER: f32 = 0.0;
    unsafe {
        ANIM_TIMER += delta * anim_speed;
        if ANIM_TIMER > ANIMATION_SPEED {
            ANIM_TIMER = 0.0;
            skier.animation_frame = (skier.animation_frame + 1) % 6;
        }
    }
}

/// Apply wind effects to skier
#[allow(dead_code)]
pub fn apply_wind(
    _query: Query<(&mut Skier, &mut CoursePosition)>,
    // TODO: Add Wind resource when weather system is implemented
) {
    // Wind affects both horizontal position and speed
    // TODO: Implement wind effects
}
