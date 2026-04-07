//! BevySki - A modern rewrite of the classic MacSki game.
//!
//! Original MacSki by David Rowbotham (mid-1990s).
//!
//! This is the main entry point for the BevySki application. It handles:
//! - Game initialization and Bevy setup
//! - Asset extraction and conversion from original MacSki resource files
//! - Primary game loop configuration

mod components;
mod constants;
mod resources;
mod states;
mod systems;

use bevy::prelude::*;
use binrw::BinReaderExt;
use components::{CoursePosition, GameTimer, Score, Skier};
use fourcc::fourcc;
use hound::{SampleFormat, WavSpec, WavWriter};
use pict_resources::extract_pict_resources_to_png;
use resource_fork::ResourceFork;
use resources::{Course, GameSettings, LastRunSummary, RunLifecycle, RunOutcome};
use rfd::FileDialog;
use snd::Sound as MacSound;
use states::GameState;
use systems::GameSystemsPlugin;

/// Directory where extracted and converted assets are stored.
const EXTRACTED_ASSETS_DIR: &str = "assets/extracted";

/// Main entry point for BevySki.
///
/// Sets up the Bevy app with default plugins, custom game systems,
/// and initializes game state management.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "BevySki - Modern rewrite of MacSki v1.7".to_string(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GameSystemsPlugin)
        .init_state::<GameState>()
        .init_resource::<GameSettings>()
        .init_resource::<RunLifecycle>()
        .init_resource::<LastRunSummary>()
        .add_systems(Startup, setup)
        .add_systems(
            OnEnter(GameState::Playing),
            (start_game, systems::course::spawn_course_obstacles).chain(),
        )
        .run();
}

/// Initial setup system that runs once at game startup.
///
/// Ensures assets are extracted, spawns the camera, and transitions
/// to the Ski Lodge state to match the original game flow.
fn setup(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    ensure_extracted_assets();

    // Spawn 2D camera
    commands.spawn(Camera2d);

    info!("BevySki started - Modern rewrite of MacSki v1.7");
    info!("Original game by David Rowbotham");

    // Start in Ski Lodge (hub/menu) to match original flow.
    next_state.set(GameState::SkiLodge);
}

/// Ensures extracted assets exist or prompts user to extract them.
///
/// If assets are already present in the extracted directory, does nothing.
/// Otherwise, prompts the user to select the folder containing original
/// MacSki `.rsrc` files and performs extraction/conversion.
fn ensure_extracted_assets() {
    let extracted_dir = std::path::Path::new(EXTRACTED_ASSETS_DIR);
    if extracted_assets_present(extracted_dir) {
        info!("Using existing extracted assets at {EXTRACTED_ASSETS_DIR}");
        return;
    }

    let Some(rsrc_path) = prompt_for_resource_directory() else {
        warn!("Resource extraction cancelled: no source directory selected.");
        return;
    };

    let sources = [
        rsrc_path.join("MacSki Color Art.rsrc"),
        rsrc_path.join("MacSki Sounds.rsrc"),
    ];

    if !sources.iter().all(|path| path.exists()) {
        warn!("Resource sources not found in {}.", rsrc_path.display());
        return;
    }

    if let Err(error) = std::fs::create_dir_all(extracted_dir) {
        warn!("Failed to create extracted assets directory: {error}");
        return;
    }

    if let Err(error) = extract_and_convert_with_resource_fork(&rsrc_path, extracted_dir) {
        warn!("Failed to extract resources: {error}");
    }
}

/// Prompts user to select the directory containing MacSki resource files.
///
/// Uses a native file dialog to let the user browse for the folder
/// containing `.rsrc` files from the original MacSki game.
///
/// # Returns
///
/// `Some(PathBuf)` if user selected a folder, `None` if cancelled.
fn prompt_for_resource_directory() -> Option<std::path::PathBuf> {
    FileDialog::new()
        .set_title("Select folder containing MacSki .rsrc files")
        .pick_folder()
}

/// Checks if extracted assets are already present in the given directory.
///
/// Verifies that both PNG (art) and WAV (sound) files exist in the directory.
///
/// # Returns
///
/// `true` if at least one PNG and one WAV file are found, `false` otherwise.
fn extracted_assets_present(dir: &std::path::Path) -> bool {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return false,
    };

    let mut has_png = false;
    let mut has_wav = false;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "png") {
            has_png = true;
        }
        if path.extension().is_some_and(|ext| ext == "wav") {
            has_wav = true;
        }
        if has_png && has_wav {
            return true;
        }
    }

    false
}

/// Extracts and converts resources from MacSki `.rsrc` files.
///
/// Reads PICT and sound resources from the original MacSki resource files
/// and converts them to modern formats (PNG and WAV).
///
/// # Arguments
///
/// * `rsrc_path` - Directory containing MacSki `.rsrc` files
/// * `output_dir` - Directory where converted assets will be written
///
/// # Errors
///
/// Returns an error if resource files cannot be read or assets cannot be converted.
fn extract_and_convert_with_resource_fork(
    rsrc_path: &std::path::Path,
    output_dir: &std::path::Path,
) -> Result<(), String> {
    let art_path = rsrc_path.join("MacSki Color Art.rsrc");
    let sounds_path = rsrc_path.join("MacSki Sounds.rsrc");

    extract_raw_type_resources(&art_path, output_dir, fourcc!("PICT"), "pict")?;
    let _ = extract_pict_resources_to_png(&art_path, output_dir).map_err(|e| e.to_string())?;
    extract_sound_resources_to_wav(&sounds_path, output_dir)?;
    Ok(())
}

/// Extracts raw resources of a specific type from a resource fork file.
///
/// Reads all resources of the given type (e.g., PICT) from the source file
/// and writes them as raw binary files with the specified extension.
///
/// # Arguments
///
/// * `source_path` - Path to the `.rsrc` file
/// * `output_dir` - Directory where resources will be written
/// * `resource_type` - FourCC code of the resource type to extract
/// * `extension` - File extension to use for extracted files
///
/// # Errors
///
/// Returns an error if the resource fork cannot be opened or files cannot be written.
fn extract_raw_type_resources(
    source_path: &std::path::Path,
    output_dir: &std::path::Path,
    resource_type: fourcc::FourCC,
    extension: &str,
) -> Result<(), String> {
    let mut fork = ResourceFork::open(source_path).map_err(|e| e.to_string())?;
    let ids = fork
        .list_resources(resource_type)
        .map_err(|e| e.to_string())?;

    for id in ids {
        let (data, _) = fork
            .read_data(resource_type, id)
            .map_err(|e| e.to_string())?;
        let out = output_dir.join(format!("{}_{}.{}", resource_type.name(), id, extension));
        std::fs::write(out, data).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Extracts sound resources and converts them to WAV format.
///
/// Reads classic Mac `snd ` resources from the source file, parses them,
/// and writes them out as standard WAV audio files.
///
/// # Arguments
///
/// * `source_path` - Path to the `.rsrc` file containing sounds
/// * `output_dir` - Directory where WAV files will be written
///
/// # Errors
///
/// Returns an error if the resource fork cannot be opened, sounds cannot
/// be parsed, or WAV files cannot be written.
fn extract_sound_resources_to_wav(
    source_path: &std::path::Path,
    output_dir: &std::path::Path,
) -> Result<(), String> {
    let mut fork = ResourceFork::open(source_path).map_err(|e| e.to_string())?;
    let snd_type = fourcc!("snd ");
    let ids = fork.list_resources(snd_type).map_err(|e| e.to_string())?;

    for id in ids {
        let (data, _) = fork.read_data(snd_type, id).map_err(|e| e.to_string())?;
        let mut cursor = std::io::Cursor::new(&data);
        let sound: MacSound = match cursor.read_be() {
            Ok(sound) => sound,
            Err(_) => continue,
        };

        let sample_rate: u32 = match sound.sample_rate.try_into() {
            Ok(rate) => rate,
            Err(_) => continue,
        };

        let spec = WavSpec {
            channels: sound.channels,
            sample_rate,
            bits_per_sample: sound.bits_per_sample,
            sample_format: SampleFormat::Int,
        };

        let output = output_dir.join(format!("snd_{}.wav", id));
        let mut writer = WavWriter::create(output, spec).map_err(|e| e.to_string())?;
        for sample in sound.samples {
            writer.write_sample(sample).map_err(|e| e.to_string())?;
        }
        writer.finalize().map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// System that starts a new game run.
///
/// Creates a random course, spawns the skier entity with initial components,
/// and starts the game timer. Runs when entering the `Playing` state.
fn start_game(
    mut commands: Commands,
    mut lifecycle: ResMut<RunLifecycle>,
    mut summary: ResMut<LastRunSummary>,
) {
    info!("Starting new game...");

    // Generate a random course (or load default)
    let course = Course::random();
    info!(
        "Course: {} - {} obstacles",
        course.name,
        course.obstacles.len()
    );

    // Spawn the skier with a simple colored square
    commands.spawn((
        Skier::default(),
        CoursePosition::default(),
        Score::default(),
        Sprite {
            color: Color::srgb(0.2, 0.6, 1.0), // Blue skier
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -100.0, 10.0),
    ));

    // Insert course resource (obstacles will be spawned by course system)
    commands.insert_resource(course);

    lifecycle.elapsed_seconds = 0.0;
    lifecycle.crashed_seconds = 0.0;
    lifecycle.crashes_since_dog = 0;
    lifecycle.dog_rescue_active = false;
    summary.outcome = RunOutcome::InProgress;
    summary.distance = 0.0;
    summary.time = 0.0;
    summary.crashes = 0;
    summary.flags_collected = 0;
    summary.jumps_performed = 0;

    // Start game timer
    commands.spawn(GameTimer { elapsed: 0.0 });
}
