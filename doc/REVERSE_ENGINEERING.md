# Reverse Engineering Notes - MacSki v1.7

This document contains detailed notes from reverse engineering the MacSki v1.7 PowerPC executable in Ghidra.

## Binary Information

- **Format**: PEF (Preferred Executable Format)
- **Architecture**: PowerPC 32-bit, big-endian
- **Compiler**: Metrowerks CodeWarrior 8
- **Address Range**: 0x10000000 - 0x1003935B
- **Total Functions**: 1039 (1036 named, 3 auto-generated)
- **Code Size**: 176,240 bytes
- **Average Function Size**: 169.6 bytes

## Key Data Structures

### Skier State (Global Variables)

From the decompiled code, the skier state appears to be stored in global variables:

- `SHORT_10037db2`: Skier horizontal position (course X)
- `_DAT_10035604`: Skier vertical distance down course
- `SHORT_10037dba`: Skier angle (10-130 degrees)
- `SHORT_10037db4`: Skier speed (0-1000+)
- `SHORT_10037db6`: Animation frame counter
- `SHORT_10037de2`: Jump/trick state
- `_DAT_10037dcc`: Crash state
- `SHORT_10037dae`: Control mode (0=human, 1=?, 2=?)

### Course Object Structure

Course objects appear to be stored as 10-byte records in an array:
- `_DAT_10034b70`: Base pointer to course objects array
- `_DAT_10034b74`: Number of course objects

Each object likely contains:
- Position (X, distance)
- Image/sprite ID
- Type/flags
- Collision bounds

## Core Game Loop

### Main Function (.main @ 0x100003C0)

1. **Initialization**:
   - `SetUpBody()` - System initialization
   - `DoMacSkiStartUp()` - Game-specific startup
   - `SetUpMacSkiMenus()` - Menu bar setup
   - `SetUpMacSkiGraphics()` - Graphics initialization

2. **Main Event Loop**:
   - Get next Mac OS event (`cl_get_next_event`)
   - Process events: mouse, keyboard, window updates
   - Call `DoGameMode()` if in active playing state
   - Call `DoSkiing()` for physics updates

3. **Shutdown**:
   - `quit()` function

### DoSkiing Function (@ 0x1001204C)

Main gameplay update, called each frame:

```c
void DoSkiing(void) {
    check_the_time();           // Update frame timing
    if (mode != computer) {
        human_controls();        // Process player input
    }
    CenterMouse();              // Reset mouse to center (for control)
    adjust_skier();             // Update physics/position
    if (animation_enabled) {
        animate_slope();        // Update slope animation
    }
    DrawSlope(...);             // Render the scene
    CheckForCollisions();       // Collision detection
    game_over_man();            // Check end conditions
}
```

## Physics System

### adjust_skier Function (@ 0x10012C60)

**Most complex function** - handles all skier physics and state:

1. **Speed Management**:
   - Gravity acceleration (constant downhill)
   - Angle efficiency (optimal 60-80°, poor at extremes)
   - Braking (reduces speed)
   - Edge friction (hitting boundaries)

2. **Turning**:
   - Angle changes from player input
   - Auto-centering tendency
   - Speed-dependent turn limits

3. **Special States**:
   - Jumping (reduces speed temporarily)
   - Crashing (nearly stops skier)
   - Tricks/flips (complex animation sequences)

4. **Vector Calculation** (`skier_vector`):
   - Converts angle + speed → X/Y velocity
   - Angle 70° = straight down
   - Angle <70° = left, >70° = right

### Constants (from analysis)

```c
#define MAX_SPEED 1000  // Actually can exceed, but capped around here
#define ACCEL_RATE variable  // Depends on slope angle
#define DECEL_RATE 7  // When braking
#define MIN_ANGLE 10
#define MAX_ANGLE 130
#define CENTER_ANGLE 70
#define COURSE_WIDTH 950  // Approximate (10-960 range)
```

## Collision Detection

### CheckForCollisions (@ 0x10001434)

Sophisticated rectangle-based collision system:

1. **Tracking**:
   - Maintains previous frame position
   - Creates rect from old → new position
   - Interpolates along movement path

2. **Detection**:
   - Checks obstacles within distance range
   - Uses `CL_AreRectsTouching()` Mac Toolbox function
   - Handles both point and area collisions

3. **Response** (`DoObjectCollision`):
    - Different behavior per obstacle type
    - Trees/rocks → crash
    - Jumps → launch skier
    - Flags → collect/score

## Course System

### Course File Format

Original MacSki used a custom binary format with these functions:

- `ReadCourse()` @ 0x10002D44 - Load course from file
- `WriteCourse()` @ 0x100031E0 - Save course to file
- `put_int()`, `get_int()` - Serialization helpers
- `put_long()`, `get_long()` - For larger values
- `put_chars()`, `get_chars()` - String data

### Course Generation

- `RandomCourse()` @ 0x10002124 - Procedural generation
- `SetupDefaultCourse()` @ 0x10002264 - Fallback course

## Graphics System

### QuickDraw Rendering

MacSki used QuickDraw, Apple's 2D graphics API:

- `DrawSlope()` @ 0x100140CC - Main render function
- `draw_ski_trails()` @ 0x10015170 - Render trail marks
- `DrawEditor()` @ 0x100042F4 - Course editor rendering
- `cl_draw_pict_at()` - Draw sprites
- Double buffering for flicker-free animation

### Animation

- Skier has multiple animation frames (0-5 for skiing, more for tricks)
- Frame rate tied to skier speed
- Trail marks left behind as textured lines

## Input System

### human_controls (referenced in DoSkiing)

- Mouse movement for turning (centered each frame)
- Keyboard alternatives (arrow keys, WASD-style)
- `skiing_keys()` @ 0x1000C1C4 - Key mapping
- `reset_ski_keys()` @ 0x1000BCDC - Clear input state

## Sound System

- `MacSkiPlaySoundResource()` @ 0x10016438 - Play sound effect
- Sound resources:
  - 0x450-0x458: Various skiing/crash sounds
  - 0x3FB-0x3FD: Voice samples
  - 0x44C-0x44F: Jump sounds

## Menu/UI Functions

- `SetUpMacSkiMenus()` @ 0x1000D3DC
- `do_menu()` - Handle menu selections
- `MacSkiInfoDialog()` @ 0x10010E04 - About dialog
- `SkisDialog()` @ 0x10015F28 - Equipment settings
- **Registration dialog** (removed in BevySki)

## Course Editor

- `SetUpCourseEditor()` @ 0x10004020
- `DrawEditor()` @ 0x100042F4
- `CourseObjectAtEditorMousePoint()` @ 0x10006CE8
- `CreateCourseObjectAtMousePointFromImage()` @ 0x10004C88
- Drag-and-drop object placement
- Palette of available objects

## Interesting Discoveries

1. **Ski Trails**: Dynamically rendered path behind skier with gap detection
2. **Wind System**: `WindChill()` function affects skier movement
3. **Game Modes**: Multiple play modes beyond basic skiing
4. **Serial/Multiplayer**: Evidence of serial port communication code
5. **Registration System**: Shareware check (removed for BevySki)
6. **"Brew's Ski Lodge"**: Menu/lobby system (`IsBrewsSkiLodgeActive`, `GoToBrewsSkiLodge`)

## Original Assets

At runtime, if extracted assets are missing, the game prompts for the folder containing original MacSki `.rsrc` files:
- `MacSki Color Art.rsrc` - Sprite graphics (298KB)
- `MacSki Sounds.rsrc` - Sound effects (233KB)
- `MacSki v1.7.rsrc` - Main game resources (419KB)
- `MacSki Courses/` - 44 original course files with .rsrc metadata

## Function Name Patterns

Many functions follow Mac Toolbox conventions:
- Capital letter after dot: `.SetUpBody`, `.DrawSlope`
- Lowercase prefixes: `cl_` = "Color Library" custom functions
- Descriptive names preserved from debug symbols

## Memory Layout

- Code: 0x10000000 - ~0x1002AFFF
- Data: ~0x1002B000 - 0x10039000
- Graphics/resources loaded dynamically
- Global variables heavily used (not very object-oriented)

## References

- Metrowerks CodeWarrior 8 documentation
- Mac OS Classic API (Inside Macintosh)
- QuickDraw reference
- CarbonLib type definitions (applied in Ghidra)
