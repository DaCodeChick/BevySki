//! Drawing context for rendering PICT operations to an image buffer.
//!
//! This module provides the core rendering engine that interprets PICT opcodes
//! and draws them onto an RGBA image buffer.

use std::collections::HashSet;

use image::Rgba;

use crate::{
    shared::{self, ColorTable, PixMap, Rect, Region, TransferMode},
    v1, v2,
};

/// Main drawing context that maintains the canvas and rendering state.
///
/// Processes PICT v1 and v2 opcodes, managing clipping regions, color tables,
/// and various QuickDraw operations.
pub struct DrawingContext {
    /// The RGBA image buffer being drawn to.
    canvas: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    /// Bounding rectangle of the drawing area.
    bounds: Rect,
    /// Current clipping region.
    clipping_region: Region,
}

impl DrawingContext {
    /// Creates a new drawing context with the specified bounds.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The bounding rectangle for the canvas
    pub fn new(bounds: Rect) -> Self {
        if bounds.width() < 0 {
            log::warn!("Bounds has negative width");
        } else if bounds.height() < 0 {
            log::warn!("Bounds has negative height");
        }

        Self {
            canvas: image::ImageBuffer::new(
                bounds.width().try_into().unwrap_or(1),
                bounds.height().try_into().unwrap_or(1),
            ),
            bounds,
            clipping_region: Region::new(),
        }
    }

    /// Processes a PICT v1 opcode.
    ///
    /// # Returns
    ///
    /// `true` if this is an end-of-picture opcode, `false` otherwise.
    pub fn command_v1(&mut self, cmd: v1::Opcode) -> bool {
        match cmd {
            v1::Opcode::NOP => log::info!("v1::Opcode::NOP"),
            v1::Opcode::ClipRgn(r) => {
                log::trace!("v1::Opcode::ClipRgn(..)");
                self.clip_region(r);
            }
            v1::Opcode::BkPat(_) => log::info!("v1::Opcode::BkPat"),
            v1::Opcode::TxFont(_) => log::info!("v1::Opcode::TxFont"),
            v1::Opcode::TxFace(_) => log::info!("v1::Opcode::TxFace"),
            v1::Opcode::TxMode(_) => log::info!("v1::Opcode::TxMode"),
            v1::Opcode::SpExtra(_) => log::info!("v1::Opcode::SpExtra"),
            v1::Opcode::PnSize(_) => log::info!("v1::Opcode::PnSize"),
            v1::Opcode::PnMode(_) => log::info!("v1::Opcode::PnMode"),
            v1::Opcode::PnPat(_) => log::info!("v1::Opcode::PnPat"),
            v1::Opcode::FillPat(_) => log::info!("v1::Opcode::FillPat"),
            v1::Opcode::OvSize(_) => log::info!("v1::Opcode::OvSize"),
            v1::Opcode::Origin(_, _) => log::info!("v1::Opcode::Origin"),
            v1::Opcode::TxSize(_) => log::info!("v1::Opcode::TxSize"),
            v1::Opcode::FgColor(_) => log::info!("v1::Opcode::FgColor"),
            v1::Opcode::BkColor(_) => log::info!("v1::Opcode::BkColor"),
            v1::Opcode::TxRatio(_, _) => log::info!("v1::Opcode::TxRatio"),
            v1::Opcode::PicVersion(_) => log::trace!("v1::Opcode::PicVersion"),
            v1::Opcode::Line(_, _) => log::info!("v1::Opcode::Line"),
            v1::Opcode::LineFrom(_) => log::info!("v1::Opcode::LineFrom"),
            v1::Opcode::ShortLine(_, _, _) => log::info!("v1::Opcode::ShortLine"),
            v1::Opcode::ShortLineFrom(_, _) => {
                log::info!("v1::Opcode::ShortLineFrom")
            }
            v1::Opcode::LongText(_, _) => log::info!("v1::Opcode::LongText"),
            v1::Opcode::DHText(_, _) => log::info!("v1::Opcode::DHText"),
            v1::Opcode::DVText(_, _) => log::info!("v1::Opcode::DVText"),
            v1::Opcode::DHDVText(_, _, _) => log::info!("v1::Opcode::DHDVText"),
            v1::Opcode::FrameRect(_) => log::info!("v1::Opcode::FrameRect"),
            v1::Opcode::PaintRect(_) => log::info!("v1::Opcode::PaintRect"),
            v1::Opcode::EraseRect(_) => log::info!("v1::Opcode::EraseRect"),
            v1::Opcode::InvertRect(_) => log::info!("v1::Opcode::InvertRect"),
            v1::Opcode::FillRect(_) => log::info!("v1::Opcode::FillRect"),
            v1::Opcode::FrameSameRect => log::info!("v1::Opcode::FrameSameRect"),
            v1::Opcode::PaintSameRect => log::info!("v1::Opcode::PaintSameRect"),
            v1::Opcode::EraseSameRect => log::info!("v1::Opcode::EraseSameRect"),
            v1::Opcode::InvertSameRect => log::info!("v1::Opcode::InvertSameRect"),
            v1::Opcode::FillSameRect => log::info!("v1::Opcode::FillSameRect"),
            v1::Opcode::FrameRRect(_) => log::info!("v1::Opcode::FrameRRect"),
            v1::Opcode::PaintRRect(_) => log::info!("v1::Opcode::PaintRRect"),
            v1::Opcode::EraseRRect(_) => log::info!("v1::Opcode::EraseRRect"),
            v1::Opcode::InvertRRect(_) => log::info!("v1::Opcode::InvertRRect"),
            v1::Opcode::FillRRect(_) => log::info!("v1::Opcode::FillRRect"),
            v1::Opcode::FrameSameRRect => log::info!("v1::Opcode::FrameSameRRect"),
            v1::Opcode::PaintSameRRect => log::info!("v1::Opcode::PaintSameRRect"),
            v1::Opcode::EraseSameRRect => log::info!("v1::Opcode::EraseSameRRect"),
            v1::Opcode::InvertSameRRect => log::info!("v1::Opcode::InvertSameRRect"),
            v1::Opcode::FillSameRRect => log::info!("v1::Opcode::FillSameRRect"),
            v1::Opcode::FrameOval(_) => log::info!("v1::Opcode::FrameOval"),
            v1::Opcode::PaintOval(_) => log::info!("v1::Opcode::PaintOval"),
            v1::Opcode::EraseOval(_) => log::info!("v1::Opcode::EraseOval"),
            v1::Opcode::InvertOval(_) => log::info!("v1::Opcode::InvertOval"),
            v1::Opcode::FillOval(_) => log::info!("v1::Opcode::FillOval"),
            v1::Opcode::FrameSameOval => log::info!("v1::Opcode::FrameSameOval"),
            v1::Opcode::PaintSameOval => log::info!("v1::Opcode::PaintSameOval"),
            v1::Opcode::EraseSameOval => log::info!("v1::Opcode::EraseSameOval"),
            v1::Opcode::InvertSameOval => log::info!("v1::Opcode::InvertSameOval"),
            v1::Opcode::FillSameOval => log::info!("v1::Opcode::FillSameOval"),
            v1::Opcode::FrameArc(_, _, _) => log::info!("v1::Opcode::FrameArc"),
            v1::Opcode::PaintArc(_, _, _) => log::info!("v1::Opcode::PaintArc"),
            v1::Opcode::EraseArc(_, _, _) => log::info!("v1::Opcode::EraseArc"),
            v1::Opcode::InvertArc(_, _, _) => log::info!("v1::Opcode::InvertArc"),
            v1::Opcode::FillArc(_, _, _) => log::info!("v1::Opcode::FillArc"),
            v1::Opcode::FrameSameArc(_, _) => log::info!("v1::Opcode::FrameSameArc"),
            v1::Opcode::PaintSameArc(_, _) => log::info!("v1::Opcode::PaintSameArc"),
            v1::Opcode::EraseSameArc(_, _) => log::info!("v1::Opcode::EraseSameArc"),
            v1::Opcode::InvertSameArc(_, _) => {
                log::info!("v1::Opcode::InvertSameArc")
            }
            v1::Opcode::FillSameArc(_, _) => log::info!("v1::Opcode::FillSameArc"),
            v1::Opcode::FramePoly(_) => log::info!("v1::Opcode::FramePoly"),
            v1::Opcode::PaintPoly(_) => log::info!("v1::Opcode::PaintPoly"),
            v1::Opcode::ErasePoly(_) => log::info!("v1::Opcode::ErasePoly"),
            v1::Opcode::InvertPoly(_) => log::info!("v1::Opcode::InvertPoly"),
            v1::Opcode::FillPoly(_) => log::info!("v1::Opcode::FillPoly"),
            v1::Opcode::FrameSamePoly => log::info!("v1::Opcode::FrameSamePoly"),
            v1::Opcode::PaintSamePoly => log::info!("v1::Opcode::PaintSamePoly"),
            v1::Opcode::EraseSamePoly => log::info!("v1::Opcode::EraseSamePoly"),
            v1::Opcode::InvertSamePoly => log::info!("v1::Opcode::InvertSamePoly"),
            v1::Opcode::FillSamePoly => log::info!("v1::Opcode::FillSamePoly"),
            v1::Opcode::FrameRgn(_) => log::info!("v1::Opcode::FrameRgn"),
            v1::Opcode::PaintRgn(_) => log::info!("v1::Opcode::PaintRgn"),
            v1::Opcode::EraseRgn(_) => log::info!("v1::Opcode::EraseRgn"),
            v1::Opcode::InvertRgn(_) => log::info!("v1::Opcode::InvertRgn"),
            v1::Opcode::FillRgn(_) => log::info!("v1::Opcode::FillRgn"),
            v1::Opcode::FrameSameRgn => log::info!("v1::Opcode::FrameSameRgn"),
            v1::Opcode::PaintSameRgn => log::info!("v1::Opcode::PaintSameRgn"),
            v1::Opcode::EraseSameRgn => log::info!("v1::Opcode::EraseSameRgn"),
            v1::Opcode::InvertSameRgn => log::info!("v1::Opcode::InvertSameRgn"),
            v1::Opcode::FillSameRgn => log::info!("v1::Opcode::FillSameRgn"),
            v1::Opcode::BitsRect(bits) => {
                log::trace!("v1::Opcode::BitsRect");
                match bits {
                    shared::CopyBits::Pixmap {
                        pix_map,
                        color_table,
                        src_rect,
                        dst_rect,
                        mode,
                        mask_region,
                        data,
                    } => {
                        self.apply_pixmap(
                            &pix_map,
                            color_table,
                            src_rect,
                            dst_rect,
                            mode,
                            mask_region,
                            data,
                        );
                    }
                    shared::CopyBits::Bitmap {
                        bytes_per_row,
                        bounds,
                        src_rect,
                        dst_rect,
                        data,
                        mask_region,
                        mode,
                    } => {
                        self.apply_bitmap(
                            src_rect,
                            dst_rect,
                            mode,
                            bounds,
                            bytes_per_row as usize,
                            data,
                            mask_region,
                        );
                    }
                }
            }
            v1::Opcode::BitsRgn(bits) => {
                log::trace!("v1::Opcode::BitsRgn");
                match bits {
                    shared::CopyBits::Pixmap {
                        pix_map,
                        color_table,
                        src_rect,
                        dst_rect,
                        mode,
                        mask_region,
                        data,
                    } => {
                        self.apply_pixmap(
                            &pix_map,
                            color_table,
                            src_rect,
                            dst_rect,
                            mode,
                            mask_region,
                            data,
                        );
                    }
                    shared::CopyBits::Bitmap {
                        bytes_per_row,
                        bounds,
                        src_rect,
                        dst_rect,
                        data,
                        mask_region,
                        mode,
                    } => {
                        self.apply_bitmap(
                            src_rect,
                            dst_rect,
                            mode,
                            bounds,
                            bytes_per_row as usize,
                            data,
                            mask_region,
                        );
                    }
                }
            }
            v1::Opcode::PackBitsRect(copy_bits) => {
                log::trace!("v1::Opcode::PackBitsRect(..)");
                match copy_bits {
                    shared::CopyBits::Pixmap {
                        pix_map,
                        color_table,
                        src_rect,
                        dst_rect,
                        mode,
                        mask_region,
                        data,
                    } => {
                        self.apply_pixmap(
                            &pix_map,
                            color_table,
                            src_rect,
                            dst_rect,
                            mode,
                            mask_region,
                            data,
                        );
                    }
                    shared::CopyBits::Bitmap {
                        bytes_per_row,
                        bounds,
                        src_rect,
                        dst_rect,
                        data,
                        mask_region,
                        mode,
                    } => {
                        self.apply_bitmap(
                            src_rect,
                            dst_rect,
                            mode,
                            bounds,
                            bytes_per_row as usize,
                            data,
                            mask_region,
                        );
                    }
                }
            }
            v1::Opcode::PackBitsRgn(packbits) => {
                log::info!("v1::Opcode::PackBitsRgn(..)");
                match packbits {
                    shared::CopyBits::Pixmap {
                        pix_map,
                        color_table,
                        src_rect,
                        dst_rect,
                        mode,
                        mask_region,
                        data,
                    } => {
                        self.apply_pixmap(
                            &pix_map,
                            color_table,
                            src_rect,
                            dst_rect,
                            mode,
                            mask_region,
                            data,
                        );
                    }
                    shared::CopyBits::Bitmap {
                        bytes_per_row,
                        bounds,
                        src_rect,
                        dst_rect,
                        data,
                        mask_region,
                        mode,
                    } => {
                        self.apply_bitmap(
                            src_rect,
                            dst_rect,
                            mode,
                            bounds,
                            bytes_per_row as usize,
                            data,
                            mask_region,
                        );
                    }
                }
            }
            v1::Opcode::ShortComment(v) => log::trace!("v1::Opcode::ShortComment({v:?})"),
            v1::Opcode::LongComment(v, _, _) => {
                log::trace!("v1::Opcode::LongComment({v:?}, ..)")
            }
            v1::Opcode::EndOfPicture => {
                log::trace!("v1::Opcode::EndOfPicture");
                return true;
            }
            v1::Opcode::Unknown(_) => {
                log::info!("v1::Opcode::Unknown");
                return true;
            }
            v1::Opcode::SetFont(_, _, _) => {
                log::trace!("v1::Opcode::SetFont");
            }
            v1::Opcode::GlpyhState(_, _, _, _, _) => {
                log::trace!("v1::Opcode::GlpyhState");
            }
        }

        false
    }

    /// Processes a PICT v2 opcode.
    ///
    /// # Returns
    ///
    /// `true` if this is an end-of-picture opcode, `false` otherwise.
    pub fn command_v2(&mut self, cmd: v2::Opcode) -> bool {
        match cmd {
            v2::Opcode::NOP => log::info!("v2::Opcode::NOP"),
            v2::Opcode::Clip(rgn) => {
                log::trace!("v2::Opcode::Clip({rgn:?})");
                self.clip_region(rgn);
            }
            v2::Opcode::BkPat(_) => log::info!("v2::Opcode::BkPat"),
            v2::Opcode::TxFont(_) => log::info!("v2::Opcode::TxFont"),
            v2::Opcode::TxFace(_) => log::info!("v2::Opcode::TxFace"),
            v2::Opcode::TxMode(_) => log::info!("v2::Opcode::TxMode"),
            v2::Opcode::SpExtra(_) => log::info!("v2::Opcode::SpExtra"),
            v2::Opcode::PnSize(_) => log::info!("v2::Opcode::PnSize"),
            v2::Opcode::PnMode(_) => log::info!("v2::Opcode::PnMode"),
            v2::Opcode::PnPat(_) => log::info!("v2::Opcode::PnPat"),
            v2::Opcode::FillPat(_) => log::info!("v2::Opcode::FillPat"),
            v2::Opcode::OvSize(_) => log::info!("v2::Opcode::OvSize"),
            v2::Opcode::Origin(_, _) => log::info!("v2::Opcode::Origin"),
            v2::Opcode::TxSize(_) => log::info!("v2::Opcode::TxSize"),
            v2::Opcode::FgColor(_) => log::info!("v2::Opcode::FgColor"),
            v2::Opcode::BkColor(_) => log::info!("v2::Opcode::BkColor"),
            v2::Opcode::TxRatio(_, _) => log::info!("v2::Opcode::TxRatio"),
            v2::Opcode::VersionOp => log::trace!("v2::Opcode::VersionOp"),
            v2::Opcode::BkPixPat(_) => log::info!("v2::Opcode::BkPixPat"),
            v2::Opcode::PnPixPat(_) => log::info!("v2::Opcode::PnPixPat"),
            v2::Opcode::FillPixPat(_) => log::info!("v2::Opcode::FillPixPat"),
            v2::Opcode::PnLocHFrac(_) => log::info!("v2::Opcode::PnLocHFrac"),
            v2::Opcode::ChExtra(_) => log::info!("v2::Opcode::ChExtra"),
            v2::Opcode::ReservedForAppleUse => {
                log::info!("v2::Opcode::ReservedForAppleUse")
            }
            v2::Opcode::ReservedForAppleUse1 => {
                log::info!("v2::Opcode::ReservedForAppleUse1")
            }
            v2::Opcode::ReservedForAppleUse2 => {
                log::info!("v2::Opcode::ReservedForAppleUse2")
            }
            v2::Opcode::RGBFgColor(_) => log::info!("v2::Opcode::RGBFgColor"),
            v2::Opcode::RGBBkCol(_) => log::info!("v2::Opcode::RGBBkCol"),
            v2::Opcode::HiliteMode => log::info!("v2::Opcode::HiliteMode"),
            v2::Opcode::HiliteColor(_) => {
                log::info!("v2::Opcode::HiliteColor")
            }
            v2::Opcode::DefHilite => log::trace!("v2::Opcode::DefHilite"),
            v2::Opcode::OpColor(_) => log::info!("v2::Opcode::OpColor"),
            v2::Opcode::Line(_, _) => log::info!("v2::Opcode::Line"),
            v2::Opcode::LineFrom(_) => log::info!("v2::Opcode::LineFrom"),
            v2::Opcode::ShortLine(_, _, _) => log::info!("v2::Opcode::ShortLine"),
            v2::Opcode::ShortLineFrom(_, _) => {
                log::info!("v2::Opcode::ShortLineFrom")
            }
            v2::Opcode::ReservedForAppleUse3(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse3")
            }
            v2::Opcode::ReservedForAppleUse4(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse4")
            }
            v2::Opcode::ReservedForAppleUse5(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse5")
            }
            v2::Opcode::ReservedForAppleUse6(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse6")
            }
            v2::Opcode::LongText(_, _) => {
                log::info!("v2::Opcode::LongText")
            }
            v2::Opcode::DHText(_, _) => log::info!("v2::Opcode::DHText"),
            v2::Opcode::DVText(_, _) => log::info!("v2::Opcode::DVText"),
            v2::Opcode::DHDVText(_, _, _) => {
                log::info!("v2::Opcode::DHDVText")
            }
            v2::Opcode::FontName(_, _, _) => {
                log::info!("v2::Opcode::FontName")
            }
            v2::Opcode::LineJustify(_, _, _) => {
                log::info!("v2::Opcode::LineJustify")
            }
            v2::Opcode::GlyphState(_, _, _, _, _) => {
                log::info!("v2::Opcode::GlyphState")
            }
            v2::Opcode::ReservedForAppleUse7(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse7")
            }
            v2::Opcode::FrameRect(_) => log::info!("v2::Opcode::FrameRect"),
            v2::Opcode::PaintRect(_) => log::info!("v2::Opcode::PaintRect"),
            v2::Opcode::EraseRect(_) => log::info!("v2::Opcode::EraseRect"),
            v2::Opcode::InvertRect(_) => log::info!("v2::Opcode::InvertRect"),
            v2::Opcode::FillRect(_) => log::info!("v2::Opcode::FillRect"),
            v2::Opcode::ReservedForAppleUse8(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse8")
            }
            v2::Opcode::ReservedForAppleUse9(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse9")
            }
            v2::Opcode::ReservedForAppleUse10(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse10")
            }
            v2::Opcode::FrameSameRect => log::info!("v2::Opcode::FrameSameRect"),
            v2::Opcode::PaintSameRect => log::info!("v2::Opcode::PaintSameRect"),
            v2::Opcode::EraseSameRect => log::info!("v2::Opcode::EraseSameRect"),
            v2::Opcode::InvertSameRect => log::info!("v2::Opcode::InvertSameRect"),
            v2::Opcode::FillSameRect => log::info!("v2::Opcode::FillSameRect"),
            v2::Opcode::ReservedForAppleUse11 => {
                log::info!("v2::Opcode::ReservedForAppleUse11")
            }
            v2::Opcode::ReservedForAppleUse12 => {
                log::info!("v2::Opcode::ReservedForAppleUse12")
            }
            v2::Opcode::ReservedForAppleUse13 => {
                log::info!("v2::Opcode::ReservedForAppleUse13")
            }
            v2::Opcode::FrameRRect(_) => log::info!("v2::Opcode::FrameRRect"),
            v2::Opcode::PaintRRect(_) => log::info!("v2::Opcode::PaintRRect"),
            v2::Opcode::EraseRRect(_) => log::info!("v2::Opcode::EraseRRect"),
            v2::Opcode::InvertRRect(_) => log::info!("v2::Opcode::InvertRRect"),
            v2::Opcode::FillRRect(_) => log::info!("v2::Opcode::FillRRect"),
            v2::Opcode::ReservedForAppleUse14(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse14")
            }
            v2::Opcode::ReservedForAppleUse15(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse15")
            }
            v2::Opcode::ReservedForAppleUse16(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse16")
            }
            v2::Opcode::FrameSameRRect => log::info!("v2::Opcode::FrameSameRRect"),
            v2::Opcode::PaintSameRRect => log::info!("v2::Opcode::PaintSameRRect"),
            v2::Opcode::EraseSameRRect => log::info!("v2::Opcode::EraseSameRRect"),
            v2::Opcode::InvertSameRRect => log::info!("v2::Opcode::InvertSameRRect"),
            v2::Opcode::FillSameRRect => log::info!("v2::Opcode::FillSameRRect"),
            v2::Opcode::ReservedForAppleUse17 => {
                log::info!("v2::Opcode::ReservedForAppleUse17")
            }
            v2::Opcode::ReservedForAppleUse18 => {
                log::info!("v2::Opcode::ReservedForAppleUse18")
            }
            v2::Opcode::ReservedForAppleUse19 => {
                log::info!("v2::Opcode::ReservedForAppleUse19")
            }
            v2::Opcode::FrameOval(_) => log::info!("v2::Opcode::FrameOval"),
            v2::Opcode::PaintOval(_) => log::info!("v2::Opcode::PaintOval"),
            v2::Opcode::EraseOval(_) => log::info!("v2::Opcode::EraseOval"),
            v2::Opcode::InvertOval(_) => log::info!("v2::Opcode::InvertOval"),
            v2::Opcode::FillOval(_) => log::info!("v2::Opcode::FillOval"),
            v2::Opcode::ReservedForAppleUse20(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse20")
            }
            v2::Opcode::ReservedForAppleUse21(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse21")
            }
            v2::Opcode::ReservedForAppleUse22(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse22")
            }
            v2::Opcode::FrameSameOval => log::info!("v2::Opcode::FrameSameOval"),
            v2::Opcode::PaintSameOval => log::info!("v2::Opcode::PaintSameOval"),
            v2::Opcode::EraseSameOval => log::info!("v2::Opcode::EraseSameOval"),
            v2::Opcode::InvertSameOval => log::info!("v2::Opcode::InvertSameOval"),
            v2::Opcode::FillSameOval => log::info!("v2::Opcode::FillSameOval"),
            v2::Opcode::ReservedForAppleUse23 => {
                log::info!("v2::Opcode::ReservedForAppleUse23")
            }
            v2::Opcode::ReservedForAppleUse24 => {
                log::info!("v2::Opcode::ReservedForAppleUse24")
            }
            v2::Opcode::ReservedForAppleUse25 => {
                log::info!("v2::Opcode::ReservedForAppleUse25")
            }
            v2::Opcode::FrameArc(_, _, _) => log::info!("v2::Opcode::FrameArc"),
            v2::Opcode::PaintArc(_, _, _) => log::info!("v2::Opcode::PaintArc"),
            v2::Opcode::EraseArc(_, _, _) => log::info!("v2::Opcode::EraseArc"),
            v2::Opcode::InvertArc(_, _, _) => log::info!("v2::Opcode::InvertArc"),
            v2::Opcode::FillArc(_, _, _) => log::info!("v2::Opcode::FillArc"),
            v2::Opcode::ReservedForAppleUse26(_, _, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse26")
            }
            v2::Opcode::ReservedForAppleUse27(_, _, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse27")
            }
            v2::Opcode::ReservedForAppleUse28(_, _, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse28")
            }
            v2::Opcode::FrameSameArc(_, _) => log::info!("v2::Opcode::FrameSameArc"),
            v2::Opcode::PaintSameArc(_, _) => log::info!("v2::Opcode::PaintSameArc"),
            v2::Opcode::EraseSameArc(_, _) => log::info!("v2::Opcode::EraseSameArc"),
            v2::Opcode::InvertSameArc(_, _) => {
                log::info!("v2::Opcode::InvertSameArc")
            }
            v2::Opcode::FillSameArc(_, _) => log::info!("v2::Opcode::FillSameArc"),
            v2::Opcode::ReservedForAppleUse29(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse29")
            }
            v2::Opcode::ReservedForAppleUse30(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse30")
            }
            v2::Opcode::ReservedForAppleUse31(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse31")
            }
            v2::Opcode::FramePoly(_) => log::info!("v2::Opcode::FramePoly"),
            v2::Opcode::PaintPoly(_) => log::info!("v2::Opcode::PaintPoly"),
            v2::Opcode::ErasePoly(_) => log::info!("v2::Opcode::ErasePoly"),
            v2::Opcode::InvertPoly(_) => log::info!("v2::Opcode::InvertPoly"),
            v2::Opcode::FillPoly(_) => log::info!("v2::Opcode::FillPoly"),
            v2::Opcode::ReservedForAppleUse32(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse32")
            }
            v2::Opcode::ReservedForAppleUse33(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse33")
            }
            v2::Opcode::ReservedForAppleUse34(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse34")
            }
            v2::Opcode::FrameSamePoly => log::info!("v2::Opcode::FrameSamePoly"),
            v2::Opcode::PaintSamePoly => log::info!("v2::Opcode::PaintSamePoly"),
            v2::Opcode::EraseSamePoly => log::info!("v2::Opcode::EraseSamePoly"),
            v2::Opcode::InvertSamePoly => log::info!("v2::Opcode::InvertSamePoly"),
            v2::Opcode::FillSamePoly => log::info!("v2::Opcode::FillSamePoly"),
            v2::Opcode::ReservedForAppleUse35 => {
                log::info!("v2::Opcode::ReservedForAppleUse35")
            }
            v2::Opcode::ReservedForAppleUse36 => {
                log::info!("v2::Opcode::ReservedForAppleUse36")
            }
            v2::Opcode::ReservedForAppleUse37 => {
                log::info!("v2::Opcode::ReservedForAppleUse37")
            }
            v2::Opcode::FrameRgn(_) => log::info!("v2::Opcode::FrameRgn"),
            v2::Opcode::PaintRgn(_) => log::info!("v2::Opcode::PaintRgn"),
            v2::Opcode::EraseRgn(_) => log::info!("v2::Opcode::EraseRgn"),
            v2::Opcode::InvertRgn(_) => log::info!("v2::Opcode::InvertRgn"),
            v2::Opcode::FillRgn(_) => log::info!("v2::Opcode::FillRgn"),
            v2::Opcode::ReservedForAppleUse38(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse38")
            }
            v2::Opcode::ReservedForAppleUse39(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse39")
            }
            v2::Opcode::ReservedForAppleUse40(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse40")
            }
            v2::Opcode::FrameSameRgn => log::info!("v2::Opcode::FrameSameRgn"),
            v2::Opcode::PaintSameRgn => log::info!("v2::Opcode::PaintSameRgn"),
            v2::Opcode::EraseSameRgn => log::info!("v2::Opcode::EraseSameRgn"),
            v2::Opcode::InvertSameRgn => log::info!("v2::Opcode::InvertSameRgn"),
            v2::Opcode::FillSameRgn => log::info!("v2::Opcode::FillSameRgn"),
            v2::Opcode::ReservedForAppleUse41 => {
                log::info!("v2::Opcode::ReservedForAppleUse41")
            }
            v2::Opcode::ReservedForAppleUse42 => {
                log::info!("v2::Opcode::ReservedForAppleUse42")
            }
            v2::Opcode::ReservedForAppleUse43 => {
                log::info!("v2::Opcode::ReservedForAppleUse43")
            }
            v2::Opcode::BitsRect(bits) => {
                log::info!("v2::Opcode::BitsRect(..)");
                match bits {
                    shared::CopyBits::Pixmap {
                        pix_map,
                        color_table,
                        src_rect,
                        dst_rect,
                        mode,
                        mask_region,
                        data,
                    } => {
                        self.apply_pixmap(
                            &pix_map,
                            color_table,
                            src_rect,
                            dst_rect,
                            mode,
                            mask_region,
                            data,
                        );
                    }
                    shared::CopyBits::Bitmap {
                        bytes_per_row,
                        bounds,
                        src_rect,
                        dst_rect,
                        data,
                        mask_region,
                        mode,
                    } => {
                        self.apply_bitmap(
                            src_rect,
                            dst_rect,
                            mode,
                            bounds,
                            bytes_per_row as usize,
                            data,
                            mask_region,
                        );
                    }
                }
            }
            v2::Opcode::BitsRgn(bits) => {
                log::info!("v2::Opcode::BitsRgn(..)");
                match bits {
                    shared::CopyBits::Pixmap {
                        pix_map,
                        color_table,
                        src_rect,
                        dst_rect,
                        mode,
                        mask_region,
                        data,
                    } => {
                        self.apply_pixmap(
                            &pix_map,
                            color_table,
                            src_rect,
                            dst_rect,
                            mode,
                            mask_region,
                            data,
                        );
                    }
                    shared::CopyBits::Bitmap {
                        bytes_per_row,
                        bounds,
                        src_rect,
                        dst_rect,
                        data,
                        mask_region,
                        mode,
                    } => {
                        self.apply_bitmap(
                            src_rect,
                            dst_rect,
                            mode,
                            bounds,
                            bytes_per_row as usize,
                            data,
                            mask_region,
                        );
                    }
                }
            }
            v2::Opcode::ReservedForAppleUse44(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse44")
            }
            v2::Opcode::ReservedForAppleUse45(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse45")
            }
            v2::Opcode::ReservedForAppleUse46(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse46")
            }
            v2::Opcode::ReservedForAppleUse47(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse47")
            }
            v2::Opcode::ReservedForAppleUse48(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse48")
            }
            v2::Opcode::ReservedForAppleUse49(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse49")
            }
            v2::Opcode::PackBitsRect(copy_bits) => {
                log::trace!("v2::Opcode::PackBitsRect(..)");
                match copy_bits {
                    shared::CopyBits::Pixmap {
                        pix_map,
                        color_table,
                        src_rect,
                        dst_rect,
                        mode,
                        mask_region,
                        data,
                    } => {
                        self.apply_pixmap(
                            &pix_map,
                            color_table,
                            src_rect,
                            dst_rect,
                            mode,
                            mask_region,
                            data,
                        );
                    }
                    shared::CopyBits::Bitmap {
                        bytes_per_row,
                        bounds,
                        src_rect,
                        dst_rect,
                        data,
                        mask_region,
                        mode,
                    } => {
                        self.apply_bitmap(
                            src_rect,
                            dst_rect,
                            mode,
                            bounds,
                            bytes_per_row as usize,
                            data,
                            mask_region,
                        );
                    }
                }
            }
            v2::Opcode::PackBitsRgn(copy_bits) => {
                log::trace!("v2::Opcode::PackBitsRgn");
                match copy_bits {
                    shared::CopyBits::Pixmap {
                        pix_map,
                        color_table,
                        src_rect,
                        dst_rect,
                        mode,
                        mask_region,
                        data,
                    } => {
                        self.apply_pixmap(
                            &pix_map,
                            color_table,
                            src_rect,
                            dst_rect,
                            mode,
                            mask_region,
                            data,
                        );
                    }
                    shared::CopyBits::Bitmap {
                        bytes_per_row,
                        bounds,
                        src_rect,
                        dst_rect,
                        mask_region,
                        data,
                        mode,
                    } => {
                        self.apply_bitmap(
                            src_rect,
                            dst_rect,
                            mode,
                            bounds,
                            bytes_per_row as usize,
                            data,
                            mask_region,
                        );
                    }
                }
            }
            v2::Opcode::DirectBitsRect(direct_bits) => {
                log::trace!("v2::Opcode::DirectBitsRect(..)");
                self.apply_direct_color_bitmap(
                    direct_bits.src_rect,
                    direct_bits.dst_rect,
                    &direct_bits.pix_map,
                    direct_bits.data,
                    direct_bits.mask,
                );
            }
            v2::Opcode::DirectBitsRgn(direct_bits) => {
                log::trace!("v2::Opcode::DirectBitsRgn(..)");
                self.apply_direct_color_bitmap(
                    direct_bits.src_rect,
                    direct_bits.dst_rect,
                    &direct_bits.pix_map,
                    direct_bits.data,
                    direct_bits.mask,
                );
            }
            v2::Opcode::ReservedForAppleUse50(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse50")
            }
            v2::Opcode::ReservedForAppleUse51(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse51")
            }
            v2::Opcode::ReservedForAppleUse52(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse52")
            }
            v2::Opcode::ReservedForAppleUse53(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse53")
            }
            v2::Opcode::ShortComment(v) => log::trace!("v2::Opcode::ShortComment({v:?})"),
            v2::Opcode::LongComment(v, _, _) => {
                log::trace!("v2::Opcode::LongComment({v:?}, ..)")
            }
            v2::Opcode::ReservedForAppleUse54(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse54")
            }
            v2::Opcode::ReservedForAppleUse55(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse55")
            }
            v2::Opcode::ReservedForAppleUse56 => {
                log::info!("v2::Opcode::ReservedForAppleUse56")
            }
            v2::Opcode::ReservedForAppleUse57 => {
                log::info!("v2::Opcode::ReservedForAppleUse57")
            }
            v2::Opcode::ReservedForAppleUse58(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse58")
            }
            v2::Opcode::ReservedForAppleUse59(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse59")
            }
            v2::Opcode::OpEndPic => {
                log::trace!("v2::Opcode::OpEndPic");
                return true;
            }
            v2::Opcode::ReservedForAppleUse60(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse60")
            }
            v2::Opcode::ReservedForAppleUse61(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse61")
            }
            v2::Opcode::ReservedForAppleUse62(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse62")
            }
            v2::Opcode::Version => log::trace!("v2::Opcode::Version"),
            v2::Opcode::ReservedForAppleUse63(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse63")
            }
            v2::Opcode::HeaderOp(_) => log::trace!("v2::Opcode::HeaderOp(..)"),
            v2::Opcode::ReservedForAppleUse64(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse64")
            }
            v2::Opcode::ReservedForAppleUse65(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse65")
            }
            v2::Opcode::ReservedForAppleUse66(_) => {
                log::info!("v2::Opcode::ReservedForAppleUse66")
            }
            v2::Opcode::ReservedForAppleUse67 => {
                log::info!("v2::Opcode::ReservedForAppleUse67")
            }
            v2::Opcode::ReservedForAppleUse68 => {
                log::info!("v2::Opcode::ReservedForAppleUse68")
            }
            v2::Opcode::ReservedForAppleUse69(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse69")
            }
            v2::Opcode::CompressedQuickTime(_, _) => {
                log::info!("v2::Opcode::CompressedQuickTime")
            }
            v2::Opcode::UncompressedQuickTime(_, _) => {
                log::info!("v2::Opcode::UncompressedQuickTime")
            }
            v2::Opcode::ReservedForAppleUse70(_, _) => {
                log::info!("v2::Opcode::ReservedForAppleUse70")
            }
            v2::Opcode::Unknown(code) => {
                log::info!("v2::Opcode::Unknown({code})");
                return true;
            }
        }

        false
    }

    /// Returns a shared reference to the current canvas image.
    pub fn image(&self) -> &image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        &self.canvas
    }

    /// Consumes the drawing context and returns the final RGBA image buffer.
    pub fn into_image(self) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        self.canvas
    }

    fn apply_bitmap(
        &mut self,
        src_rect: Rect,
        dst_rect: Rect,
        mode: TransferMode,
        source_image_bounds: Rect,
        bytes_per_row: usize,
        data: Vec<u8>,
        mut mask_region: Option<Region>,
    ) {
        if let Some(region) = mask_region.as_mut() {
            region.prepare();
        }

        if src_rect.width() != dst_rect.width() || src_rect.height() != dst_rect.height() {
            log::warn!("PackBitsRect can only be used to translate image.");
            return;
        }

        if !source_image_bounds.contains(&src_rect) {
            log::error!("Source image does not contain source rect.");
            return;
        }

        let source_image = decode_bitmap(&source_image_bounds, bytes_per_row, data);
        self.blit_masked(
            src_rect,
            dst_rect,
            mask_region,
            source_image_bounds,
            source_image,
            mode,
        );
    }

    fn apply_direct_color_bitmap(
        &mut self,
        src_rect: Rect,
        dst_rect: Rect,
        pix_map: &PixMap,
        pix_map_data: Vec<u8>,
        mask_region: Option<Region>,
    ) {
        self.apply_pixmap(
            pix_map,
            ColorTable::new(),
            src_rect,
            dst_rect,
            TransferMode::default(),
            mask_region,
            pix_map_data,
        );
    }

    fn apply_pixmap(
        &mut self,
        pix_map: &PixMap,
        color_table: ColorTable,
        src_rect: Rect,
        dst_rect: Rect,
        mode: TransferMode,
        mut mask_region: Option<Region>,
        data: Vec<u8>,
    ) {
        if let Some(region) = mask_region.as_mut() {
            region.prepare();
        }

        if src_rect.width() != dst_rect.width() || src_rect.height() != dst_rect.height() {
            log::warn!("PackBitsRect can only be used to translate image.");
            return;
        }

        if !pix_map.bounds.contains(&src_rect) {
            log::error!("Source image does not contain source rect.");
            return;
        }
        let src_image_bounds = pix_map.bounds;
        let source_image = decode_pixmap(pix_map, &color_table, &data);

        self.blit_masked(
            src_rect,
            dst_rect,
            mask_region,
            src_image_bounds,
            source_image,
            mode,
        );
    }

    fn blit_masked(
        &mut self,
        src_rect: Rect,
        dst_rect: Rect,
        mask_region: Option<Region>,
        src_image_bounds: Rect,
        source_image: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>,
        mode: TransferMode,
    ) {
        assert!(
            !mode.is_unknown(),
            "Unknown mask mode {mode:?} encountered!"
        );
        for canvas_y in 0..self.canvas.height() {
            for canvas_x in 0..self.canvas.width() {
                let image_x = canvas_x as i32 + self.bounds.min_x() as i32;
                let image_y = canvas_y as i32 + self.bounds.min_y() as i32;

                // image pixel is clipped
                if !self.clipping_region.contains(image_x, image_y) {
                    continue;
                }

                // destination rect clips pixel
                if !dst_rect.includes(image_x, image_y) {
                    continue;
                }

                let source_x = image_x - dst_rect.min_x() as i32 + src_rect.min_x() as i32;
                let source_y = image_y - dst_rect.min_y() as i32 + src_rect.min_y() as i32;

                // source rect clips pixel
                if !src_rect.includes(source_x, source_y) {
                    continue;
                }

                // Mask clips pixel at the source
                if let Some(regn) = mask_region.as_ref()
                    && !regn.contains(image_x, image_y)
                {
                    continue;
                }

                let source_canvas_x = source_x - src_image_bounds.min_x() as i32;
                let source_canvas_y = source_y - src_image_bounds.min_y() as i32;
                let pixel = *source_image.get_pixel(source_canvas_x as u32, source_canvas_y as u32);
                *self.canvas.get_pixel_mut(canvas_x, canvas_y) = pixel;
            }
        }
    }

    fn clip_region(&mut self, mut region: Region) {
        region.prepare();
        self.clipping_region = region;
    }
}

/// Decodes a 1-bit bitmap into an RGBA image.
///
/// Bits are expanded to grayscale pixels (`0x00`/`0xFF`) with opaque alpha.
pub fn decode_bitmap(
    bounds: &Rect,
    bytes_per_row: usize,
    data: Vec<u8>,
) -> image::ImageBuffer<Rgba<u8>, Vec<u8>> {
    let width = bounds.width() as u32;
    let height = bounds.height() as u32;

    let mut image = image::ImageBuffer::new(width, height);
    if bytes_per_row == 0 {
        return image;
    }

    for y in 0..height {
        for x in (0..width).step_by(8) {
            let mut row = data[y as usize * bytes_per_row + (x >> 3) as usize];
            let z_limit = if (x + 8) <= width { 8 } else { width - x };
            for z in 0..z_limit {
                let value: u8 = if (row & 0x80) != 0 { 0 } else { 0xff };
                row <<= 1;
                *image.get_pixel_mut(x + z, y) = image::Rgba([value, value, value, 0xFF]);
            }
        }
    }

    image
}

/// Decodes a pixmap payload and color table into an RGBA image.
///
/// Supports indexed and direct-color pixmaps used by QuickDraw PICT data.
pub fn decode_pixmap(
    pix_map: &PixMap,
    color_table: &ColorTable,
    data: &Vec<u8>,
) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    let width = pix_map.bounds.width() as u32;
    let height = pix_map.bounds.height() as u32;

    if pix_map.pixel_type.is_unknown() {
        panic!("Pixel type must be 0 (indexed) or 16 (direct color)");
    }

    if pix_map.pixel_type.is_indexed() && color_table.ct_table.is_empty() {
        panic!("Color table should not be empty for indexed pixel type");
    };

    if pix_map.pixel_type.is_direct_color()
        && (pix_map.component_count < 3 || pix_map.component_count > 4)
    {
        panic!(
            "Direct color images with {} components are not supported",
            pix_map.component_count
        );
    }

    if pix_map.pixel_type.is_direct_color()
        && pix_map.pixel_size == 16
        && pix_map.component_size != 5
    {
        panic!("unsupported 16-bit channel width");
    }

    if pix_map.pixel_type.is_direct_color()
        && pix_map.pixel_size == 32
        && pix_map.component_size != 8
    {
        panic!("unsupported 32-bit channel width");
    }

    let lookup_entry = match pix_map.pixel_size {
        1 => |x: usize, y: usize, bytes_per_row: usize, data: &Vec<u8>| -> u32 {
            (data[(y * bytes_per_row) + (x / 8)] as u32 >> (7 - (x & 7))) & 1
        },
        2 => |x: usize, y: usize, bytes_per_row: usize, data: &Vec<u8>| -> u32 {
            (data[(y * bytes_per_row) + (x / 4)] as u32 >> (6 - ((x & 3) * 2))) & 3
        },
        4 => |x: usize, y: usize, bytes_per_row: usize, data: &Vec<u8>| -> u32 {
            (data[(y * bytes_per_row) + (x / 2)] as u32 >> (4 - ((x & 1) * 4))) & 15
        },
        8 => |x: usize, y: usize, bytes_per_row: usize, data: &Vec<u8>| -> u32 {
            data[(y * bytes_per_row) + x] as u32
        },
        16 => |x: usize, y: usize, bytes_per_row: usize, data: &Vec<u8>| -> u32 {
            ((data[(y * bytes_per_row) + (x * 2)] as u32) << 8)
                | (data[(y * bytes_per_row) + (x * 2) + 1] as u32)
        },
        32 => |x: usize, y: usize, bytes_per_row: usize, data: &Vec<u8>| -> u32 {
            (data[(y * bytes_per_row) + (x * 4)] as u32) << 24
                | (data[(y * bytes_per_row) + (x * 4) + 1] as u32) << 16
                | (data[(y * bytes_per_row) + (x * 4) + 2] as u32) << 8
                | (data[(y * bytes_per_row) + (x * 4) + 3] as u32)
        },
        pixel_size => {
            panic!("Pixel size {pixel_size} is not supported");
        }
    };

    if pix_map.pixel_size == 16 && pix_map.component_size == 5 {
        log::info!("Color Format: xrgb1555");
    }

    if pix_map.pixel_size == 32 && pix_map.component_size == 8 {
        log::info!("Color Format: xrgb8888");
    }

    let mut image = image::ImageBuffer::new(width, height);
    let mut warned_colors: HashSet<u32> = HashSet::new();
    for y in 0..height {
        for x in 0..width {
            let color = lookup_entry(
                x as usize,
                y as usize,
                pix_map.bytes_per_row() as usize,
                data,
            );

            if pix_map.pixel_type.is_indexed() {
                if let Some(e) = color_table.ct_table.get(color as usize) {
                    *image.get_pixel_mut(x, y) = image::Rgba::<u8>([
                        (e.rgb.red >> 8) as u8,
                        (e.rgb.green >> 8) as u8,
                        (e.rgb.blue >> 8) as u8,
                        0xFF,
                    ]);
                } else if !warned_colors.contains(&color) {
                    //log::warn!("color {} not found in color map", color_id);
                    warned_colors.insert(color);
                }
            } else if pix_map.pixel_size == 16 && pix_map.component_size == 5 {
                let r: u8 = ((color >> 7) & 0xF8) as u8 | ((color >> 12) & 0x07) as u8;
                let g: u8 = ((color >> 2) & 0xF8) as u8 | ((color >> 7) & 0x07) as u8;
                let b: u8 = ((color << 3) & 0xF8) as u8 | ((color >> 2) & 0x07) as u8;
                *image.get_pixel_mut(x, y) = image::Rgba::<u8>([r, g, b, 0xFFu8]);
            } else if pix_map.pixel_size == 32
                && pix_map.component_size == 8
                && pix_map.component_count == 3
            {
                *image.get_pixel_mut(x, y) = image::Rgba::<u8>([
                    ((color >> 16) & 0xFF) as u8,
                    ((color >> 8) & 0xFF) as u8,
                    (color & 0xFF) as u8,
                    0xFF,
                ]);
            } else if pix_map.pixel_size == 32
                && pix_map.component_size == 8
                && pix_map.component_count == 4
            {
                *image.get_pixel_mut(x, y) = image::Rgba::<u8>([
                    ((color >> 16) & 0xFF) as u8,
                    ((color >> 8) & 0xFF) as u8,
                    (color & 0xFF) as u8,
                    ((color >> 24) & 0xFF) as u8,
                ]);
            } else {
                log::error!("unsupported pixel format");
                return image;
            }
        }
    }

    image
}
