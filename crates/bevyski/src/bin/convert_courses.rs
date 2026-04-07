//! Legacy MacSki course file converter.
//!
//! This binary utility converts original MacSki course files into JSON format
//! for use in BevySki. It uses the exact file format recovered from Ghidra
//! decompilation of the original MacSki executable.
//!
//! # File Format
//!
//! MacSki course files use XOR encoding and contain:
//! - Format version marker (0x0096)
//! - Text sections (course name, briefing, etc.)
//! - Configuration values
//! - Object records (obstacles, flags, etc.)
//! - Checksum validation
//!
//! # Usage
//!
//! ```sh
//! cargo run --bin convert_courses [input_dir] [output_dir]
//! ```
//!
//! Defaults:
//! - Input: `assets/original/MacSki Courses`
//! - Output: `courses/legacy`

use serde::Serialize;
use std::ffi::OsStr;
use std::path::PathBuf;

/// Expected format version identifier from MacSki v1.7.
const MAGIC_VERSION: u16 = 0x0096;

/// Initial checksum seed used in course file validation.
const CHECKSUM_SEED: u16 = 0x7011;

/// XOR key for integer (u16) decoding.
const INT_XOR_KEY: u8 = 0xba;

/// XOR key for character string decoding.
const CHAR_XOR_KEY: u8 = 0x5b;

/// JSON output structure for a legacy course file.
#[derive(Serialize)]
struct LegacyCourseJson {
    /// Human-readable course name.
    name: String,
    /// Original source filename.
    source_file: String,
    /// Course file format version.
    format_version: u16,
    /// Parsed text sections from the course file.
    text_sections: TextSections,
    /// Course configuration values.
    config: CourseConfig,
    /// Reserved/unknown values from the file.
    reserved_values: [u16; 6],
    /// Number of objects/obstacles in the course.
    object_count: usize,
    /// Obstacle and feature records.
    objects: Vec<CourseObjectRecord>,
    /// Checksum read from the file.
    checksum_read: u16,
    /// Checksum computed during parsing.
    checksum_computed: u16,
    /// Whether the full checksum is valid.
    checksum_valid: bool,
    /// Whether the low byte of the checksum is valid (fallback check).
    checksum_low_byte_valid: bool,
    /// Associated `.rsrc` sidecar file, if present.
    rsrc_file: Option<String>,
    /// Asset mappings extracted from `.rsrc` file.
    rsrc_assets: Vec<RsrcAssetMapping>,
}

/// Text sections from a MacSki course file.
#[derive(Serialize)]
struct TextSections {
    /// Introductory course name.
    intro_name: String,
    /// Course briefing text.
    briefing: String,
    /// Additional text line 2.
    line_2: String,
    /// Additional text line 3.
    line_3: String,
}

/// Course configuration values from the file.
#[derive(Serialize)]
struct CourseConfig {
    /// Default course flag indicator.
    default_course_flag: u16,
    /// Configuration value at offset 0x10034b84.
    value_10034b84: u16,
    /// Configuration value at offset 0x10034b82.
    value_10034b82: u16,
    /// Configuration value at offset 0x10034b80.
    value_10034b80: u16,
    /// Configuration value at offset 0x10034b7c.
    value_10034b7c: u16,
    /// Configuration value at offset 0x10034b78.
    value_10034b78: u32,
}

/// A single object/obstacle record from the course file.
#[derive(Serialize)]
struct CourseObjectRecord {
    /// Index of this object in the course.
    index: usize,
    /// Horizontal position on the course.
    x: u16,
    /// Image/sprite ID for this object.
    image_id: u16,
    /// Distance down the course where this object appears.
    distance: u32,
}

/// Asset mapping extracted from `.rsrc` sidecar file.
#[derive(Serialize)]
struct RsrcAssetMapping {
    /// Asset code string (e.g., "HIL1iSki").
    asset_code: String,
    /// Type of asset mapping.
    mapping_kind: String,
    /// Hill variant number, if applicable.
    hill_variant: Option<u8>,
}

/// Main entry point for the course converter.
///
/// Reads legacy MacSki course files from the source directory and converts
/// them to JSON format in the output directory.
///
/// # Arguments
///
/// 1. Source directory (default: `assets/original/MacSki Courses`)
/// 2. Output directory (default: `courses/legacy`)
///
/// # Errors
///
/// Returns an error if directories cannot be accessed or files cannot be parsed.
fn main() -> Result<(), String> {
    let source_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("assets/original/MacSki Courses"));
    let output_dir = std::env::args()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("courses/legacy"));

    if !source_dir.exists() {
        return Err(format!(
            "Source directory does not exist: {}",
            source_dir.display()
        ));
    }

    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    let mut converted = 0usize;
    for entry in std::fs::read_dir(&source_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }
        if path
            .extension()
            .is_some_and(|ext| ext == OsStr::new("rsrc"))
        {
            continue;
        }

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let raw = std::fs::read(&path).map_err(|e| e.to_string())?;
        if raw.is_empty() {
            continue;
        }

        let parsed = match parse_course_file(&raw) {
            Ok(parsed) => parsed,
            Err(err) => return Err(format!("Failed to parse {file_name}: {err}")),
        };

        let rsrc_path = source_dir.join(format!("{}.rsrc", file_name));

        let mut rsrc_assets = Vec::new();
        if rsrc_path.exists() {
            if let Ok(mappings) = extract_rsrc_mappings(&rsrc_path) {
                rsrc_assets = mappings;
            }
        }

        let output = LegacyCourseJson {
            name: normalize_course_name(&file_name),
            source_file: file_name.clone(),
            format_version: parsed.format_version,
            text_sections: parsed.text_sections,
            config: parsed.config,
            reserved_values: parsed.reserved_values,
            object_count: parsed.objects.len(),
            objects: parsed.objects,
            checksum_read: parsed.checksum_read,
            checksum_computed: parsed.checksum_computed,
            checksum_valid: parsed.checksum_read == parsed.checksum_computed,
            checksum_low_byte_valid: (parsed.checksum_read & 0xff)
                == (parsed.checksum_computed & 0xff),
            rsrc_file: rsrc_path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.to_string())
                .filter(|_| rsrc_path.exists()),
            rsrc_assets,
        };

        let output_path = output_dir.join(format!("{}.json", slugify(&file_name)));
        let json = serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?;
        std::fs::write(output_path, json).map_err(|e| e.to_string())?;
        converted += 1;
    }

    println!(
        "Converted {} legacy course files into {}",
        converted,
        output_dir.display()
    );
    Ok(())
}

/// Normalizes a course name by trimming and removing carriage returns.
fn normalize_course_name(name: &str) -> String {
    name.trim().replace("\r", "")
}

/// Converts a course name to a URL-friendly slug.
fn slugify(name: &str) -> String {
    let trimmed = normalize_course_name(name).to_lowercase();
    let mut out = String::with_capacity(trimmed.len());
    let mut last_dash = false;

    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }

    out.trim_matches('-').to_string()
}

/// Result of parsing a course file.
struct CourseParseResult {
    /// Format version number.
    format_version: u16,
    /// Parsed text sections.
    text_sections: TextSections,
    /// Course configuration.
    config: CourseConfig,
    /// Reserved/unknown values.
    reserved_values: [u16; 6],
    /// Parsed object records.
    objects: Vec<CourseObjectRecord>,
    /// Checksum read from file.
    checksum_read: u16,
    /// Checksum computed during parsing.
    checksum_computed: u16,
}

/// Cursor for reading XOR-encoded course files with checksum tracking.
struct Cursor<'a> {
    /// Raw file data.
    data: &'a [u8],
    /// Current read position.
    pos: usize,
    /// Running checksum value.
    checksum: u16,
}

impl<'a> Cursor<'a> {
    /// Creates a new cursor with initial checksum seed.
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            checksum: CHECKSUM_SEED,
        }
    }

    /// Reads a single byte without XOR decoding.
    ///
    /// # Errors
    ///
    /// Returns an error if EOF is reached.
    fn read_u8(&mut self) -> Result<u8, String> {
        if self.pos >= self.data.len() {
            return Err("Unexpected EOF".to_string());
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    /// Reads an XOR-encoded big-endian u16 (`get_int` from decompilation).
    ///
    /// Updates the running checksum with the decoded value.
    fn get_int(&mut self) -> Result<u16, String> {
        let hi = self.read_u8()? ^ INT_XOR_KEY;
        let lo = self.read_u8()? ^ INT_XOR_KEY;
        let value = u16::from_be_bytes([hi, lo]);
        self.checksum ^= value;
        Ok(value)
    }

    /// Reads an XOR-encoded big-endian u32 (`get_long` from decompilation).
    ///
    /// Reads two u16 values and combines them.
    fn get_long(&mut self) -> Result<u32, String> {
        let hi = self.get_int()? as u32;
        let lo = self.get_int()? as u32;
        Ok((hi << 16) | lo)
    }

    /// Reads an XOR-encoded NUL-terminated string (`get_chars` from decompilation).
    ///
    /// Updates the checksum with each decoded byte.
    fn get_chars(&mut self) -> Result<String, String> {
        let mut out = Vec::new();
        for _ in 0..=0xfe {
            let decoded = self.read_u8()? ^ CHAR_XOR_KEY;
            self.checksum ^= decoded as u16;
            if decoded == 0 {
                break;
            }
            out.push(decoded);
        }
        Ok(String::from_utf8_lossy(&out).to_string())
    }

    /// Reads the checksum value without XOR decoding (`get_check` from decompilation).
    fn get_check(&mut self) -> Result<u16, String> {
        let hi = self.read_u8()? as u16;
        let lo = self.read_u8()? as u16;
        Ok((hi << 8) | lo)
    }
}

/// Parses a MacSki course file using the decompiled format.
///
/// # Errors
///
/// Returns an error if the file format is invalid or the version doesn't match.
fn parse_course_file(bytes: &[u8]) -> Result<CourseParseResult, String> {
    let mut c = Cursor::new(bytes);

    let format_version = c.get_int()?;
    let intro_name = c.get_chars()?;
    let briefing = c.get_chars()?;
    let line_2 = c.get_chars()?;
    let line_3 = c.get_chars()?;

    let config = CourseConfig {
        default_course_flag: c.get_int()?,
        value_10034b84: c.get_int()?,
        value_10034b82: c.get_int()?,
        value_10034b80: c.get_int()?,
        value_10034b7c: c.get_int()?,
        value_10034b78: c.get_long()?,
    };

    let mut reserved_values = [0u16; 6];
    for value in &mut reserved_values {
        *value = c.get_int()?;
    }

    let object_count = c.get_int()? as usize;
    let mut objects = Vec::with_capacity(object_count);
    for index in 0..object_count {
        let x = c.get_int()?;
        let image_id = c.get_int()?;
        let distance = c.get_long()?;
        objects.push(CourseObjectRecord {
            index,
            x,
            image_id,
            distance,
        });
    }

    let checksum_read = c.get_check()?;
    let checksum_computed = c.checksum;

    if format_version != MAGIC_VERSION {
        return Err(format!(
            "Unexpected course version: {format_version} (expected {MAGIC_VERSION})"
        ));
    }

    Ok(CourseParseResult {
        format_version,
        text_sections: TextSections {
            intro_name,
            briefing,
            line_2,
            line_3,
        },
        config,
        reserved_values,
        objects,
        checksum_read,
        checksum_computed,
    })
}

/// Extracts asset mappings from a `.rsrc` sidecar file.
///
/// Searches for patterns like "HIL?iSki" that indicate course hill art variants.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
fn extract_rsrc_mappings(path: &PathBuf) -> Result<Vec<RsrcAssetMapping>, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut mappings = Vec::new();

    if bytes.len() >= 8 {
        for i in 0..=(bytes.len() - 8) {
            if &bytes[i..i + 3] == b"HIL" && &bytes[i + 4..i + 8] == b"iSki" {
                let variant_byte = bytes[i + 3];
                let variant = if variant_byte.is_ascii_digit() {
                    Some(variant_byte - b'0')
                } else {
                    None
                };

                let code = String::from_utf8_lossy(&bytes[i..i + 8]).to_string();
                mappings.push(RsrcAssetMapping {
                    asset_code: code,
                    mapping_kind: "course_hill_art".to_string(),
                    hill_variant: variant,
                });
            }
        }
    }

    mappings.sort_by(|a, b| a.asset_code.cmp(&b.asset_code));
    mappings.dedup_by(|a, b| a.asset_code == b.asset_code);
    Ok(mappings)
}
