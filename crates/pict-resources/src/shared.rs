//! Shared data structures and utilities for PICT decoding.
//!
//! This module contains common types used by both PICT v1 and v2 formats,
//! including geometric primitives, color tables, pixel maps, and transfer modes.

use binrw::{binread, io, BinRead, BinReaderExt, BinResult, Error};
use std::io::Read;

use crate::PictVersion;

/// Extension trait for PackBits compression decoding.
trait PackBitsReaderExt: io::Read {
    /// Reads PackBits-compressed data into the output buffer.
    ///
    /// PackBits is a simple run-length encoding scheme used in PICT files.
    fn read_packbits(&mut self, out: &mut [u8]) -> io::Result<()> {
        let mut written = 0usize;
        while written < out.len() {
            let mut control = [0u8; 1];
            self.read_exact(&mut control)?;
            let n = control[0] as i8;

            if n >= 0 {
                let literal_count = n as usize + 1;
                if written + literal_count > out.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "PackBits literal overrun",
                    ));
                }
                self.read_exact(&mut out[written..written + literal_count])?;
                written += literal_count;
                continue;
            }

            if n == -128 {
                continue;
            }

            let repeat_count = (1i16 - n as i16) as usize;
            if written + repeat_count > out.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "PackBits repeat overrun",
                ));
            }

            let mut value = [0u8; 1];
            self.read_exact(&mut value)?;
            out[written..written + repeat_count].fill(value[0]);
            written += repeat_count;
        }

        Ok(())
    }
}

impl<T: io::Read + ?Sized> PackBitsReaderExt for T {}

fn unpack_exact(input: &[u8], expected_len: usize) -> io::Result<Vec<u8>> {
    let mut cursor = io::Cursor::new(input);
    let mut out = vec![0u8; expected_len];
    cursor.read_packbits(&mut out)?;
    Ok(out)
}

fn unpack_words_exact(input: &[u8], expected_len: usize) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(expected_len);
    let mut cursor = io::Cursor::new(input);

    while out.len() < expected_len {
        let mut control = [0u8; 1];
        cursor.read_exact(&mut control)?;
        let n = control[0] as i8;

        if n >= 0 {
            let literal_words = n as usize + 1;
            let bytes = literal_words * 2;
            if out.len() + bytes > expected_len {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "PackBits word literal overrun",
                ));
            }
            let mut buf = vec![0u8; bytes];
            cursor.read_exact(&mut buf)?;
            out.extend_from_slice(&buf);
            continue;
        }

        if n == -128 {
            continue;
        }

        let repeat_words = (1i16 - n as i16) as usize;
        let bytes = repeat_words * 2;
        if out.len() + bytes > expected_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "PackBits word repeat overrun",
            ));
        }

        let mut word = [0u8; 2];
        cursor.read_exact(&mut word)?;
        for _ in 0..repeat_words {
            out.extend_from_slice(&word);
        }
    }

    if out.len() != expected_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "PackBits decoded size mismatch",
        ));
    }

    Ok(out)
}

/// Signed 16-bit QuickDraw scalar.
pub type Short = i16;
/// Unsigned 32-bit QuickDraw scalar.
pub type Long = u32; // 4 bytes

/// QuickDraw transfer mode used when compositing source and destination pixels.
#[allow(missing_docs)]
#[derive(Debug, Clone, BinRead)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default)]
pub enum TransferMode {
    #[br(magic(0u16))]
    #[default]
    SrcCopy,
    #[br(magic(1u16))]
    NotCopy,
    #[br(magic(2u16))]
    SrcOr,
    #[br(magic(3u16))]
    NotSrcOr,
    #[br(magic(4u16))]
    SrcXor,
    #[br(magic(5u16))]
    NotSrcXor,
    #[br(magic(4u16))]
    SrcBic,
    #[br(magic(5u16))]
    NotSrcBic,

    #[br(magic(32u16))]
    Blend,

    #[br(magic(33u16))]
    AddPin,
    #[br(magic(34u16))]
    AddOver,
    #[br(magic(35u16))]
    SubPin,
    #[br(magic(36u16))]
    Transparent,
    #[br(magic(37u16))]
    AddMax,
    #[br(magic(38u16))]
    SubOver,
    #[br(magic(39u16))]
    AddMin,
    #[br(magic(49u16))]
    GrayishTextOr,

    #[br(magic(50u16))]
    Highlight,

    #[br(magic(64u16))]
    DitherCopy,

    /// Unknown mode found in some AppleWorks 6 resources
    #[br(magic(100u16))]
    AppleWorksUnknown,

    Unknown(u16),
}

/// Packed pixmap bytes decoded from PICT copy-bits payloads.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PixMapData {
    data: Vec<u8>,
}

impl From<PixMapData> for Vec<u8> {
    fn from(val: PixMapData) -> Self {
        val.data
    }
}

impl BinRead for PixMapData {
    type Args<'a> = &'a PixMap;

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        pix_map: Self::Args<'_>,
    ) -> BinResult<Self> {
        let scanline_count = pix_map.bounds.height() as usize;
        let unpacked_scanline_size = pix_map.bytes_per_row() as usize;
        let unpacked_size = scanline_count * unpacked_scanline_size;

        let mut data = vec![0u8; unpacked_size];
        let start_large = unpacked_scanline_size > 250;

        match pix_map.pack_type {
            PackType::None => {
                for y in 0..scanline_count {
                    let unpacked_scanline_range =
                        (y * unpacked_scanline_size)..((y + 1) * unpacked_scanline_size);
                    reader.read_exact(&mut data[unpacked_scanline_range])?;
                }
            }
            PackType::Default => {
                if pix_map.bytes_per_row() < 8 {
                    reader
                        .read_exact(&mut data)
                        .expect("Could not read uncompressed data");
                } else {
                    decode_rle_scanlines(
                        scanline_count,
                        start_large,
                        pix_map.pixel_size == 16,
                        unpacked_scanline_size,
                        reader,
                        &mut data,
                    )
                    .expect("Could not decode default scanlines")
                }
            }
            PackType::RemovePadByte => {
                todo!("Don't know how to handle remove pad byte")
            }
            PackType::RunLengthEncoding => {
                decode_rle_scanlines(
                    scanline_count,
                    start_large,
                    true,
                    unpacked_scanline_size,
                    reader,
                    &mut data,
                )
                .expect("Could not decode RLE scanlines");
            }
            PackType::RunLengthEncodedComponents => {
                decode_rle_components_scanlines(
                    scanline_count,
                    start_large,
                    false,
                    unpacked_scanline_size,
                    pix_map.component_count as usize,
                    reader,
                    &mut data,
                )
                .expect("Could not decode rle components scanlines");

                if pix_map.component_count == 1 {
                } else if pix_map.component_count == 3 {
                    // TODO: Avoid additional copy of image data
                    let lookup = data.clone();
                    let width = pix_map.bounds.width() as usize;

                    for y in 0..scanline_count {
                        let scanline_start = y * unpacked_scanline_size;

                        for x in 0..width {
                            let a = 0;
                            let r = lookup[scanline_start + x];
                            let g = lookup[scanline_start + width + x];
                            let b = lookup[scanline_start + 2 * width + x];

                            data[scanline_start + 4 * x] = a;
                            data[scanline_start + 4 * x + 1] = r;
                            data[scanline_start + 4 * x + 2] = g;
                            data[scanline_start + 4 * x + 3] = b;
                        }
                    }
                } else if pix_map.component_count == 4 {
                    // TODO: Avoid additional copy of image data
                    let lookup = data.clone();
                    let width = pix_map.bounds.width() as usize;

                    for y in 0..scanline_count {
                        let scanline_start = y * unpacked_scanline_size;

                        for x in 0..width {
                            let a = 0xFF - lookup[scanline_start + x];
                            let r = lookup[scanline_start + width + x];
                            let g = lookup[scanline_start + 2 * width + x];
                            let b = lookup[scanline_start + 3 * width + x];

                            data[scanline_start + 4 * x] = a;
                            data[scanline_start + 4 * x + 1] = r;
                            data[scanline_start + 4 * x + 2] = g;
                            data[scanline_start + 4 * x + 3] = b;
                        }
                    }
                } else {
                    panic!(
                        "Don't know how to interleave pixels with {} components",
                        pix_map.component_count
                    );
                }
            }
            PackType::Other(v) => {
                todo!("Don't know how to handle {v} pack type")
            }
        };

        Ok(Self { data })
    }
}

fn decode_rle_components_scanlines(
    scanline_count: usize,
    start_large: bool,
    large_chunks: bool,
    unpacked_scanline_size: usize,
    component_count: usize,
    mut reader: impl io::Read + io::Seek,
    data: &mut [u8],
) -> Result<(), binrw::Error> {
    let start = reader.stream_position()?;
    let mut success = true;
    for y in 0..scanline_count {
        match decode_rle_components_scanline(
            start_large,
            large_chunks,
            unpacked_scanline_size,
            component_count,
            &mut reader,
        ) {
            Ok(scanline) => {
                let unpacked_scanline_range =
                    (y * unpacked_scanline_size)..((y + 1) * unpacked_scanline_size);
                data[unpacked_scanline_range].copy_from_slice(&scanline);
            }
            Err(_) => {
                success = false;
                break;
            }
        }
    }

    if !success {
        reader.seek(std::io::SeekFrom::Start(start))?;
        for y in 0..scanline_count {
            match decode_rle_components_scanline(
                !start_large,
                large_chunks,
                unpacked_scanline_size,
                component_count,
                &mut reader,
            ) {
                Ok(scanline) => {
                    let unpacked_scanline_range =
                        (y * unpacked_scanline_size)..((y + 1) * unpacked_scanline_size);
                    data[unpacked_scanline_range].copy_from_slice(&scanline);
                }
                Err(_) => {
                    return Err(binrw::Error::Custom {
                        pos: 0,
                        err: Box::new("Could not decode scanline"),
                    });
                }
            }
        }
    }

    Ok(())
}

fn decode_rle_scanlines(
    scanline_count: usize,
    start_large: bool,
    large_chunks: bool,
    unpacked_scanline_size: usize,
    mut reader: impl io::Read + io::Seek,
    data: &mut [u8],
) -> Result<(), binrw::Error> {
    let start = reader.stream_position()?;
    let mut success = true;
    for y in 0..scanline_count {
        match decode_rle_scanline(
            start_large,
            large_chunks,
            unpacked_scanline_size,
            &mut reader,
        ) {
            Ok(scanline) => {
                let unpacked_scanline_range =
                    (y * unpacked_scanline_size)..((y + 1) * unpacked_scanline_size);
                data[unpacked_scanline_range].copy_from_slice(&scanline);
            }
            Err(_) => {
                success = false;
                break;
            }
        }
    }

    if !success {
        reader.seek(std::io::SeekFrom::Start(start))?;
        for y in 0..scanline_count {
            match decode_rle_scanline(
                !start_large,
                large_chunks,
                unpacked_scanline_size,
                &mut reader,
            ) {
                Ok(scanline) => {
                    let unpacked_scanline_range =
                        (y * unpacked_scanline_size)..((y + 1) * unpacked_scanline_size);
                    data[unpacked_scanline_range].copy_from_slice(&scanline);
                }
                Err(_) => {
                    return Err(binrw::Error::Custom {
                        pos: 0,
                        err: Box::new("Could not decode scanline"),
                    });
                }
            }
        }
    }

    Ok(())
}

fn decode_rle_components_scanline(
    large: bool,
    large_chunks: bool,
    unpacked_scanline_size: usize,
    component_count: usize,
    mut reader: impl io::Read + io::Seek,
) -> Result<Vec<u8>, Error> {
    let mut combined_scanline = vec![0u8; unpacked_scanline_size];
    let component_scanline_size = unpacked_scanline_size / 4 * component_count;
    let packed_scanline_size: usize = if large {
        reader.read_be::<u16>()?.into()
    } else {
        reader.read_be::<u8>()?.into()
    };

    let mut packed_scanline = vec![0u8; packed_scanline_size];
    reader.read_exact(&mut packed_scanline)?;

    let component_scanline = if large_chunks {
        unpack_words_exact(&packed_scanline, component_scanline_size)
            .map_err(|e| binrw::Error::Custom {
                pos: 0,
                err: Box::new(e),
            })
            .unwrap()
    } else {
        unpack_exact(&packed_scanline, component_scanline_size)
            .map_err(|e| binrw::Error::Custom {
                pos: 0,
                err: Box::new(e),
            })
            .unwrap()
    };

    let component_range = 0..component_scanline_size;
    combined_scanline[component_range].copy_from_slice(&component_scanline);

    Ok(combined_scanline)
}

fn decode_rle_scanline(
    large: bool,
    large_chunks: bool,
    unpacked_scanline_size: usize,
    mut reader: impl io::Read + io::Seek,
) -> Result<Vec<u8>, Error> {
    let packed_scanline_size: usize = if large {
        reader.read_be::<u16>()?.into()
    } else {
        reader.read_be::<u8>()?.into()
    };

    let mut packed_scanline = vec![0u8; packed_scanline_size];
    reader.read_exact(&mut packed_scanline)?;

    if large_chunks {
        unpack_words_exact(&packed_scanline, unpacked_scanline_size).map_err(|e| {
            binrw::Error::Custom {
                pos: 0,
                err: Box::new(e),
            }
        })
    } else {
        unpack_exact(&packed_scanline, unpacked_scanline_size).map_err(|e| binrw::Error::Custom {
            pos: 0,
            err: Box::new(e),
        })
    }
}

/// 16.16 fixed-point number used by QuickDraw.
#[derive(BinRead, Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fixed {
    /// Signed integer part.
    pub int: i16,
    /// Fractional part.
    pub fraction: i16,
}

/// Compression mode for pixmap scanline data.
#[derive(BinRead, Debug, Clone, PartialEq, Eq)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PackType {
    #[br(magic(0u16))]
    /// Use default packing
    Default,
    #[br(magic(1u16))]
    /// Use no packing
    None,
    #[br(magic(2u16))]
    /// Remove pad byte - supported only for 32-bit pixels (24-bit data)
    RemovePadByte,
    #[br(magic(3u16))]
    /// Run length encoding by pixelSize chunks, one scan line at a time - supported only for 16-bit pixels
    RunLengthEncoding,
    #[br(magic(4u16))]
    /// Run length encoding one component at a time, one scan line at a time, red component first - supported only for 32-bit pixels (24-bit data)
    RunLengthEncodedComponents,

    /// Unrecognized pack type value.
    Other(u16),
}

/// QuickDraw rectangle, using top/left/bottom/right coordinates.
#[derive(BinRead, Debug, Copy, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rect {
    /// Top Y coordinate.
    pub top: Short,
    /// Left X coordinate.
    pub left: Short,
    /// Bottom Y coordinate.
    pub bottom: Short,
    /// Right X coordinate.
    pub right: Short,
}

impl Rect {
    /// Creates a rectangle with origin at `(0, 0)` and the given size.
    pub fn new_with_size(width: Short, height: Short) -> Self {
        Self {
            top: 0,
            left: 0,
            bottom: width,
            right: height,
        }
    }

    /// Returns rectangle height.
    pub fn height(&self) -> Short {
        self.bottom - self.top
    }

    /// Returns rectangle width.
    pub fn width(&self) -> Short {
        self.right - self.left
    }

    /// Returns the top-left point.
    pub fn origin(&self) -> Point {
        Point {
            x: self.left,
            y: self.top,
        }
    }

    /// Returns `true` when `other` is fully contained by this rectangle.
    pub fn contains(&self, other: &Rect) -> bool {
        self.min_x() <= other.min_x()
            && self.min_y() <= other.min_y()
            && self.max_x() >= other.max_x()
            && self.max_y() >= other.max_y()
    }

    /// Returns `true` when the given point lies inside this rectangle.
    pub fn includes(&self, x: i32, y: i32) -> bool {
        self.min_x() as i32 <= x
            && self.max_x() as i32 > x
            && self.min_y() as i32 <= y
            && self.max_y() as i32 > y
    }

    /// Minimum X coordinate.
    pub fn min_x(&self) -> Short {
        self.left
    }

    /// Maximum X coordinate.
    pub fn max_x(&self) -> Short {
        self.right
    }

    /// Minimum Y coordinate.
    pub fn min_y(&self) -> Short {
        self.top
    }

    /// Maximum Y coordinate.
    pub fn max_y(&self) -> Short {
        self.bottom
    }
}

/// 2D point used by QuickDraw commands.
#[derive(BinRead, Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point {
    /// Horizontal coordinate.
    pub x: i16,
    /// Vertical coordinate.
    pub y: i16,
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.x, self.y)
    }
}

/// 8-byte monochrome pattern.
#[derive(BinRead, Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pattern {
    /// Pattern bit data.
    pub data: [u8; 8],
}

/// Variant tag for pixel patterns.
#[allow(missing_docs)]
#[derive(BinRead, Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PatternType {
    #[br(magic(1u16))]
    Normal,

    #[br(magic(2u16))]
    Dither,

    Unknown(u16),
}

/// Pixel pattern value used by color pattern opcodes.
#[derive(BinRead, Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PixelPattern {
    pattern_type: PatternType,
    monochrome: Pattern,

    #[br(if(pattern_type.is_normal()))]
    normal_pattern: Option<PatternImage>,

    #[br(if(pattern_type.is_dither()))]
    dither_color: Option<RGBColor>,
}

/// Embedded pattern image payload for `PixelPattern`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatternImage {
    pixmap: PixMap,
    color_table: ColorTable,
    data: Vec<u8>,
}

impl BinRead for PatternImage {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let row_bytes_and_flags_hi: u8 = reader.read_be()?;
        let pixmap: PixMap = reader.read_be_args((row_bytes_and_flags_hi,))?;
        let color_table: ColorTable = reader.read_be()?;

        let scanline_count = pixmap.bounds.height() as usize;
        let scanline_size = pixmap.bytes_per_row() as usize;
        let unpacked_size = scanline_count * scanline_size;
        let mut data = vec![0u8; unpacked_size];

        if scanline_size <= 8 {
            reader.read_exact(&mut data)?;
        } else {
            for y in 0..scanline_count {
                #[allow(unused)]
                let packed_scanline_size: usize = if scanline_size > 250 {
                    reader.read_be::<u16>()?.into()
                } else {
                    reader.read_be::<u8>()?.into()
                };
                // TODO: Use packed_scanline_size to limit bytes read
                let unpacked_scanline_range = (y * scanline_size)..((y + 1) * scanline_size);
                reader.read_packbits(&mut data[unpacked_scanline_range])?;
            }
        };

        Ok(Self {
            pixmap,
            color_table,
            data: Vec::new(),
        })
    }
}

/// Pascal-style short string (`len` followed by bytes).
#[binread]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShortString {
    #[br(temp)]
    string_len: u8,
    #[br(count = string_len)]
    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    /// Decoded UTF-8 lossy string payload.
    pub string: String,
}

impl From<ShortString> for String {
    fn from(val: ShortString) -> Self {
        val.string
    }
}

/// Arc angle in degrees used by arc opcodes.
pub type Angle = u16;

/// QuickDraw region, represented as a run-length encoded mask.
#[binread]
#[br(big)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Region {
    /// Encoded region record size in bytes.
    pub size: u16,
    /// Bounding box for the region, if present.
    #[br(if(size >= 2 + 8))]
    pub bounding_box: Option<Rect>,
    /// Encoded row/column toggle data.
    #[br(count(size.saturating_sub(10) / 2))]
    pub data: Vec<i16>,
    #[br(ignore)]
    decoded_mask: Option<Vec<u8>>,
}

impl Default for Region {
    fn default() -> Self {
        Self::new()
    }
}

impl Region {
    /// Creates an empty region.
    pub fn new() -> Self {
        Region {
            size: 0,
            bounding_box: None,
            data: Vec::new(),
            decoded_mask: None,
        }
    }

    /// Prepares the decoded mask cache for point-in-region queries.
    pub fn prepare(&mut self) {
        if self.is_prepared() {
            return;
        }

        self.decoded_mask = Some(self.as_mask());
    }

    fn is_prepared(&self) -> bool {
        self.decoded_mask.is_some() || self.data.is_empty()
    }

    /// Returns `true` if the point is inside the prepared region mask.
    #[inline]
    pub fn contains(&self, x: i32, y: i32) -> bool {
        assert!(self.is_prepared(), "Mask has not bee prepared yet");

        let Some(b) = self.bounding_box.as_ref() else {
            return true;
        };

        if !b.includes(x, y) {
            return false;
        }

        let Some(r) = self.decoded_mask.as_ref() else {
            return true;
        };

        let idx = y as usize * b.max_x() as usize + x as usize;
        r[idx] != 0
    }

    fn as_mask(&self) -> Vec<u8> {
        if !self.size.is_multiple_of(2) {
            panic!("Size is not even!");
        }
        // Decode `data` as run-length encoded sliding window mask as described by Hackerjack
        // See https://info-mac.org/viewtopic.php?t=17328 for the post
        if let Some(rect) = self.bounding_box {
            let width = rect.max_x() as usize;
            let height = rect.max_y() as usize;

            if rect.min_x() != 0 || rect.min_y() != 0 {
                log::warn!("Mask is offset: {rect:?}");
            }

            if self.data.is_empty() {
                log::warn!(
                    "It does not really make sense to force us to create a mask if you can just check the bounding box"
                );
                return vec![1u8; width * height];
            }

            let mut image: Vec<u8> = vec![0u8; width * height];
            let mut cursor = self.data.iter();
            let mut last_row = 0;
            let mut scanline: Vec<u8> = vec![0u8; width];
            loop {
                match cursor.next() {
                    None => {
                        log::warn!("Unexpected end of data");
                        return image;
                    }
                    Some(0x7fffi16) => {
                        if last_row < height {
                            // apply mask row to rest of image
                            for row in last_row..height {
                                image[(row * width)..((row + 1) * width)]
                                    .copy_from_slice(&scanline);
                            }
                        }
                        log::info!("Reached end of region");
                        return image;
                    }
                    Some(row) => loop {
                        let row = *row as usize;
                        // apply last row mask to all rows up until this new one
                        for row in last_row..row {
                            image[(row * width)..((row + 1) * width)].copy_from_slice(&scanline);
                        }
                        last_row = row;

                        match cursor.next() {
                            None => {
                                log::warn!("Unexpected end of data in row {row}");
                                return image;
                            }
                            Some(0x7fffi16) => {
                                if row >= height {
                                    log::warn!(
                                        "Won't apply row {row} that's out of range ({height})"
                                    );
                                } else {
                                    image[(row * width)..((row + 1) * width)]
                                        .copy_from_slice(&scanline);
                                }
                                last_row = row;
                                break;
                            }

                            Some(start) => {
                                if row >= height {
                                    log::warn!("Row {row} is out of range");
                                    continue;
                                }
                                let Some(end) = cursor.next() else {
                                    log::warn!("Unexpected end of data in row {row}");
                                    return image;
                                };

                                for (column, item) in scanline
                                    .iter_mut()
                                    .enumerate()
                                    .skip(*start as usize)
                                    .take(*end as usize - *start as usize)
                                {
                                    if column >= width {
                                        log::error!("Column {column} is out of range");
                                        continue;
                                    }

                                    *item = if *item == 0 { 1 } else { 0 };
                                }
                            }
                        }
                    },
                }
            }
        } else {
            log::warn!("Don't know how to create mask image without bounding box");
            vec![]
        }
    }
}

/// Polygon payload used by poly drawing opcodes.
#[derive(BinRead, Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Polygon {
    /// Size of the polygon record.
    pub size: u16,
    #[br(count(size - 2))]
    /// Raw polygon point payload bytes.
    pub data: Vec<u8>,
}

#[repr(C)]
/// source modes for color graphics ports
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SourceMode {
    ///  determine how close the color of the source pixel is
    /// to black, and assign this relative amount of
    /// foreground color to the destination pixel; determine
    /// how close the color of the source pixel is to white,
    /// and assign this relative amount of background color
    /// to the destination pixel
    SrcCopy = 0,
    /// determine how close the color of the source pixel is
    /// to black, and assign this relative amount of
    /// foreground color to the destination pixel
    SrcOr = 1,
    /// where source pixel is black, invert the destination
    /// pixel--for a colored destination pixel, use the
    /// complement of its color if the pixel is direct,
    /// invert its index if the pixel is indexed
    SrcXor = 2,
    ///  determine how close the color of the source pixel is
    /// to black, and assign this relative amount of
    /// background color to the destination pixel
    SrcBic = 3,
    /// determine how close the color of the source pixel is
    /// to black, and assign this relative amount of
    /// background color to the destination pixel; determine
    /// how close the color of the source pixel is to white,
    /// and assign this relative amount of foreground color
    /// to the destination pixel
    NotSrcCopy = 4,
    /// determine how close the color of the source pixel is
    /// to white, and assign this relative amount of
    /// foreground color to the destination pixel
    NotSrcOr = 5,
    /// where source pixel is white, invert destination
    /// pixel--for a colored destination pixel, use the
    /// complement of its color if the pixel is direct,
    /// invert its index if the pixel is indexed
    NotSrcXor = 6,
    /// determine how close the color of the source pixel is
    /// to white, and assign this relative amount of
    /// background color to the destination pixel
    NotSrcBic = 7,
}

/// RGB color in QuickDraw 16-bit channel format.
#[derive(BinRead, Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RGBColor {
    /// magnitude of red component
    pub red: u16,
    /// magnitude of green component
    pub green: u16,
    /// magnitude of blue component
    pub blue: u16,
}

/// Generic counted byte buffer.
#[derive(BinRead, Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Buffer {
    /// Number of bytes in `data`.
    pub len: u16,
    #[br(count(len))]
    /// Buffer payload bytes.
    pub data: Vec<u8>,
}

/// Color table entry.
#[derive(BinRead, Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColorSpec {
    /// index or other value
    pub value: Short,
    /// true color
    pub rgb: RGBColor,
}

/// QuickDraw color table used for indexed pixmaps.
#[binread]
#[derive(Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColorTable {
    /// unique identifier for table
    pub ct_seed: u32,
    /// high bit: 0 = PixMap; 1 = device
    pub ct_flags: u16,
    /// number of entries in next field
    #[br(temp)]
    pub ct_size: u16,
    ///  array[0..0] of ColorSpec records
    #[br(count(if ct_size == 0xFFFF {0} else { ct_size + 1 }))]
    pub ct_table: Vec<ColorSpec>,
}

impl Default for ColorTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorTable {
    /// Creates an empty color table.
    pub fn new() -> Self {
        ColorTable {
            ct_seed: 0,
            ct_flags: 0,
            ct_table: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Parsed payload for CopyBits/PackBits style opcodes.
pub enum CopyBits {
    /// Four opcodes ($0090, $0091, $0098, $0099) are modifications of version 1 opcodes. The first word
    /// following the opcode is rowBytes. If the high bit of rowBytes is set, then it is a pixel map
    /// containing multiple bits per pixel; if it is not set, it is a bitmap containing 1 bit per pixel. In
    /// general, the difference between version 2 and version 1 formats is that the pixel map replaces
    /// the bitmap, a color table has been added, and pixData replaces bitData.
    Pixmap {
        /// Source pixmap definition.
        pix_map: PixMap,
        /// Source color table for indexed pixmaps.
        color_table: ColorTable,
        /// Source rectangle.
        src_rect: Rect,
        /// Destination rectangle.
        dst_rect: Rect,
        /// Transfer mode.
        mode: TransferMode,
        /// Optional mask region.
        mask_region: Option<Region>,
        /// Pixel bytes.
        data: Vec<u8>,
    },
    /// 1-bit bitmap copy payload.
    Bitmap {
        /// Row-bytes including flags.
        bytes_per_row: u16,
        /// Source bitmap bounds.
        bounds: Rect,
        /// Transfer mode.
        mode: TransferMode,
        /// Source rectangle.
        src_rect: Rect,
        /// Destination rectangle.
        dst_rect: Rect,
        /// Optional mask region.
        mask_region: Option<Region>,
        /// Bitmap bytes.
        data: Vec<u8>,
    },
}

impl BinRead for CopyBits {
    type Args<'a> = (bool, bool, PictVersion);

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        (packed, masked, version): Self::Args<'_>,
    ) -> BinResult<Self> {
        let row_bytes_and_flags_hi: u8 = reader.read_be()?;
        let data_is_pixmap = row_bytes_and_flags_hi & 0x80 != 0;
        if data_is_pixmap {
            let pix_map: PixMap = reader.read_be_args((row_bytes_and_flags_hi,))?;
            let color_table: ColorTable = reader.read_be()?;

            let src_rect: Rect = reader.read_be()?;
            let dst_rect: Rect = reader.read_be()?;

            let mode = reader.read_be()?;
            let mask_region: Option<Region> = if masked {
                Some(reader.read_be()?)
            } else {
                None
            };

            let scanline_count = pix_map.bounds.height() as usize;
            let bytes_per_row = pix_map.bytes_per_row() as usize;
            let unpacked_size = scanline_count * bytes_per_row;

            let data = if !packed || version.is_version2() && bytes_per_row < 8 {
                let mut data = vec![0u8; unpacked_size];
                reader.read_exact(&mut data).unwrap();
                data
            } else {
                reader.read_be_args::<PixMapData>(&pix_map)?.into()
            };

            Ok(CopyBits::Pixmap {
                pix_map,
                color_table,
                src_rect,
                dst_rect,
                mode,
                mask_region,
                data,
            })
        } else {
            let row_bytes_lo: u8 = reader.read_be()?;
            let flags_and_row_bytes = ((row_bytes_and_flags_hi as u16) << 8) | row_bytes_lo as u16;
            let bounds: Rect = reader.read_be()?;
            let src_rect: Rect = reader.read_be()?;
            let dst_rect: Rect = reader.read_be()?;

            let mode = reader.read_be()?;
            let mask_region: Option<Region> = if masked {
                Some(reader.read_be()?)
            } else {
                None
            };

            let scanline_count = bounds.height() as usize;
            let bytes_per_row = (flags_and_row_bytes & 0x7FFF) as usize;
            let unpacked_size = scanline_count * bytes_per_row;
            let mut data = vec![0u8; unpacked_size];

            if !packed || version.is_version2() && bytes_per_row < 8 {
                reader.read_exact(&mut data)?;
            } else {
                decode_rle_scanlines(
                    scanline_count,
                    bytes_per_row > 250,
                    false,
                    bytes_per_row,
                    reader,
                    &mut data,
                )?;
            };

            Ok(CopyBits::Bitmap {
                bytes_per_row: flags_and_row_bytes,
                bounds,
                mode,
                src_rect,
                dst_rect,
                mask_region,
                data,
            })
        }
    }
}

/// Pixel encoding category for a pixmap.
#[allow(missing_docs)]
#[binread]
#[derive(Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PixelType {
    #[br(magic(0i16))]
    Indexed,
    #[br(magic(16i16))]
    DirectColor,

    Unknown(i16),
}

impl TransferMode {
    /// Returns `true` if this transfer mode is not recognized by the parser.
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
}

impl PatternType {
    /// Returns `true` when this pattern uses an embedded color pattern image.
    pub const fn is_normal(&self) -> bool {
        matches!(self, Self::Normal)
    }

    /// Returns `true` when this pattern is represented by a dither color.
    pub const fn is_dither(&self) -> bool {
        matches!(self, Self::Dither)
    }
}

impl PixelType {
    /// Returns `true` when the pixmap uses indexed color entries.
    pub const fn is_indexed(&self) -> bool {
        matches!(self, Self::Indexed)
    }

    /// Returns `true` when the pixmap stores direct color values.
    pub const fn is_direct_color(&self) -> bool {
        matches!(self, Self::DirectColor)
    }

    /// Returns `true` when the value is not recognized.
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
}

/// QuickDraw pixmap header.
#[binread]
#[derive(Debug, Clone)]
#[br(big, import(row_bytes_and_flags_low_byte: u8))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PixMap {
    /// flags, and row width
    #[br(map(|low_bytes:u8| (((row_bytes_and_flags_low_byte & 0x7f) as u16) <<8) | (low_bytes  as u16) ))]
    pub row_bytes_and_flags: u16,
    /// boundary rectangle
    pub bounds: Rect,
    /// PixMap version number
    pub pm_version: i16,
    /// packing format
    pub pack_type: PackType,
    /// size of data in packed state
    pub pack_size: u32,
    /// horizontal resolution (dpi)
    pub h_res: Fixed,
    /// vertical resolution (dpi)
    pub v_res: Fixed,
    /// format of pixel image
    pub pixel_type: PixelType,
    /// physical bits per pixel
    pub pixel_size: i16,
    /// logical components per pixel
    pub component_count: i16,
    /// logical bits per component
    pub component_size: i16,
    /// offset to next plane
    pub plane_bytes: u32,
    /// handle to the ColorTable struct
    pub pm_table: u32,
    /// reserved for future expansion; must be 0
    // #[br(temp, assert(pm_reserved==0))]
    pub pm_reserved: u32,
}

impl PixMap {
    /// Returns logical bytes per row, stripping high-bit flags.
    pub fn bytes_per_row(&self) -> u16 {
        self.row_bytes_and_flags & 0x7FFF
    }
}

/// Structured QuickDraw comment kind.
#[allow(missing_docs)]
#[derive(Debug, BinRead, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CommentKind {
    /// Archaic grouping command
    #[br(magic(0u16))]
    LParen,
    /// Ends group begun by RParen
    #[br(magic(1u16))]
    RParen,
    /// Application-specific comment
    #[br(magic(100u16))]
    AppComment,
    /// Begin MacDraw picture
    #[br(magic(130u16))]
    DwgBeg,
    /// End MacDraw picture
    #[br(magic(131u16))]
    DwgEnd,
    /// Begin grouped objects
    #[br(magic(140u16))]
    GrpBeg,
    /// End grouped objects
    #[br(magic(141u16))]
    GrpEnd,
    /// Begin series of bitmap bands
    #[br(magic(142u16))]
    BitBeg,
    /// End of bitmap bands
    #[br(magic(143u16))]
    BitEnd,
    /// Beginning of a text string
    #[br(magic(150u16))]
    TextBegin,
    /// End of text string
    #[br(magic(151u16))]
    TextEnd,
    /// Beginning of banded string
    #[br(magic(152u16))]
    StringBegin,
    /// End of banded string
    #[br(magic(153u16))]
    StringEnd,
    /// Center of rotation to TextBegin
    #[br(magic(154u16))]
    TextCenter,
    /// Turns off line layout
    #[br(magic(155u16))]
    LineLayoutOff,
    /// Turns on line layout
    #[br(magic(156u16))]
    LineLayoutOn,
    /// Specify line layout for next text call
    #[br(magic(157u16))]
    LineLayout,
    /// Following LineTo() operations are part of a polygon
    #[br(magic(160u16))]
    PolyBegin,
    /// End of special MacDraw polygon
    #[br(magic(161u16))]
    PolyEnd,
    /// Following data part of freehand curve
    #[br(magic(162u16))]
    PlyByt,
    /// Ignore the following polygon
    #[br(magic(163u16))]
    PolyIgnore,
    /// Close, fill, frame a polygon
    #[br(magic(164u16))]
    PolySmooth,
    /// MacDraw polygon is closed
    #[br(magic(165u16))]
    PlyClose,
    /// One arrow from point1 to point2
    #[br(magic(170u16))]
    Arrw1,
    /// One arrow from point2 to point1
    #[br(magic(171u16))]
    Arrw2,
    /// Two arrows, one on each end of a line
    #[br(magic(172u16))]
    Arrw3,
    /// End of arrow comment
    #[br(magic(173u16))]
    ArrwEnd,
    /// Subsequent lines are PostScript dashed lines
    #[br(magic(180u16))]
    DashedLine,
    /// Ends picDashedLine comment
    #[br(magic(181u16))]
    DashedStop,
    /// Mult. fraction for pen size
    #[br(magic(182u16))]
    SetLineWidth,
    /// Saves QD state; and send PostScript
    #[br(magic(190u16))]
    PostScriptBegin,
    /// Restore QD state
    #[br(magic(191u16))]
    PostScriptEnd,
    /// Remaining data is PostScript
    #[br(magic(192u16))]
    PostScriptHandle,
    /// Use filename to send 'POST' resources
    #[br(magic(193u16))]
    PostScriptFile,
    /// QD text is PostScript until PostScriptEnd
    #[br(magic(194u16))]
    TextIsPostScript,
    /// Send PostScript from 'STR ' or 'STR#' resources
    #[br(magic(195u16))]
    ResourcePS,
    /// Like PostScriptBegin
    #[br(magic(196u16))]
    NewPostScriptBegin,
    /// Set gray level from fixed-point number
    #[br(magic(197u16))]
    SetGrayLevel,
    /// Begin rotation of the coordinate plane
    #[br(magic(200u16))]
    RotateBegin,
    /// End rotated plane
    #[br(magic(201u16))]
    RotateEnd,
    /// Specifies center of rotation
    #[br(magic(202u16))]
    RotateCenter,
    /// DonÕt flush print buffer after each page
    #[br(magic(210u16))]
    FormsPrinting,
    /// Ends forms printing
    #[br(magic(211u16))]
    EndFormsPrinting,
    /// used by MacDraw II
    #[br(magic(214u16))]
    AutoNap,
    /// used by MacDraw II
    #[br(magic(215u16))]
    AutoWake,
    /// used by MacDraw II
    #[br(magic(216u16))]
    ManNap,
    /// used by MacDraw II
    #[br(magic(217u16))]
    ManWake,
    /// File creator for application
    #[br(magic(498u16))]
    Creator,
    /// Scaling of image
    #[br(magic(499u16))]
    PICTScale,
    /// Begin bitmap thinning
    #[br(magic(1000u16))]
    BegBitmapThin,
    /// End bitmap thinning
    #[br(magic(1001u16))]
    EndBitmapThin,
    /// Image in the scrap created using lasso
    #[br(magic(12345u16))]
    Lasso,

    Unknown(u16),
}

/*
 100 is an Application Comment (see below).
220 is used for ICC profile data.
498 appears to be related to Photoshop, though it might also be used for other things.
*/
