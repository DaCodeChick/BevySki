//! Run lifecycle systems for start, finish, crash recovery, and cleanup.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::{CoursePosition, Obstacle, Score, Skier};
use crate::resources::{Course, LastRunSummary, RunLifecycle, RunOutcome};
use crate::states::GameState;

const CRASH_RECOVERY_SECONDS: f32 = 1.5;
const MAX_CRASH_DISTANCE_PENALTY: f32 = 75.0;
const DOG_ANCHOR_FRACTION_X: f32 = 0.5;
const DOG_RESCUE_SETBACK_MULTIPLIER: f32 = 80.0;
const DOG_SPRITE_ID_BASE: u16 = 400;
const DOG_SPRITE_LUT: [u16; 40] = [
    0, 1, 2, 1, 3, 4, 5, 8, 12, 13, 12, 13, 14, 15, 16, 13, 12, 13, 12, 5, 4, 3, 1, 2, 1, 0, 1, 3,
    4, 5, 6, 7, 5, 4, 3, 1, 9, 10, 11, 0,
];

/// Sprite handle lookup table keyed by original MacSki PICT id.
#[derive(Resource, Default)]
pub struct ExtractedSpriteAtlas {
    /// Mapping from PICT id to loaded image handle.
    pub by_id: HashMap<u16, Handle<Image>>,
}

/// Marker for the temporary rescue dog animation entity.
#[derive(Component)]
pub struct RescueDog {
    /// Current dog animation phase, mirroring MacSki's `gDogAnimFrame`.
    anim_frame: u32,
    /// Current dog pose frame, mirroring MacSki's `gDogPoseFrame`.
    pose_frame: u32,
    /// Skier X offset used to anchor dog near skier before rescue pull.
    skier_offset_x: f32,
    /// Vertical screen Y for the dog sequence.
    screen_y: f32,
    /// World distance where dog starts pulling skier forward.
    rescue_target_distance: f32,
}

fn current_dog_sprite_id(dog: &RescueDog) -> u16 {
    let frame_index = if dog.pose_frame != 0 {
        dog.pose_frame
    } else {
        dog.anim_frame
    };
    let lut_index = (frame_index as usize).min(DOG_SPRITE_LUT.len() - 1);
    DOG_SPRITE_ID_BASE + DOG_SPRITE_LUT[lut_index]
}

fn parse_pict_id_from_stem(stem: &str) -> Option<u16> {
    let rest = stem
        .strip_prefix("PICT_")
        .or_else(|| stem.strip_prefix("pict_"))?;
    rest.parse::<u16>().ok()
}

/// Rebuilds the extracted sprite lookup from `assets/extracted`.
pub fn refresh_extracted_sprite_atlas(
    mut atlas: ResMut<ExtractedSpriteAtlas>,
    asset_server: Res<AssetServer>,
) {
    atlas.by_id.clear();

    let Ok(entries) = std::fs::read_dir("assets/extracted") else {
        warn!("No extracted assets directory found for sprite atlas.");
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
        {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(id) = parse_pict_id_from_stem(stem) else {
            continue;
        };
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        let handle: Handle<Image> = asset_server.load(format!("extracted/{name}"));
        atlas.by_id.insert(id, handle);
    }

    info!("Loaded {} extracted sprite handles.", atlas.by_id.len());
}

/// Marker for the run summary UI root node.
#[derive(Component)]
pub struct RunSummaryRoot;

/// Marker for in-run HUD text widget.
#[derive(Component)]
pub struct RunHudText;

/// Advances run timers and mirrors core values into the score component.
pub fn update_run_stats(
    time: Res<Time>,
    mut lifecycle: ResMut<RunLifecycle>,
    mut query: Query<(&CoursePosition, &mut Score), With<Skier>>,
) {
    let delta = time.delta_secs();
    lifecycle.elapsed_seconds += delta;

    for (position, mut score) in &mut query {
        score.distance = position.distance;
        score.time = lifecycle.elapsed_seconds;
    }
}

/// Spawns in-run timer/HUD overlay.
pub fn spawn_run_hud(mut commands: Commands) {
    commands.spawn((
        RunHudText,
        Text::new("Time 0.0  Dist 0"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.96, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: px(14.0),
            left: px(16.0),
            ..default()
        },
    ));
}

/// Updates in-run timer/HUD text.
pub fn update_run_hud(
    lifecycle: Res<RunLifecycle>,
    skier_query: Query<(&CoursePosition, &Score), With<Skier>>,
    mut hud_query: Query<&mut Text, With<RunHudText>>,
) {
    let Ok((position, score)) = skier_query.single() else {
        return;
    };
    let Ok(mut text) = hud_query.single_mut() else {
        return;
    };

    *text = Text::new(format!(
        "Time {:.1}s  Dist {:.0}  Crashes {}  Flags {}",
        lifecycle.elapsed_seconds, position.distance, score.crashes, score.flags_collected
    ));
}

/// Handles crash recovery with MacSki-style dog rescue activation.
pub fn handle_crash_recovery(
    mut commands: Commands,
    time: Res<Time>,
    mut lifecycle: ResMut<RunLifecycle>,
    mut skiers: Query<(&mut Skier, &mut Score, &mut CoursePosition)>,
) {
    let delta = time.delta_secs();

    for (mut skier, mut score, mut position) in &mut skiers {
        if !skier.is_crashed {
            lifecycle.crashed_seconds = 0.0;
            continue;
        }

        lifecycle.crashed_seconds += delta;

        if lifecycle.crashed_seconds >= CRASH_RECOVERY_SECONDS {
            let setback = (skier.speed * 0.25).min(MAX_CRASH_DISTANCE_PENALTY);
            skier.is_crashed = false;
            skier.is_jumping = false;
            skier.speed = 120.0;
            score.crashes += 1;
            position.distance = (position.distance - setback).max(0.0);

            if !lifecycle.dog_rescue_active {
                lifecycle.crashes_since_dog = lifecycle.crashes_since_dog.saturating_add(1);
            }

            if lifecycle.crashes_since_dog >= 5 && !lifecycle.dog_rescue_active {
                lifecycle.dog_rescue_active = true;
                let skier_offset_x = match skier.angle {
                    ..=59 => 32.0,
                    60..=79 => 22.0,
                    _ => 50.0,
                };
                let screen_y = match skier.angle {
                    ..=59 => 17.0,
                    60..=79 => 20.0,
                    _ => 8.0,
                };

                commands.spawn((
                    RescueDog {
                        anim_frame: 0,
                        pose_frame: 0,
                        skier_offset_x,
                        screen_y,
                        rescue_target_distance: position.distance
                            + lifecycle.crashes_since_dog as f32 * DOG_RESCUE_SETBACK_MULTIPLIER,
                    },
                    Sprite {
                        color: Color::srgb(0.63, 0.42, 0.24),
                        custom_size: Some(Vec2::new(28.0, 18.0)),
                        ..default()
                    },
                    Transform::from_xyz(-55.0, -170.0, 18.0),
                ));
            }

            lifecycle.crashed_seconds = 0.0;
        }
    }
}

/// Runs a lightweight approximation of MacSki's dog rescue state machine.
pub fn animate_rescue_dog(
    mut commands: Commands,
    mut lifecycle: ResMut<RunLifecycle>,
    mut dogs: Query<(Entity, &mut RescueDog, &mut Transform, &mut Sprite)>,
    mut skier_query: Query<(&mut CoursePosition, &mut Skier), With<Skier>>,
    atlas: Res<ExtractedSpriteAtlas>,
) {
    let Ok((skier_position, mut skier)) = skier_query.single_mut() else {
        return;
    };

    for (entity, mut dog, mut transform, mut sprite) in &mut dogs {
        if dog.pose_frame == 0 {
            dog.anim_frame = dog.anim_frame.saturating_add(1);

            if dog.anim_frame < 5 {
                let anchor_x = skier_position.x * DOG_ANCHOR_FRACTION_X - dog.skier_offset_x;
                transform.translation.x = (transform.translation.x + 15.0).min(anchor_x);

                if transform.translation.x >= anchor_x {
                    dog.anim_frame = 4;
                }
            } else if dog.anim_frame == 0xf {
                if skier_position.distance < dog.rescue_target_distance {
                    dog.anim_frame = 0xd;
                }
            } else if dog.anim_frame > 0x15 {
                transform.translation.x += 20.0;
                if transform.translation.x > 520.0 {
                    lifecycle.dog_rescue_active = false;
                    lifecycle.crashes_since_dog = 0;
                    commands.entity(entity).despawn();
                    continue;
                }
                if dog.anim_frame > 0x19 {
                    dog.anim_frame = 0x16;
                }
            }
        }

        if dog.pose_frame == 0 && dog.anim_frame == 0xb {
            if skier.is_crashed {
                dog.pose_frame = 0x1a;
            }
        } else if dog.pose_frame != 0 {
            dog.pose_frame = dog.pose_frame.saturating_add(1);
            if dog.pose_frame == 0x25 {
                skier.is_crashed = false;
                skier.speed = skier.speed.max(90.0);
            }
            if dog.pose_frame == 0x24 || dog.pose_frame > 0x26 {
                dog.pose_frame = 0;
            }
        }

        transform.translation.y = -180.0 + dog.screen_y;

        let dog_sprite_id = current_dog_sprite_id(&dog);
        if let Some(image) = atlas.by_id.get(&dog_sprite_id) {
            sprite.image = image.clone();
            sprite.color = Color::WHITE;
            sprite.custom_size = None;
        } else {
            let tint_seed = (dog_sprite_id % 11) as f32 / 10.0;
            sprite.color = Color::srgb(0.45 + tint_seed * 0.2, 0.30 + tint_seed * 0.1, 0.18);
            sprite.custom_size = Some(Vec2::new(28.0, 18.0));
        }
    }
}

/// Completes the run when the skier reaches course length.
pub fn check_run_completion(
    course: Res<Course>,
    lifecycle: Res<RunLifecycle>,
    skier_query: Query<(&CoursePosition, &Score), With<Skier>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut summary: ResMut<LastRunSummary>,
) {
    let Ok((position, score)) = skier_query.single() else {
        return;
    };

    if position.distance < course.length {
        return;
    }

    summary.outcome = RunOutcome::Finished;
    summary.distance = position.distance;
    summary.time = lifecycle.elapsed_seconds;
    summary.crashes = score.crashes;
    summary.flags_collected = score.flags_collected;
    summary.jumps_performed = score.jumps_performed;
    next_state.set(GameState::GameOver);
}

/// Returns to Ski Lodge during gameplay when Escape is pressed.
pub fn return_to_ski_lodge_shortcut(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut summary: ResMut<LastRunSummary>,
    lifecycle: Res<RunLifecycle>,
    skier_query: Query<(&CoursePosition, &Score), With<Skier>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    let Ok((position, score)) = skier_query.single() else {
        next_state.set(GameState::SkiLodge);
        return;
    };

    summary.outcome = RunOutcome::Aborted;
    summary.distance = position.distance;
    summary.time = lifecycle.elapsed_seconds;
    summary.crashes = score.crashes;
    summary.flags_collected = score.flags_collected;
    summary.jumps_performed = score.jumps_performed;
    next_state.set(GameState::SkiLodge);
}

/// Despawns gameplay entities when leaving the Playing state.
pub fn cleanup_playing_entities(
    mut commands: Commands,
    skiers: Query<Entity, With<Skier>>,
    obstacles: Query<Entity, With<Obstacle>>,
    timers: Query<Entity, With<crate::components::GameTimer>>,
    dogs: Query<Entity, With<RescueDog>>,
    huds: Query<Entity, With<RunHudText>>,
) {
    for entity in &skiers {
        commands.entity(entity).despawn();
    }
    for entity in &obstacles {
        commands.entity(entity).despawn();
    }
    for entity in &timers {
        commands.entity(entity).despawn();
    }
    for entity in &dogs {
        commands.entity(entity).despawn();
    }
    for entity in &huds {
        commands.entity(entity).despawn();
    }
}

/// Spawns a simple game-over summary UI.
pub fn spawn_game_over_ui(mut commands: Commands, summary: Res<LastRunSummary>) {
    let outcome_label = match summary.outcome {
        RunOutcome::InProgress => "Run in progress",
        RunOutcome::Finished => "Run Finished!",
        RunOutcome::Aborted => "Run Aborted",
    };

    commands
        .spawn((
            RunSummaryRoot,
            Node {
                width: percent(100.0),
                height: percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.08, 0.12, 0.92)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(outcome_label),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.98, 1.0)),
            ));
            parent.spawn((
                Text::new(format!(
                    "Distance {:.0} / Time {:.1}s / Crashes {} / Flags {} / Jumps {}",
                    summary.distance,
                    summary.time,
                    summary.crashes,
                    summary.flags_collected,
                    summary.jumps_performed
                )),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.9, 1.0)),
            ));
            parent.spawn((
                Text::new("Press Enter for Ski Lodge, R to retry"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.82, 0.9)),
            ));
        });
}

/// Handles input on the game-over summary screen.
pub fn handle_game_over_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::SkiLodge);
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Playing);
    }
}

/// Cleans up game-over UI when leaving the state.
pub fn cleanup_game_over_ui(mut commands: Commands, roots: Query<Entity, With<RunSummaryRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}
