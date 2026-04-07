// BevySki - A modern rewrite of the classic MacSki game
// Original MacSki by David Rowbotham (mid-1990s)

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
use resources::{Course, GameSettings};
use rfd::FileDialog;
use snd::Sound as MacSound;
use states::GameState;
use systems::GameSystemsPlugin;

const EXTRACTED_ASSETS_DIR: &str = "assets/extracted";

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
        .add_systems(Startup, setup)
        .add_systems(
            OnEnter(GameState::Playing),
            (start_game, systems::course::spawn_course_obstacles).chain(),
        )
        .run();
}

fn setup(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    ensure_extracted_assets();

    // Spawn 2D camera
    commands.spawn(Camera2d);

    info!("BevySki started - Modern rewrite of MacSki v1.7");
    info!("Original game by David Rowbotham");

    // Start in playing state for now (TODO: add main menu)
    next_state.set(GameState::Playing);
}

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

fn prompt_for_resource_directory() -> Option<std::path::PathBuf> {
    FileDialog::new()
        .set_title("Select folder containing MacSki .rsrc files")
        .pick_folder()
}

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

fn start_game(mut commands: Commands) {
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

    // Start game timer
    commands.spawn(GameTimer { elapsed: 0.0 });
}
