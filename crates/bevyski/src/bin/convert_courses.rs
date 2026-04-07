use serde::Serialize;
use std::ffi::OsStr;
use std::path::PathBuf;

const MAGIC_VERSION: u16 = 0x0096;
const CHECKSUM_SEED: u16 = 0x7011;
const INT_XOR_KEY: u8 = 0xba;
const CHAR_XOR_KEY: u8 = 0x5b;

#[derive(Serialize)]
struct LegacyCourseJson {
    name: String,
    source_file: String,
    format_version: u16,
    text_sections: TextSections,
    config: CourseConfig,
    reserved_values: [u16; 6],
    object_count: usize,
    objects: Vec<CourseObjectRecord>,
    checksum_read: u16,
    checksum_computed: u16,
    checksum_valid: bool,
    checksum_low_byte_valid: bool,
    rsrc_file: Option<String>,
    rsrc_assets: Vec<RsrcAssetMapping>,
}

#[derive(Serialize)]
struct TextSections {
    intro_name: String,
    briefing: String,
    line_2: String,
    line_3: String,
}

#[derive(Serialize)]
struct CourseConfig {
    default_course_flag: u16,
    value_10034b84: u16,
    value_10034b82: u16,
    value_10034b80: u16,
    value_10034b7c: u16,
    value_10034b78: u32,
}

#[derive(Serialize)]
struct CourseObjectRecord {
    index: usize,
    x: u16,
    image_id: u16,
    distance: u32,
}

#[derive(Serialize)]
struct RsrcAssetMapping {
    asset_code: String,
    mapping_kind: String,
    hill_variant: Option<u8>,
}

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

fn normalize_course_name(name: &str) -> String {
    name.trim().replace("\r", "")
}

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

struct CourseParseResult {
    format_version: u16,
    text_sections: TextSections,
    config: CourseConfig,
    reserved_values: [u16; 6],
    objects: Vec<CourseObjectRecord>,
    checksum_read: u16,
    checksum_computed: u16,
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
    checksum: u16,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            checksum: CHECKSUM_SEED,
        }
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        if self.pos >= self.data.len() {
            return Err("Unexpected EOF".to_string());
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn get_int(&mut self) -> Result<u16, String> {
        let hi = self.read_u8()? ^ INT_XOR_KEY;
        let lo = self.read_u8()? ^ INT_XOR_KEY;
        let value = u16::from_be_bytes([hi, lo]);
        self.checksum ^= value;
        Ok(value)
    }

    fn get_long(&mut self) -> Result<u32, String> {
        let hi = self.get_int()? as u32;
        let lo = self.get_int()? as u32;
        Ok((hi << 16) | lo)
    }

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

    fn get_check(&mut self) -> Result<u16, String> {
        let hi = self.read_u8()? as u16;
        let lo = self.read_u8()? as u16;
        Ok((hi << 8) | lo)
    }
}

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
