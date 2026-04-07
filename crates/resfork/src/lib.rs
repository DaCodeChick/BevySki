//! Parser for classic Macintosh resource fork files (.rsrc)
//!
//! This crate provides functionality to read and parse resource fork files
//! from classic Mac OS applications. Resource forks contain structured data
//! including images, sounds, courses, and other game assets.
//!
//! # Format Overview
//!
//! A resource fork file consists of:
//! - Resource Header (256 bytes)
//! - Resource Data
//! - Resource Map
//!
//! # Example
//!
//! ```no_run
//! use resfork::ResourceFork;
//!
//! let fork = ResourceFork::open("MacSki Color Art.rsrc").unwrap();
//! for resource_type in fork.resource_types() {
//!     println!("Type: {}", resource_type);
//! }
//! ```

use byteorder::{BigEndian, ReadBytesExt};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use thiserror::Error;

pub mod types;

/// Errors that can occur when parsing resource forks.
#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid resource fork header")]
    InvalidHeader,

    #[error("Invalid resource type: {0}")]
    InvalidType(String),

    #[error("Resource not found: type={0}, id={1}")]
    NotFound(String, i16),

    #[error("Invalid resource data")]
    InvalidData,
}

/// A four-character code (OSType) used to identify resource types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResType([u8; 4]);

impl ResType {
    /// Creates a new resource type from a four-character code.
    pub const fn new(code: &[u8; 4]) -> Self {
        Self(*code)
    }

    /// Creates a resource type from a string (truncates/pads to 4 bytes).
    pub fn from_str(s: &str) -> Self {
        let mut code = [b' '; 4];
        let bytes = s.as_bytes();
        let len = bytes.len().min(4);
        code[..len].copy_from_slice(&bytes[..len]);
        Self(code)
    }

    /// Returns the type code as a string.
    pub fn as_str(&self) -> String {
        String::from_utf8_lossy(&self.0).to_string()
    }
}

impl std::fmt::Display for ResType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A single resource within the resource fork.
#[derive(Debug, Clone)]
pub struct Resource {
    /// Resource type (four-character code).
    pub res_type: ResType,
    /// Resource ID.
    pub id: i16,
    /// Resource name (if present).
    pub name: Option<String>,
    /// Raw resource data.
    pub data: Vec<u8>,
}

/// Resource fork file parser.
pub struct ResourceFork {
    /// File handle.
    file: File,
    /// Offset to resource data section.
    data_offset: u32,
    /// Offset to resource map.
    map_offset: u32,
    /// Resources indexed by type and ID.
    resources: HashMap<ResType, HashMap<i16, Resource>>,
}

impl ResourceFork {
    /// Opens and parses a resource fork file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or is not a valid resource fork.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, ResourceError> {
        let mut file = File::open(path)?;

        // Read resource fork header (first 16 bytes)
        file.seek(SeekFrom::Start(0))?;

        let data_offset = file.read_u32::<BigEndian>()?;
        let map_offset = file.read_u32::<BigEndian>()?;
        let _data_length = file.read_u32::<BigEndian>()?;
        let _map_length = file.read_u32::<BigEndian>()?;

        let mut fork = Self {
            file,
            data_offset,
            map_offset,
            resources: HashMap::new(),
        };

        fork.parse_resource_map()?;

        Ok(fork)
    }

    /// Parses the resource map section.
    fn parse_resource_map(&mut self) -> Result<(), ResourceError> {
        self.file.seek(SeekFrom::Start(self.map_offset as u64))?;

        // Skip reserved fields and attributes
        self.file.seek(SeekFrom::Current(22))?;

        // Read type list offset
        let type_list_offset = self.file.read_u16::<BigEndian>()? as u32;
        let _name_list_offset = self.file.read_u16::<BigEndian>()? as u32;

        // Jump to type list
        self.file.seek(SeekFrom::Start(
            self.map_offset as u64 + type_list_offset as u64,
        ))?;

        // Read number of types (count - 1)
        let type_count = self.file.read_u16::<BigEndian>()? as usize + 1;

        // Read each resource type
        for _ in 0..type_count {
            let mut type_code = [0u8; 4];
            self.file.read_exact(&mut type_code)?;
            let res_type = ResType::new(&type_code);

            let resource_count = self.file.read_u16::<BigEndian>()? as usize + 1;
            let ref_list_offset = self.file.read_u16::<BigEndian>()? as u32;

            // Save current position
            let current_pos = self.file.stream_position()?;

            // Jump to reference list for this type
            let ref_list_pos =
                self.map_offset as u64 + type_list_offset as u64 + ref_list_offset as u64;
            self.file.seek(SeekFrom::Start(ref_list_pos))?;

            // Read each resource reference
            let mut type_resources = HashMap::new();
            for _ in 0..resource_count {
                let id = self.file.read_i16::<BigEndian>()?;
                let name_offset = self.file.read_i16::<BigEndian>()?;
                let data_offset_raw = self.file.read_u32::<BigEndian>()?;
                let _reserved = self.file.read_u32::<BigEndian>()?;

                // Extract data offset (top 3 bytes, mask off attributes)
                let data_offset = (data_offset_raw & 0x00FFFFFF) as u32;

                // Read resource name if present
                let name = if name_offset >= 0 {
                    self.read_resource_name(name_offset as u32)?
                } else {
                    None
                };

                // Read resource data
                let data = self.read_resource_data(data_offset)?;

                type_resources.insert(
                    id,
                    Resource {
                        res_type,
                        id,
                        name,
                        data,
                    },
                );
            }

            self.resources.insert(res_type, type_resources);

            // Restore position to continue reading type list
            self.file.seek(SeekFrom::Start(current_pos))?;
        }

        Ok(())
    }

    /// Reads a resource name from the name list.
    fn read_resource_name(&mut self, _offset: u32) -> Result<Option<String>, ResourceError> {
        // TODO: Implement name reading from name list
        Ok(None)
    }

    /// Reads resource data from the data section.
    fn read_resource_data(&mut self, offset: u32) -> Result<Vec<u8>, ResourceError> {
        self.file
            .seek(SeekFrom::Start(self.data_offset as u64 + offset as u64))?;

        // Read data length
        let length = self.file.read_u32::<BigEndian>()? as usize;

        // Read data
        let mut data = vec![0u8; length];
        self.file.read_exact(&mut data)?;

        Ok(data)
    }

    /// Returns all resource types in this fork.
    pub fn resource_types(&self) -> Vec<ResType> {
        self.resources.keys().copied().collect()
    }

    /// Gets a resource by type and ID.
    pub fn get_resource(&self, res_type: ResType, id: i16) -> Option<&Resource> {
        self.resources.get(&res_type)?.get(&id)
    }

    /// Gets all resources of a specific type.
    pub fn get_resources_by_type(&self, res_type: ResType) -> Option<&HashMap<i16, Resource>> {
        self.resources.get(&res_type)
    }

    /// Returns the total number of resources.
    pub fn resource_count(&self) -> usize {
        self.resources.values().map(|m| m.len()).sum()
    }
}
