# BevySki

A modern, cross-platform rewrite of the classic Macintosh game **MacSki v1.7** using Rust and the Bevy game engine.

## About

MacSki was a beloved skiing game for Mac OS Classic (PowerPC, mid-1990s) created by David Rowbotham. This project is a faithful recreation reverse-engineered from the original PowerPC executable, bringing this classic game to modern platforms while preserving its original gameplay mechanics.

**This is free, open-source software.** All registration/serial number systems from the original have been removed.

## Reverse Engineering Notes

This project was created by analyzing the MacSki v1.7 PEF (Preferred Executable Format) PowerPC binary in Ghidra with Metrowerks CodeWarrior 8 Mac API/CarbonLib type libraries applied. The original game used the low-level QuickDraw graphics API, which predated OpenGL.

### Original Game Architecture (from reverse engineering)

- **Main game loop**: Classic Mac event-driven architecture
- **Physics system**: Custom skiing physics with speed (0-1000+), angle (10-130°), and animation states
- **Collision detection**: Rectangle-based collision system for obstacles
- **Course system**: Binary course file format with procedural generation
- **Graphics**: QuickDraw-based rendering with sprite animation
- **Sound**: Mac Toolbox sound resources
- **Editor**: Built-in course editor with drag-and-drop object placement

### Key Functions Translated

- `adjust_skier()` → Movement and physics system
- `CheckForCollisions()` → Collision detection
- `DoSkiing()` → Main game update loop
- `DrawSlope()` → Rendering system
- `RandomCourse()` → Procedural course generation
- `human_controls()` → Input handling

## Features

### Implemented
- ✅ Basic Bevy project structure
- ✅ Core game entities (Skier, Obstacles, Course)
- ✅ Physics and movement system based on original
- ✅ Collision detection framework
- ✅ Course data structures
- ✅ Random course generation

### In Progress
- 🚧 Rendering system (sprites, trails)
- 🚧 Complete collision responses
- 🚧 Jump mechanics
- 🚧 Animation system

### Planned
- ⬜ Sound effects
- ⬜ Main menu and UI
- ⬜ Course editor
- ⬜ Score tracking and leaderboards
- ⬜ Multiple game modes
- ⬜ Course file loading/saving (JSON format)
- ⬜ Classic MacSki obstacle types (trees, rocks, jumps, flags)
- ⬜ Weather effects (wind/snow)

## Building and Running

### Prerequisites
- Rust 1.70+ 
- Cargo

### Build
```bash
cargo build --release
```

### Run
```bash
cargo run
```

## Controls

- **Arrow Keys / WASD**: Steer the skier
- **Space**: Jump
- **Down Arrow / S**: Brake
- **ESC**: Quit

## Game Mechanics

### Skiing Physics
The skiing physics are based on the original MacSki implementation:

- **Speed**: Increases automatically going downhill, max ~1000 units
- **Angle**: 10° (hard left) to 130° (hard right), 70° is straight down
- **Optimal angle**: 60-80° for maximum acceleration
- **Extreme angles**: <20° or >130° cause significant slowdown
- **Edge collision**: Hitting course boundaries reduces speed

### Obstacles
Based on the original obstacle types:
- **Trees**: Instant crash
- **Rocks**: Instant crash  
- **Jumps**: Launch the skier into the air
- **Flags**: Collect for points

## Development

### Workspace Structure

This project uses a Cargo workspace with multiple crates:

```
BevySki/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── bevyski/           # Main game executable
│   │   ├── src/
│   │   │   ├── main.rs           # Entry point and game setup
│   │   │   ├── components.rs     # ECS components (Skier, Obstacle, etc.)
│   │   │   ├── systems/
│   │   │   │   ├── movement.rs   # Physics and input
│   │   │   │   ├── collision.rs  # Collision detection
│   │   │   │   ├── rendering.rs  # Graphics
│   │   │   │   └── course.rs     # Course generation/loading
│   │   │   ├── resources.rs      # Global resources (Course, Settings)
│   │   │   ├── constants.rs      # Game constants from original
│   │   │   └── states.rs         # Game state machine
│   │   └── Cargo.toml
│   └── resfork/           # Resource fork parser library
│       ├── src/
│       │   ├── lib.rs            # Core parser
│       │   └── types.rs          # MacSki resource types
│       ├── examples/
│       │   └── list_resources.rs # Example usage
│       ├── README.md
│       └── Cargo.toml
├── courses/               # Course data files
├── README.md
├── REVERSE_ENGINEERING.md
└── ROADMAP.md
```

### Building

```bash
# Build entire workspace
cargo build

# Build release version
cargo build --release

# Run the game
cargo run -p bevyski

# Run resource fork examples
cargo run -p resfork --example list_resources
```

### Technology Stack

- **Rust Edition**: 2024
- **Game Engine**: Bevy 0.18
- **Resource Parser**: Custom resfork crate for classic Mac resource files

### Contributing
Contributions are welcome! This project aims to be a faithful recreation while modernizing for cross-platform play.

## Credits

- **Original MacSki**: David Rowbotham (mid-1990s)
- **BevySki**: Reverse engineered and reimplemented by the open-source community
- **Engine**: Built with [Bevy](https://bevyengine.org/) game engine

## License

MIT License - See LICENSE file

**Note**: This is a clean-room reverse engineering project. No original game assets or copyrighted materials are included. All code was written by analyzing the compiled binary behavior and reimplementing the logic in Rust.

## Screenshots

*Coming soon!*

## Technical Notes

### MacSki Original Specifications
- Platform: PowerPC Macintosh (Mac OS Classic)
- Compiler: Metrowerks CodeWarrior 8
- Graphics API: QuickDraw
- Architecture: PEF (Preferred Executable Format)
- Functions: 1039 total, ~176KB code size
- Course Format: Custom binary format

### BevySki Target
- Platform: Cross-platform (Windows, macOS, Linux, potentially Web)
- Language: Rust
- Graphics: Bevy (WebGPU-based)
- Course Format: JSON for editability
