# resfork

A Rust library for parsing classic Macintosh resource fork files (.rsrc).

## Overview

This crate provides functionality to read and parse resource fork files from classic Mac OS applications. Resource forks contain structured data including images, sounds, courses, and other game assets.

## Status

🚧 **Work in Progress** - Basic structure is in place, but the parser needs refinement to handle the specific format used by MacSki resource files.

## Features

- Parse resource fork file headers
- Extract resources by type and ID
- Support for MacSki-specific resource types (COLRiSki, HILLiSki, etc.)
- Type-safe resource type handling with `ResType`

## Usage

```rust
use resfork::ResourceFork;

let fork = ResourceFork::open("MacSki Color Art.rsrc")?;

// List all resource types
for res_type in fork.resource_types() {
    println!("Type: {}", res_type);
}

// Get specific resource
use resfork::types::macski;
if let Some(resource) = fork.get_resource(macski::PICT, 128) {
    println!("Found PICT resource #{}", resource.id);
}
```

## Resource Fork Format

Classic Macintosh resource forks consist of:

1. **Resource Header** - File metadata and offsets
2. **Resource Data** - Raw binary data for each resource
3. **Resource Map** - Index of all resources with type, ID, and name

Each resource is identified by:
- **Type**: Four-character code (e.g., "PICT", "snd ", "STR ")
- **ID**: 16-bit signed integer
- **Name**: Optional string identifier

## MacSki Resources

The original MacSki v1.7 uses several custom resource types:

- `COLRiSki` - Color palettes
- `HILLiSki` - Course terrain data
- `PICT` - Graphics/sprites
- `snd ` - Sound effects
- `STR ` - Text strings
- `vers` - Version information

## Next Steps

- [ ] Debug and fix resource fork parsing for MacSki files
- [ ] Implement resource name extraction
- [ ] Add parsers for specific resource types (PICT, snd, etc.)
- [ ] Add course file parser for MacSki course format
- [ ] Export functionality for converting resources to modern formats

## License

GPL-3.0 - Same as BevySki project
