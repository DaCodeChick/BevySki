# Development Roadmap

This document outlines the development plan for BevySki, organized by priority and dependencies.

## Phase 1: Core Foundation ✅ COMPLETE

- [x] Project structure setup
- [x] Basic Bevy application
- [x] Component definitions
- [x] Physics system framework
- [x] Collision detection framework
- [x] Course data structures
- [x] Random course generation
- [x] Documentation

## Phase 2: Visual Rendering 🚧 IN PROGRESS

### High Priority
- [ ] Create simple placeholder sprites
  - Rectangle for skier
  - Circles/shapes for obstacles
- [ ] Implement sprite rendering system
- [ ] Camera system (scrolling)
- [ ] Background rendering (snow/slope)
- [ ] Ski trail rendering (lines behind skier)

### Technical Notes
- Start with simple shapes before adding detailed sprites
- Use Bevy's 2D sprite system
- Reference original QuickDraw rendering in `DrawSlope()` @ 0x100140CC

## Phase 3: Gameplay Mechanics

### High Priority
- [ ] Complete jump mechanics (animation + physics)
- [ ] Crash detection and recovery
- [ ] Speed display / HUD
- [ ] Distance counter
- [ ] Basic sound effects (placeholder beeps)

### Medium Priority
- [ ] Different obstacle behaviors
  - Trees (crash)
  - Rocks (crash)
  - Jumps (launch)
  - Flags (collect)
- [ ] Trick system during jumps
- [ ] Wind/weather effects

### Reference Functions
- `adjust_skier()` @ 0x10012C60 - Jump logic around line 200-300
- `skier_fall_down()` @ 0x100156CC - Crash mechanics
- `DoObjectCollision()` @ 0x10001C84 - Obstacle responses

## Phase 4: User Interface

### High Priority
- [ ] Main menu
  - New Game
  - Load Course
  - Settings
  - Quit
- [ ] Pause menu
- [ ] Game over screen with stats
- [ ] Settings menu
  - Sound on/off
  - Controls configuration

### Medium Priority
- [ ] Course selection screen
- [ ] High score table
- [ ] Tutorial/How to Play

### Reference Functions
- `SetUpMacSkiMenus()` @ 0x1000D3DC
- `MacSkiInfoDialog()` @ 0x10010E04

## Phase 5: Course System

### High Priority
- [ ] Course loading from JSON
- [ ] Course saving
- [ ] Multiple built-in courses
  - Beginner
  - Intermediate
  - Advanced
  - Expert

### Medium Priority
- [ ] Course editor UI
  - Object palette
  - Drag-and-drop placement
  - Terrain editing
  - Preview mode
- [ ] Course validation
- [ ] Course sharing (export/import)

### Reference Functions
- `SetUpCourseEditor()` @ 0x10004020
- `DrawEditor()` @ 0x100042F4
- `ReadCourse()` @ 0x10002D44
- `WriteCourse()` @ 0x100031E0

## Phase 6: Audio

### High Priority
- [ ] Sound effect system
- [ ] Basic skiing sounds
- [ ] Crash sounds
- [ ] Jump sounds

### Medium Priority
- [ ] Background music
- [ ] Voice samples (if recreating original)
- [ ] Volume controls

### Reference
- `MacSkiPlaySoundResource()` @ 0x10016438
- Sound IDs: 0x450-0x458 (skiing), 0x44C-0x44F (jumps), etc.

## Phase 7: Polish

### High Priority
- [ ] Particle effects (snow spray)
- [ ] Better animations
- [ ] Improved graphics
- [ ] Performance optimization

### Medium Priority
- [ ] Achievements
- [ ] Statistics tracking
- [ ] Replay system
- [ ] Multiple skier characters

## Phase 8: Advanced Features

### Medium Priority
- [ ] Multiplayer (local)
- [ ] Multiplayer (online)
- [ ] Course challenges/objectives
- [ ] Time trials
- [ ] Leaderboards

### Low Priority
- [ ] Web version (WASM)
- [ ] Mobile ports
- [ ] Mod support
- [ ] Level packs

## Known Issues / TODOs

From current codebase:

1. **main.rs:79** - Obstacle spawning needs proper system integration
2. **movement.rs** - Static mut for animation timer (replace with proper resource)
3. **rendering.rs** - No actual sprite rendering yet (just transform updates)
4. **collision.rs** - Need to track which objects have been passed
5. **course.rs** - load_course/save_course not integrated into main app

## Testing Priorities

1. Physics feel - Does it feel like skiing?
2. Collision accuracy - Fair and predictable
3. Course generation - Fun and varied
4. Performance - 60 FPS on moderate hardware
5. Cross-platform - Works on Windows, macOS, Linux

## Success Metrics

- [ ] Can play a full course from start to finish
- [ ] Crashes feel fair and recovery is smooth
- [ ] Course editor is intuitive
- [ ] Game runs at 60 FPS
- [ ] Faithful to original MacSki feel
- [ ] New players can pick it up easily

## Community Contributions

Looking for help with:
- **Artists**: Ski sprites, obstacle sprites, backgrounds
- **Sound designers**: Sound effects, music
- **Level designers**: Creating interesting courses
- **Testers**: Cross-platform testing, bug reports
- **Developers**: See CONTRIBUTING.md

## Long-term Vision

Make BevySki the definitive modern version of MacSki:
- Preserve the classic gameplay
- Add quality-of-life improvements
- Support modding community
- Regular content updates
- Active community

## References

- Original MacSki reverse engineering: `REVERSE_ENGINEERING.md`
- Contributing guidelines: `CONTRIBUTING.md`
- README: `README.md`
