//! PICT version 1 format opcodes and structures.
//!
//! PICT v1 is the original QuickDraw picture format used in early versions
//! of Mac OS. This module defines the opcodes and parsing logic for v1 files.

use binrw::prelude::*;

use crate::PictVersion;

use super::shared::*;

/// PICT version 1 opcode enumeration.
///
/// Each variant represents a QuickDraw drawing command from the original
/// Macintosh imaging model.
#[derive(BinRead, Debug, Clone)]
#[br(big)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Opcode {
    /// No operation
    #[br(magic = 0x00u8)]
    NOP,
    /// Clipping region
    #[br(magic = 0x01u8)]
    ClipRgn(Region),
    /// Background pattern
    #[br(magic = 0x02u8)]
    BkPat(Pattern),
    /// Font number for text
    #[br(magic = 0x03u8)]
    TxFont(u16),
    /// Text's font style
    #[br(magic = 0x04u8)]
    TxFace(u8),
    /// Source mode
    #[br(magic = 0x05u8)]
    TxMode(u16),
    /// Extra space (Fixed)
    #[br(magic = 0x06u8)]
    SpExtra(u32),
    /// Pen size
    #[br(magic = 0x07u8)]
    PnSize(Point),
    /// Pen mode
    #[br(magic = 0x08u8)]
    PnMode(u16),
    /// Pen pattern
    #[br(magic = 0x09u8)]
    PnPat(Pattern),
    /// Fill pattern
    #[br(magic = 0x0Au8)]
    FillPat(Pattern),
    /// Oval size
    #[br(magic = 0x0Bu8)]
    OvSize(Point),
    /// dh (u16), dv (u16)
    #[br(magic = 0x0Cu8)]
    Origin(u16, u16),
    /// Text size
    #[br(magic = 0x0Du8)]
    TxSize(u16),
    /// Foreground color
    #[br(magic = 0x0Eu8)]
    FgColor(u32),
    /// Background color
    #[br(magic = 0x0Fu8)]
    BkColor(u32),
    /// Numerator (Point), denominator (Point)
    #[br(magic = 0x10u8)]
    TxRatio(Point, Point),
    /// Version
    #[br(magic = 0x11u8)]
    PicVersion(u8),
    /// pnLoc (Point), newPt (Point)
    #[br(magic = 0x20u8)]
    Line(Point, Point),
    /// newPt (Point)
    #[br(magic = 0x21u8)]
    LineFrom(Point),
    /// pnLoc (Point), dh (-128..127), dv (-128..127)
    #[br(magic = 0x22u8)]
    ShortLine(Point, i8, i8),
    /// dh (-128..127), dv (-128..127)
    #[br(magic = 0x23u8)]
    ShortLineFrom(i8, i8),
    /// txLoc (Point), count (0..255), text
    #[br(magic = 0x28u8)]
    LongText(Point, #[br(map(|x:ShortString| x.into()))] String),
    /// dh (0..255), count (0..255), text              | 2 + text                                 |
    #[br(magic = 0x29u8)]
    DHText(u8, #[br(map(|x:ShortString| x.into()))] String),
    /// dv (0..255), count (0..255), text              | 2 + text                                 |
    #[br(magic = 0x2Au8)]
    DVText(u8, #[br(map(|x:ShortString| x.into()))] String),
    /// dh (0..255), dv (0..255), count (0..255), text | 3 + text                                 |
    #[br(magic = 0x2Bu8)]
    DHDVText(u8, u8, #[br(map(|x:ShortString| x.into()))] String),

    /// Set current font
    #[br(magic = 0x2Cu8)]
    SetFont(u16, u16, #[br(map(|x:ShortString| x.into()))] String),

    /// Set glpyh state font
    #[br(magic = 0x2Eu8)]
    GlpyhState(u16, u8, u8, u8, u8),

    /// Rectangle (Rect)
    #[br(magic = 0x30u8)]
    FrameRect(Rect),
    /// Rectangle (Rect)
    #[br(magic = 0x31u8)]
    PaintRect(Rect),
    /// Rectangle (Rect)
    #[br(magic = 0x32u8)]
    EraseRect(Rect),
    /// Rectangle (Rect)
    #[br(magic = 0x33u8)]
    InvertRect(Rect),
    /// Rectangle (Rect)
    #[br(magic = 0x34u8)]
    FillRect(Rect),
    /// Rectangle (Rect)                               | 0                                        |
    #[br(magic = 0x38u8)]
    FrameSameRect,
    /// Rectangle (Rect)                               | 0                                        |
    #[br(magic = 0x39u8)]
    PaintSameRect,
    /// Rectangle (Rect)                               | 0                                        |
    #[br(magic = 0x3Au8)]
    EraseSameRect,
    /// Rectangle (Rect)                               | 0                                        |
    #[br(magic = 0x3Bu8)]
    InvertSameRect,
    /// Rectangle (Rect)                               | 0                                        |
    #[br(magic = 0x3Cu8)]
    FillSameRect,
    /// Rectangle (Rect)
    #[br(magic = 0x40u8)]
    FrameRRect(Rect),
    /// Rectangle (Rect)
    #[br(magic = 0x41u8)]
    PaintRRect(Rect),
    /// Rectangle (Rect)
    #[br(magic = 0x42u8)]
    EraseRRect(Rect),
    /// Rectangle (Rect)
    #[br(magic = 0x43u8)]
    InvertRRect(Rect),
    /// Rectangle (Rect)
    #[br(magic = 0x44u8)]
    FillRRect(Rect),
    /// Rectangle (Rect)                               | 0                                        |
    #[br(magic = 0x48u8)]
    FrameSameRRect,
    /// Rectangle (Rect)                               | 0                                        |
    #[br(magic = 0x49u8)]
    PaintSameRRect,
    /// Rectangle (Rect)                               | 0                                        |
    #[br(magic = 0x4Au8)]
    EraseSameRRect,
    /// Rectangle (Rect)                               | 0                                        |
    #[br(magic = 0x4Bu8)]
    InvertSameRRect,
    /// Rectangle (Rect)                               | 0                                        |
    #[br(magic = 0x4Cu8)]
    FillSameRRect,
    /// Rectangle (Rect)
    #[br(magic = 0x50u8)]
    FrameOval(Rect),
    /// Rectangle (Rect)
    #[br(magic = 0x51u8)]
    PaintOval(Rect),
    /// Rectangle (Rect)
    #[br(magic = 0x52u8)]
    EraseOval(Rect),
    /// Rectangle (Rect)
    #[br(magic = 0x53u8)]
    InvertOval(Rect),
    /// Rectangle (Rect)
    #[br(magic = 0x54u8)]
    FillOval(Rect),
    /// Rectangle (Rect)
    #[br(magic = 0x58u8)]
    FrameSameOval,
    /// Rectangle (Rect)
    #[br(magic = 0x59u8)]
    PaintSameOval,
    /// Rectangle (Rect)
    #[br(magic = 0x5Au8)]
    EraseSameOval,
    /// Rectangle (Rect)
    #[br(magic = 0x5Bu8)]
    InvertSameOval,
    /// Rectangle (Rect)
    #[br(magic = 0x5Cu8)]
    FillSameOval,
    /// Rectangle (Rect), startAngle, arcAngle
    #[br(magic = 0x60u8)]
    FrameArc(Rect, Angle, Angle),
    /// Rectangle (Rect), startAngle, arcAngle
    #[br(magic = 0x61u8)]
    PaintArc(Rect, Angle, Angle),
    /// Rectangle (Rect), startAngle, arcAngle
    #[br(magic = 0x62u8)]
    EraseArc(Rect, Angle, Angle),
    /// Rectangle (Rect), startAngle, arcAngle
    #[br(magic = 0x63u8)]
    InvertArc(Rect, Angle, Angle),
    /// Rectangle (Rect), startAngle, arcAngle
    #[br(magic = 0x64u8)]
    FillArc(Rect, Angle, Angle),
    /// Rectangle (Rect)                               | 4                                        |
    #[br(magic = 0x68u8)]
    FrameSameArc(Angle, Angle),
    /// Rectangle (Rect)                               | 4                                        |
    #[br(magic = 0x69u8)]
    PaintSameArc(Angle, Angle),
    /// Rectangle (Rect)                               | 4                                        |
    #[br(magic = 0x6Au8)]
    EraseSameArc(Angle, Angle),
    /// Rectangle (Rect)                               | 4                                        |
    #[br(magic = 0x6Bu8)]
    InvertSameArc(Angle, Angle),
    /// Rectangle (Rect)                               | 4                                        |
    #[br(magic = 0x6Cu8)]
    FillSameArc(Angle, Angle),
    /// Polygon (Poly)                      
    #[br(magic = 0x70u8)]
    FramePoly(Polygon),
    /// Polygon (Poly)                      
    #[br(magic = 0x71u8)]
    PaintPoly(Polygon),
    /// Polygon (Poly)                      
    #[br(magic = 0x72u8)]
    ErasePoly(Polygon),
    /// Polygon (Poly)                      
    #[br(magic = 0x73u8)]
    InvertPoly(Polygon),
    /// Polygon (Poly)                      
    #[br(magic = 0x74u8)]
    FillPoly(Polygon),
    /// (Not yet implemented)
    #[br(magic = 0x78u8)]
    FrameSamePoly,
    /// (Not yet implemented)
    #[br(magic = 0x79u8)]
    PaintSamePoly,
    /// (Not yet implemented)
    #[br(magic = 0x7Au8)]
    EraseSamePoly,
    /// (Not yet implemented)
    #[br(magic = 0x7Bu8)]
    InvertSamePoly,
    /// (Not yet implemented)
    #[br(magic = 0x7Cu8)]
    FillSamePoly,
    /// Region (Rgn)                        
    #[br(magic = 0x80u8)]
    FrameRgn(Region),
    /// Region (Rgn)                        
    #[br(magic = 0x81u8)]
    PaintRgn(Region),
    /// Region (Rgn)                        
    #[br(magic = 0x82u8)]
    EraseRgn(Region),
    /// Region (Rgn)                        
    #[br(magic = 0x83u8)]
    InvertRgn(Region),
    /// Region (Rgn)                        
    #[br(magic = 0x84u8)]
    FillRgn(Region),
    /// (Not yet implemented)
    #[br(magic = 0x88u8)]
    FrameSameRgn,
    /// (Not yet implemented)
    #[br(magic = 0x89u8)]
    PaintSameRgn,
    /// (Not yet implemented)
    #[br(magic = 0x8Au8)]
    EraseSameRgn,
    /// (Not yet implemented)
    #[br(magic = 0x8Bu8)]
    InvertSameRgn,
    /// (Not yet implemented)
    #[br(magic = 0x8Cu8)]
    FillSameRgn,
    /// CopyBits with clipped rectangle
    #[br(magic = 0x90u8)]
    BitsRect(#[br(args(false, false, PictVersion::Version1(0)))] CopyBits),
    /// CopyBits with clipped region
    #[br(magic = 0x91u8)]
    BitsRgn(#[br(args(false, true, PictVersion::Version1(0)))] CopyBits),
    /// Packed CopyBits with clipped rectangle
    #[br(magic = 0x98u8)]
    PackBitsRect(#[br(args(true, false, PictVersion::Version1(0)))] CopyBits),
    /// Packed CopyBits with clipped rectangle
    #[br(magic = 0x99u8)]
    PackBitsRgn(#[br(args(true, true, PictVersion::Version1(0)))] CopyBits),
    /// Kind (u16)
    #[br(magic = 0xA0u8)]
    ShortComment(CommentKind),
    /// Kind (u16), size (u16), data
    #[br(magic = 0xA1u8)]
    LongComment(CommentKind, u16, #[br(count(self_1))] Vec<u8>),

    /// End of picture
    #[br(magic = 0xFFu8)]
    EndOfPicture,

    Unknown(u8),
}
