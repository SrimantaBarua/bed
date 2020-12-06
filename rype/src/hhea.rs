// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::error::Result;
use crate::rcbuffer::RcBuf;
use crate::types::{get_i16, get_u16};

/// Wrapper around the horizontal head table
#[derive(Debug)]
pub(crate) struct Hhea {
    pub(crate) ascender: i16,
    pub(crate) descender: i16,
    pub(crate) line_gap: i16,
    pub(crate) advance_width_max: u16,
    pub(crate) min_lsb: i16,
    pub(crate) min_rsb: i16,
    pub(crate) x_max_extent: i16,
    pub(crate) num_h_metrics: u16,
}

impl Hhea {
    pub(crate) fn load(data: RcBuf) -> Result<Hhea> {
        let slice = data.as_ref();
        let ascender = get_i16(slice, offsets::ASCENDER)?;
        let descender = get_i16(slice, offsets::DESCENDER)?;
        let line_gap = get_i16(slice, offsets::LINE_GAP)?;
        let advance_width_max = get_u16(slice, offsets::ADVANCE_WIDTH_MAX)?;
        let min_lsb = get_i16(slice, offsets::MIN_LSB)?;
        let min_rsb = get_i16(slice, offsets::MIN_RSB)?;
        let x_max_extent = get_i16(slice, offsets::X_MAX_EXTENT)?;
        let num_h_metrics = get_u16(slice, offsets::NUM_H_METRICS)?;
        Ok(Hhea {
            ascender,
            descender,
            line_gap,
            advance_width_max,
            min_lsb,
            min_rsb,
            x_max_extent,
            num_h_metrics,
        })
    }
}

mod offsets {
    pub(super) const ASCENDER: usize = 4;
    pub(super) const DESCENDER: usize = 6;
    pub(super) const LINE_GAP: usize = 8;
    pub(super) const ADVANCE_WIDTH_MAX: usize = 10;
    pub(super) const MIN_LSB: usize = 12;
    pub(super) const MIN_RSB: usize = 14;
    pub(super) const X_MAX_EXTENT: usize = 16;
    pub(super) const NUM_H_METRICS: usize = 34;
}
