// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use geom::{point2, BBox};

use crate::error::*;
use crate::rcbuffer::RcBuf;
use crate::types::{get_i16, get_u16};

bitflags! {
    /// Head table flags
    pub(crate) struct Flags : u16 {
        const BASELINE_Y0 = 0x0001; // Baseline for font at y = 0
        const LEFT_SIDEBEARING_X0 = 0x0002; // Left sidebearing at x = 0 (relevant for TrueType)
        const INSTR_PT_SIZE = 0x0004; // Instructions may depend on point size
        const FORCE_PPEM_INT = 0x0008; // Force PPEM to integer values for all scalar math
        const INSTR_ADV_WIDTH = 0x0010; // Instructions may alter advance width
        const LAST_RESORT = 0x4000; // Indicates that glyphs in cmap are generic representations
    }
}

/// Index to loc format
#[derive(Debug)]
pub(crate) enum IdxLocFmt {
    Off16, // Short offsets (16 bit)
    Off32, // Long offsets (32 bit)
}

/// Wrapper around OpenType HEAD table
#[derive(Debug)]
pub(crate) struct Head {
    pub(crate) units_per_em: u16,
    pub(crate) flags: Flags,         // Head table flags
    pub(crate) bbox: BBox<i16>,      // Glyph bounding box (limits)
    pub(crate) lowest_rec_ppem: u16, // Smallest readable size in pixels
    pub(crate) idx_loc_fmt: IdxLocFmt,
}

impl Head {
    pub(crate) fn load(data: RcBuf) -> Result<Head> {
        let slice = data.as_ref();
        let flags = Flags::from_bits_truncate(get_u16(slice, offsets::FLAGS)?);
        let units_per_em = get_u16(slice, offsets::UNITS_PER_EM)?;
        let xmin = get_i16(slice, offsets::XMIN)?;
        let ymin = get_i16(slice, offsets::YMIN)?;
        let xmax = get_i16(slice, offsets::XMAX)?;
        let ymax = get_i16(slice, offsets::YMAX)?;
        let bbox = BBox::new(point2(xmin, ymin), point2(xmax, ymax));
        let lowest_rec_ppem = get_u16(slice, offsets::LOW_REC_PPEM)?;
        let idx_loc_fmt = get_i16(slice, offsets::IDX_LOC_FMT).and_then(|i| match i {
            0 => Ok(IdxLocFmt::Off16),
            1 => Ok(IdxLocFmt::Off32),
            _ => Err(Error::Invalid),
        })?;
        Ok(Head {
            units_per_em,
            flags,
            bbox,
            lowest_rec_ppem,
            idx_loc_fmt,
        })
    }
}

mod offsets {
    pub(super) const FLAGS: usize = 16;
    pub(super) const UNITS_PER_EM: usize = 18;
    pub(super) const XMIN: usize = 36;
    pub(super) const YMIN: usize = 38;
    pub(super) const XMAX: usize = 40;
    pub(super) const YMAX: usize = 42;
    pub(super) const LOW_REC_PPEM: usize = 46;
    pub(super) const IDX_LOC_FMT: usize = 50;
}
