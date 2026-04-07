//! Classic Macintosh PICT resource decoding and PNG conversion.

use binrw::{BinRead, BinReaderExt};
use fourcc::fourcc;
use resource_fork::ResourceFork;
use std::io::{Cursor, Seek};
use std::path::Path;
use thiserror::Error;

pub mod drawing_context;
pub mod shared;
pub mod v1;
pub mod v2;

pub use drawing_context::DrawingContext;

/// Errors returned when converting PICT resources.
#[derive(Debug, Error)]
pub enum PictResourceError {
    /// Error from reading classic Macintosh resource forks.
    #[error(transparent)]
    ResourceFork(#[from] resource_fork::Error),
    /// Error while parsing binary PICT structures.
    #[error(transparent)]
    BinRead(#[from] binrw::Error),
    /// I/O error while reading or writing image data.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Error reported by the `image` crate while encoding PNG output.
    #[error(transparent)]
    Image(#[from] image::ImageError),
}

/// PICT file major version marker found in the picture stream.
#[allow(missing_docs)]
#[derive(BinRead, Debug)]
#[br(big)]
pub enum PictVersion {
    /// Original PICT v1 marker.
    #[br(magic(0x11u8))]
    Version1(u8),
    /// Extended PICT v2 marker.
    #[br(magic(0x0011u16))]
    Version2,
}

impl PictVersion {
    /// Returns `true` when this marker is PICT v2.
    pub const fn is_version2(&self) -> bool {
        matches!(self, Self::Version2)
    }
}

#[derive(BinRead, Debug)]
#[br(big)]
struct PictHeader {
    size: u16,
    bounds: shared::Rect,
    version: PictVersion,
}

/// Decodes a single raw PICT payload into an RGBA image.
pub fn decode_pict_bytes_to_image(
    data: &[u8],
) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, PictResourceError> {
    let mut cursor = Cursor::new(data);
    let header: PictHeader = cursor.read_be()?;
    let mut context = DrawingContext::new(header.bounds);
    let _ = header.size;

    match header.version {
        PictVersion::Version1(_) => loop {
            let opcode: v1::Opcode = cursor.read_be()?;
            if context.command_v1(opcode) {
                break;
            }
        },
        PictVersion::Version2 => loop {
            let opcode: v2::Opcode = cursor.read_be()?;
            if context.command_v2(opcode) {
                break;
            }
            if cursor.stream_position()? % 2 == 1 {
                let _: u8 = cursor.read_be()?;
            }
        },
    }

    Ok(context.into_image())
}

/// Extracts all `PICT` resources from a resource fork and writes PNG files.
///
/// Returns the list of written PNG paths.
pub fn extract_pict_resources_to_png(
    resource_fork_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<Vec<std::path::PathBuf>, PictResourceError> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir)?;

    let mut fork = ResourceFork::open(resource_fork_path.as_ref())?;
    let ids = fork.list_resources(fourcc!("PICT"))?;

    let mut outputs = Vec::new();
    for id in ids {
        let (data, _) = fork.read_data(fourcc!("PICT"), id)?;

        let png_path = output_dir.join(format!("PICT_{}.png", id));
        match decode_pict_bytes_to_image(&data) {
            Ok(image) => {
                image.save_with_format(&png_path, image::ImageFormat::Png)?;
                outputs.push(png_path);
            }
            Err(_) => {
                continue;
            }
        }
    }

    Ok(outputs)
}
