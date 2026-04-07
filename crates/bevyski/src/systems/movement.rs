//! Skier movement and physics system.

use crate::components::{CoursePosition, Skier};
use crate::constants::*;
use bevy::prelude::*;

/// Processes keyboard input for skier control.
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

/// Calculates skier velocity vector from speed and angle.
///
/// Angle 70 is straight down, smaller angles go left, larger go right.
const fn skier_vector(speed: f32, angle: i16) -> Vec2 {
    let normalized_angle = angle - CENTER_ANGLE;
    let side_speed = (normalized_angle as f32 * speed) / 100.0;
    let forward_speed = speed;

    Vec2::new(side_speed, forward_speed)
}

/// Calculates angle efficiency multiplier for acceleration.
const fn calculate_angle_efficiency(angle: i16) -> f32 {
    if angle < 20 || angle > 130 {
        0.5 // Inefficient angle, lose speed
    } else if angle > 60 && angle < 80 {
        1.2 // Optimal downhill angle, gain speed
    } else {
        1.0
    }
}

/// Applies auto-centering to skier angle.
fn apply_auto_center(angle: &mut i16) {
    if *angle < CENTER_ANGLE - 2 {
        *angle += 1;
    } else if *angle > CENTER_ANGLE + 2 {
        *angle -= 1;
    }
}

/// Enforces course boundaries on skier position and speed.
fn enforce_course_bounds(pos: &mut CoursePosition, speed: &mut f32) {
    if pos.x < 10.0 {
        pos.x = 10.0;
        *speed *= 0.8; // Hit edge, slow down
    } else if pos.x > COURSE_WIDTH - 10.0 {
        pos.x = COURSE_WIDTH - 10.0;
        *speed *= 0.8;
    }
}

/// Main skier physics update.
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
        let angle_efficiency = calculate_angle_efficiency(skier.angle);

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
        enforce_course_bounds(&mut pos, &mut skier.speed);

        // Update animation frame based on speed and state
        if !skier.is_jumping && !skier.is_crashed {
            update_skiing_animation(&mut skier, delta);
        }

        // Auto-center tendency
        apply_auto_center(&mut skier.angle);
    }
}

/// Updates skiing animation frame based on speed.
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

/// Applies wind effects to skier.
///
/// # Note
///
/// This is a placeholder for future wind/weather system implementation.
#[allow(dead_code)]
pub fn apply_wind(
    _query: Query<(&mut Skier, &mut CoursePosition)>,
    // TODO: Add Wind resource when weather system is implemented
) {
    // Wind affects both horizontal position and speed
    // TODO: Implement wind effects
}
