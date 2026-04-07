//! PICT version 2 format opcodes and structures.
//!
//! PICT v2 is the extended QuickDraw picture format introduced with 32-bit
//! QuickDraw, supporting color images, pixmaps, and more advanced operations.

use binrw::{binread, BinRead, BinReaderExt, BinResult};
#[cfg(feature = "serde")]
use serde_big_array::BigArray;

use crate::PictVersion;

use super::shared::*;

/// PICT version 2 opcode enumeration.
///
/// Version 2 opcodes include all the operations from v1 plus support for
/// color images, pixmaps, and additional QuickDraw features.
#[derive(BinRead, Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FixedRect {
    pub left: Fixed,
    pub top: Fixed,
    pub right: Fixed,
    pub bottom: Fixed,
}

#[derive(BinRead, Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MinorVersion {
    #[br(magic(-1i32))]
    Normal,
    #[br(magic(-2i32))]
    Extended,

    Unknown(i32),
}

#[binread]
#[derive(Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header {
    pub version: MinorVersion,
    /// fixed-point bounding rectangle for picture
    pub bounds: FixedRect,
    //#[br(temp, assert(reserved == 0x00000000u32))]
    /// reserved
    pub reserved: u32,
}

#[derive(BinRead, Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Opcode {
    /// No operation
    #[br(magic(0x0000u16))]
    NOP,
    /// Clipping region
    #[br(magic(0x0001u16))]
    Clip(Region),
    /// Background pattern
    #[br(magic(0x0002u16))]
    BkPat(Pattern),
    /// Font number for text
    #[br(magic(0x0003u16))]
    TxFont(u16),
    /// Text's font style
    #[br(magic(0x0004u16))]
    TxFace(u8),
    /// Source mode
    #[br(magic(0x0005u16))]
    TxMode(u16),
    /// Extra space (Fixed)
    #[br(magic(0x0006u16))]
    SpExtra(Fixed),
    /// Pen size (Point)
    #[br(magic(0x0007u16))]
    PnSize(Point),
    /// Pen mode
    #[br(magic(0x0008u16))]
    PnMode(u16),
    /// Pen pattern
    #[br(magic(0x0009u16))]
    PnPat(Pattern),
    /// Fill pattern
    #[br(magic(0x000Au16))]
    FillPat(Pattern),
    /// Oval size
    #[br(magic(0x000Bu16))]
    OvSize(Point),
    /// dh, dv
    #[br(magic(0x000Cu16))]
    Origin(u16, u16),
    /// Text size
    #[br(magic(0x000Du16))]
    TxSize(u16),
    /// Foreground color
    #[br(magic(0x000Eu16))]
    FgColor(Long),
    /// Background color
    #[br(magic(0x000Fu16))]
    BkColor(Long),
    /// Numerator (Point), denominator (Point)
    #[br(magic(0x0010u16))]
    TxRatio(Point, Point),
    /// Version
    #[br(magic(0x0011u16))]
    VersionOp,
    /// Background pixel pattern    Variable; see Listing A-1 on page A-17
    #[br(magic(0x0012u16))]
    BkPixPat(PixelPattern),
    /// Pen pixel pattern    Variable; see Listing A-1 on page A-17
    #[br(magic(0x0013u16))]
    PnPixPat(PixelPattern),
    /// Fill pixel pattern    Variable; see Listing A-1 on page A-17
    #[br(magic(0x0014u16))]
    FillPixPat(PixelPattern),
    /// Fractional pen position (u16 - low word of Fixed); if value is not 0.5, pen position is always set to the picture before each text-drawing operation. 2
    #[br(magic(0x0015u16))]
    PnLocHFrac(u16),
    /// Added width for non-space characters
    #[br(magic(0x0016u16))]
    ChExtra(u16),
    /// use Notdetermined
    #[br(magic(0x0017u16))]
    ReservedForAppleUse,
    /// use Notdetermined
    #[br(magic(0x0018u16))]
    ReservedForAppleUse1,
    /// use Notdetermined
    #[br(magic(0x0019u16))]
    ReservedForAppleUse2,
    /// Foreground color
    #[br(magic(0x001Au16))]
    RGBFgColor(RGBColor),
    /// Background color
    #[br(magic(0x001Bu16))]
    RGBBkCol(RGBColor),
    /// Highlight mode flag: no data; this opcode is sent before a drawing operation that uses the highlight mode
    #[br(magic(0x001Cu16))]
    HiliteMode,
    /// Highlight color
    #[br(magic(0x001Du16))]
    HiliteColor(RGBColor),
    /// Use default highlight color; no data; set highlight to default (from low memory)
    #[br(magic(0x001Eu16))]
    DefHilite,
    /// Opcolor
    #[br(magic(0x001Fu16))]
    OpColor(RGBColor),
    /// pnLoc (Point), newPt (Point)
    #[br(magic(0x0020u16))]
    Line(Point, Point),
    /// newPt
    #[br(magic(0x0021u16))]
    LineFrom(Point),
    /// pnLoc (Point), dh (-128..127), dv (-128..127)
    #[br(magic(0x0022u16))]
    ShortLine(Point, i8, i8),
    /// dh (-128..127), dv (-128..127)
    #[br(magic(0x0023u16))]
    ShortLineFrom(i8, i8),
    /// Data length (u16), data
    #[br(magic(0x0024u16))]
    ReservedForAppleUse3(u16, #[br(count(self_0))] Vec<u8>),
    /// Data length (u16), data
    #[br(magic(0x0025u16))]
    ReservedForAppleUse4(u16, #[br(count(self_0))] Vec<u8>),
    /// Data length (u16), data
    #[br(magic(0x0026u16))]
    ReservedForAppleUse5(u16, #[br(count(self_0))] Vec<u8>),
    /// Data length (u16), data
    #[br(magic(0x0027u16))]
    ReservedForAppleUse6(u16, #[br(count(self_0))] Vec<u8>),
    /// txLoc (Point), count (0..255), text
    #[br(magic(0x0028u16))]
    LongText(Point, ShortString),
    /// dh (0..255), count (0..255), text
    #[br(magic(0x0029u16))]
    DHText(u8, ShortString),
    /// dv (0..255), count (0..255), text
    #[br(magic(0x002Au16))]
    DVText(u8, ShortString),
    /// dh (0..255), dv (0..255), count (0..255), text
    #[br(magic(0x002Bu16))]
    DHDVText(u8, u8, ShortString),
    /// Data length (u16), old font ID (u16), name length (0..255), font name*
    #[br(magic(0x002Cu16))]
    FontName(u16, u16, ShortString),
    /// Operand data length (u16), intercharacter spacing (Fixed), total extra space for justification (Fixed)†
    #[br(magic(0x002Du16))]
    LineJustify(u16, Fixed, Fixed),
    /// Data length (word), followed by these 1-byte Boolean values: outline preferred, preserve glyph, fractional widths, scaling disabled        8
    #[br(magic(0x002Eu16))]
    GlyphState(u16, u8, u8, u8, u8),
    /// Data length (u16), data
    #[br(magic(0x002Fu16))]
    ReservedForAppleUse7(u16, #[br(count(self_0))] Vec<u8>),
    /// Rectangle (Rect)
    #[br(magic(0x0030u16))]
    FrameRect(Rect),
    /// Rectangle (Rect)
    #[br(magic(0x0031u16))]
    PaintRect(Rect),
    /// Rectangle (Rect)
    #[br(magic(0x0032u16))]
    EraseRect(Rect),
    /// Rectangle (Rect)
    #[br(magic(0x0033u16))]
    InvertRect(Rect),
    /// Rectangle (Rect)
    #[br(magic(0x0034u16))]
    FillRect(Rect),
    /// 8 bytes of data
    #[br(magic(0x0035u16))]
    ReservedForAppleUse8(Rect),
    /// 8 bytes of data
    #[br(magic(0x0036u16))]
    ReservedForAppleUse9(Rect),
    /// 8 bytes of data
    #[br(magic(0x0037u16))]
    ReservedForAppleUse10(Rect),
    /// Rectangle (Rect)
    #[br(magic(0x0038u16))]
    FrameSameRect,
    /// Rectangle (Rect)
    #[br(magic(0x0039u16))]
    PaintSameRect,
    /// Rectangle (Rect)
    #[br(magic(0x003Au16))]
    EraseSameRect,
    /// Rectangle (Rect)
    #[br(magic(0x003Bu16))]
    InvertSameRect,
    /// Rectangle (Rect)
    #[br(magic(0x003Cu16))]
    FillSameRect,
    #[br(magic(0x003Du16))]
    ReservedForAppleUse11,
    #[br(magic(0x003Eu16))]
    ReservedForAppleUse12,
    #[br(magic(0x003Fu16))]
    ReservedForAppleUse13,
    /// ‡ Rectangle (Rect)
    #[br(magic(0x0040u16))]
    FrameRRect(Rect),
    /// Rectangle (Rect)‡
    #[br(magic(0x0041u16))]
    PaintRRect(Rect),
    /// Rectangle (Rect)‡
    #[br(magic(0x0042u16))]
    EraseRRect(Rect),
    /// Rectangle (Rect)‡
    #[br(magic(0x0043u16))]
    InvertRRect(Rect),
    /// Rectangle (Rect)‡
    #[br(magic(0x0044u16))]
    FillRRect(Rect),
    /// 8 bytes of data
    #[br(magic(0x0045u16))]
    ReservedForAppleUse14(Rect),
    /// 8 bytes of data
    #[br(magic(0x0046u16))]
    ReservedForAppleUse15(Rect),
    /// 8 bytes of data
    #[br(magic(0x0047u16))]
    ReservedForAppleUse16(Rect),
    /// Rectangle (Rect)
    #[br(magic(0x0048u16))]
    FrameSameRRect,
    /// Rectangle (Rect)
    #[br(magic(0x0049u16))]
    PaintSameRRect,
    /// Rectangle (Rect)
    #[br(magic(0x004Au16))]
    EraseSameRRect,
    /// Rectangle (Rect)
    #[br(magic(0x004Bu16))]
    InvertSameRRect,
    /// Rectangle (Rect)
    #[br(magic(0x004Cu16))]
    FillSameRRect,
    #[br(magic(0x004Du16))]
    ReservedForAppleUse17,
    #[br(magic(0x004Eu16))]
    ReservedForAppleUse18,
    #[br(magic(0x004Fu16))]
    ReservedForAppleUse19,
    /// Rectangle (Rect)
    #[br(magic(0x0050u16))]
    FrameOval(Rect),
    /// Rectangle (Rect)
    #[br(magic(0x0051u16))]
    PaintOval(Rect),
    /// Rectangle (Rect)
    #[br(magic(0x0052u16))]
    EraseOval(Rect),
    /// Rectangle (Rect)
    #[br(magic(0x0053u16))]
    InvertOval(Rect),
    /// Rectangle (Rect)
    #[br(magic(0x0054u16))]
    FillOval(Rect),
    /// 8 bytes of data
    #[br(magic(0x0055u16))]
    ReservedForAppleUse20(Rect),
    /// 8 bytes of data
    #[br(magic(0x0056u16))]
    ReservedForAppleUse21(Rect),
    /// 8 bytes of data
    #[br(magic(0x0057u16))]
    ReservedForAppleUse22(Rect),
    /// Rectangle (Rect)
    #[br(magic(0x0058u16))]
    FrameSameOval,
    /// Rectangle (Rect)
    #[br(magic(0x0059u16))]
    PaintSameOval,
    /// Rectangle (Rect)
    #[br(magic(0x005Au16))]
    EraseSameOval,
    /// Rectangle (Rect)
    #[br(magic(0x005Bu16))]
    InvertSameOval,
    /// Rectangle (Rect)
    #[br(magic(0x005Cu16))]
    FillSameOval,
    #[br(magic(0x005Du16))]
    ReservedForAppleUse23,
    #[br(magic(0x005Eu16))]
    ReservedForAppleUse24,
    #[br(magic(0x005Fu16))]
    ReservedForAppleUse25,
    /// Rectangle (Rect), startAngle, arcAngle
    #[br(magic(0x0060u16))]
    FrameArc(Rect, u16, u16),
    /// Rectangle (Rect), startAngle, arcAngle
    #[br(magic(0x0061u16))]
    PaintArc(Rect, u16, u16),
    /// Rectangle (Rect), startAngle, arcAngle
    #[br(magic(0x0062u16))]
    EraseArc(Rect, u16, u16),
    /// Rectangle (Rect), startAngle, arcAngle
    #[br(magic(0x0063u16))]
    InvertArc(Rect, u16, u16),
    /// Rectangle (Rect), startAngle, arcAngle
    #[br(magic(0x0064u16))]
    FillArc(Rect, u16, u16),
    /// 12 bytes of data
    #[br(magic(0x0065u16))]
    ReservedForAppleUse26(Rect, u16, u16),
    /// 12 bytes of data
    #[br(magic(0x0066u16))]
    ReservedForAppleUse27(Rect, u16, u16),
    /// 12 bytes of data
    #[br(magic(0x0067u16))]
    ReservedForAppleUse28(Rect, u16, u16),
    /// Rectangle (Rect)
    #[br(magic(0x0068u16))]
    FrameSameArc(u16, u16),
    /// Rectangle (Rect)
    #[br(magic(0x0069u16))]
    PaintSameArc(u16, u16),
    /// Rectangle (Rect)
    #[br(magic(0x006Au16))]
    EraseSameArc(u16, u16),
    /// Rectangle (Rect)
    #[br(magic(0x006Bu16))]
    InvertSameArc(u16, u16),
    /// Rectangle (Rect)
    #[br(magic(0x006Cu16))]
    FillSameArc(u16, u16),
    /// 4 bytes of data
    #[br(magic(0x006Du16))]
    ReservedForAppleUse29(u16, u16),
    /// 4 bytes of data
    #[br(magic(0x006Eu16))]
    ReservedForAppleUse30(u16, u16),
    /// 4 bytes of data
    #[br(magic(0x006Fu16))]
    ReservedForAppleUse31(u16, u16),
    /// Polygon (Poly)
    #[br(magic(0x0070u16))]
    FramePoly(Polygon),
    /// Polygon (Poly)
    #[br(magic(0x0071u16))]
    PaintPoly(Polygon),
    /// Polygon (Poly)
    #[br(magic(0x0072u16))]
    ErasePoly(Polygon),
    /// Polygon (Poly)
    #[br(magic(0x0073u16))]
    InvertPoly(Polygon),
    /// Polygon (Poly)
    #[br(magic(0x0074u16))]
    FillPoly(Polygon),
    /// Polygon (Poly)
    #[br(magic(0x0075u16))]
    ReservedForAppleUse32(Polygon),
    /// Polygon (Poly)
    #[br(magic(0x0076u16))]
    ReservedForAppleUse33(Polygon),
    /// Polygon (Poly)
    #[br(magic(0x0077u16))]
    ReservedForAppleUse34(Polygon),
    /// (Not yet implemented)
    #[br(magic(0x0078u16))]
    FrameSamePoly,
    /// (Not yet implemented)
    #[br(magic(0x0079u16))]
    PaintSamePoly,
    /// (Not yet implemented)
    #[br(magic(0x007Au16))]
    EraseSamePoly,
    /// (Not yet implemented)
    #[br(magic(0x007Bu16))]
    InvertSamePoly,
    /// (Not yet implemented)
    #[br(magic(0x007Cu16))]
    FillSamePoly,
    #[br(magic(0x007Du16))]
    ReservedForAppleUse35,
    #[br(magic(0x007Eu16))]
    ReservedForAppleUse36,
    #[br(magic(0x007Fu16))]
    ReservedForAppleUse37,
    /// Region(Rgn)
    #[br(magic(0x0080u16))]
    FrameRgn(Region),
    /// Region(Rgn)
    #[br(magic(0x0081u16))]
    PaintRgn(Region),
    /// Region(Rgn)
    #[br(magic(0x0082u16))]
    EraseRgn(Region),
    /// Region(Rgn)
    #[br(magic(0x0083u16))]
    InvertRgn(Region),
    /// Region (Rgn)
    #[br(magic(0x0084u16))]
    FillRgn(Region),
    /// Region (Rgn)
    #[br(magic(0x0085u16))]
    ReservedForAppleUse38(Region),
    /// Region (Rgn)
    #[br(magic(0x0086u16))]
    ReservedForAppleUse39(Region),
    /// Region (Rgn)
    #[br(magic(0x0087u16))]
    ReservedForAppleUse40(Region),
    /// (Not yet implemented)
    #[br(magic(0x0088u16))]
    FrameSameRgn,
    /// (Not yet implemented)
    #[br(magic(0x0089u16))]
    PaintSameRgn,
    /// (Not yet implemented)
    #[br(magic(0x008Au16))]
    EraseSameRgn,
    /// (Not yet implemented)
    #[br(magic(0x008Bu16))]
    InvertSameRgn,
    /// (Not yet implemented)
    #[br(magic(0x008Cu16))]
    FillSameRgn,
    #[br(magic(0x008Du16))]
    ReservedForAppleUse41,
    #[br(magic(0x008Eu16))]
    ReservedForAppleUse42,
    #[br(magic(0x008Fu16))]
    ReservedForAppleUse43,
    /// CopyBits with clipped rectangle     Variable§¶; see Listing A-2 on page A-17
    #[br(magic(0x0090u16))]
    BitsRect(#[br(args(true, false, PictVersion::Version2))] CopyBits),
    /// CopyBits with clipped region     Variable§¶; see Listing A-3 on page A-18
    #[br(magic(0x0091u16))]
    BitsRgn(#[br(args(false, true, PictVersion::Version2))] CopyBits),
    /// Data length (u16), data
    #[br(magic(0x0092u16))]
    ReservedForAppleUse44(u16, #[br(count(self_0))] Vec<u8>),
    /// Data length (u16), data
    #[br(magic(0x0093u16))]
    ReservedForAppleUse45(u16, #[br(count(self_0))] Vec<u8>),
    /// Data length (u16), data
    #[br(magic(0x0094u16))]
    ReservedForAppleUse46(u16, #[br(count(self_0))] Vec<u8>),
    /// Data length (u16), data
    #[br(magic(0x0095u16))]
    ReservedForAppleUse47(u16, #[br(count(self_0))] Vec<u8>),
    /// Data length (u16), data
    #[br(magic(0x0096u16))]
    ReservedForAppleUse48(u16, #[br(count(self_0))] Vec<u8>),
    /// Data length (u16), data
    #[br(magic(0x0097u16))]
    ReservedForAppleUse49(u16, #[br(count(self_0))] Vec<u8>),
    /// Packed CopyBits with clipped rectangle
    #[br(magic(0x0098u16))]
    PackBitsRect(#[br(args(true, false, PictVersion::Version2))] CopyBits),
    /// Packed CopyBits with clipped rectangle
    #[br(magic(0x0099u16))]
    PackBitsRgn(#[br(args(true, true, PictVersion::Version2))] CopyBits),
    /// PixMap, srcRect, dstRect, mode (u16), PixData
    #[br(magic(0x009Au16))]
    DirectBitsRect(#[br(args(false))] DirectBits),
    /// PixMap, srcRect, dstRect, mode (u16), maskRgn, PixData
    #[br(magic(0x009Bu16))]
    DirectBitsRgn(#[br(args(true))] DirectBits),
    /// Data length (u16), data
    #[br(magic(0x009Cu16))]
    ReservedForAppleUse50(u16, #[br(count(self_0))] Vec<u8>),
    /// Data length (u16), data
    #[br(magic(0x009Du16))]
    ReservedForAppleUse51(u16, #[br(count(self_0))] Vec<u8>),
    /// Data length (u16), data
    #[br(magic(0x009Eu16))]
    ReservedForAppleUse52(u16, #[br(count(self_0))] Vec<u8>),
    /// Data length (u16), data
    #[br(magic(0x009Fu16))]
    ReservedForAppleUse53(u16, #[br(count(self_0))] Vec<u8>),
    /// Kind (u16)
    #[br(magic(0x00A0u16))]
    ShortComment(CommentKind),
    /// Kind (u16), size (u16), data
    #[br(magic(0x00A1u16))]
    LongComment(CommentKind, u16, #[br(count(self_1))] Vec<u8>),
    /// Data length (u16), data
    #[br(magic(0x00A2u16))]
    ReservedForAppleUse54(u16, #[br(count(self_0))] Vec<u8>),
    // ...
    /// Data length (u16), data
    #[br(magic(0x00AFu16))]
    ReservedForAppleUse55(u16, #[br(count(self_0))] Vec<u8>),
    #[br(magic(0x00B0u16))]
    ReservedForAppleUse56,
    // ...
    #[br(magic(0x00CFu16))]
    ReservedForAppleUse57,
    /// Data length (Long), data
    #[br(magic(0x00D0u16))]
    ReservedForAppleUse58(Long, #[br(count(self_0))] Vec<u8>),
    // ...
    /// Data length (Long), data
    #[br(magic(0x00FEu16))]
    ReservedForAppleUse59(Long, #[br(count(self_0))] Vec<u8>),
    /// End of picture
    #[br(magic(0x00FFu16))]
    OpEndPic,
    /// 2 bytes of data
    #[br(magic(0x0100u16))]
    ReservedForAppleUse60(u16),
    // ...
    /// 2 bytes of data
    #[br(magic(0x01FFu16))]
    ReservedForAppleUse61(u16),
    /// 4 bytes of data
    #[br(magic(0x0200u16))]
    ReservedForAppleUse62(u32),
    /// Version number of picture
    #[br(magic(0x02FFu16))]
    Version,
    // ...
    /// 22 bytes of data
    #[br(magic(0x0BFFu16))]
    ReservedForAppleUse63([u8; 22]),
    /// For extended version 2: version  (u16), reserved (u16), hRes, vRes (Fixed), srcRect, reserved (Long); for version 2: opcode
    #[br(magic(0x0C00u16))]
    HeaderOp(Header),
    /// 24 bytes of data
    #[br(magic(0x0C01u16))]
    ReservedForAppleUse64([u8; 24]),
    // ...
    /// 254 bytes of data
    #[br(magic(0x7F00u16))]
    ReservedForAppleUse65(#[cfg_attr(feature = "serde", serde(with = "BigArray"))] [u8; 254]),
    // ...
    #[br(magic(0x7FFFu16))]
    ReservedForAppleUse66(#[cfg_attr(feature = "serde", serde(with = "BigArray"))] [u8; 254]),
    #[br(magic(0x8000u16))]
    ReservedForAppleUse67,
    // ...
    #[br(magic(0x80FFu16))]
    ReservedForAppleUse68,
    /// Data length (Long), data
    #[br(magic(0x8100u16))]
    ReservedForAppleUse69(Long, #[br(count(self_0))] Vec<u8>),
    // ...
    /// Data length (Long), data (private to QuickTime)
    #[br(magic(0x8200u16))]
    CompressedQuickTime(Long, #[br(count(self_0))] Vec<u8>),
    /// Data length (Long), data (private to QuickTime)
    #[br(magic(0x8201u16))]
    UncompressedQuickTime(Long, #[br(count(self_0))] Vec<u8>),
    /// Data length (Long), data
    #[br(magic(0xFFFFu16))]
    ReservedForAppleUse70(Long, #[br(count(self_0))] Vec<u8>),

    Unknown(u16),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DirectBits {
    pub pix_map: PixMap,
    pub src_rect: Rect,
    pub dst_rect: Rect,
    pub mode: TransferMode,
    pub mask: Option<Region>,
    pub data: Vec<u8>,
}

impl BinRead for DirectBits {
    type Args<'a> = (bool,);

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        (masked,): Self::Args<'_>,
    ) -> BinResult<Self> {
        let _base_address: u32 = reader.read_be()?;
        let row_bytes_and_flags_hi: u8 = reader.read_be()?;
        let pix_map: PixMap = reader.read_be_args((row_bytes_and_flags_hi,))?;
        let src_rect = reader.read_be()?;
        let dst_rect = reader.read_be()?;
        let mode = reader.read_be()?;
        let mask = if masked {
            Some(reader.read_be()?)
        } else {
            None
        };
        let data = reader.read_be_args::<PixMapData>(&pix_map)?;

        Ok(DirectBits {
            pix_map,
            src_rect,
            dst_rect,
            mode,
            mask,
            data: data.into(),
        })
    }
}
