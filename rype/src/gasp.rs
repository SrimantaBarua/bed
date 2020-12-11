// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::error::*;
use crate::types::get_u16;

/// Wrapper around grid-fitting and scan-conversion procedure table
#[derive(Debug)]
pub(crate) struct Gasp(Vec<GaspRangeRecord>);

impl Gasp {
    pub(crate) fn load(data: &[u8]) -> Result<Gasp> {
        let num_ranges = get_u16(data, offsets::NUM_RANGES)? as usize;
        let mut records = Vec::new();
        for off in (0..num_ranges * sizes::RECORD).step_by(sizes::RECORD) {
            let max_ppem = get_u16(data, off + offsets::RANGES)?;
            let behavior = get_u16(data, off + offsets::RANGES + 2)?;
            let behavior = GaspBehavior::from_bits_truncate(behavior);
            records.push(GaspRangeRecord { max_ppem, behavior });
        }
        Ok(Gasp(records))
    }

    pub(crate) fn get_for_ppem(&self, ppem: u16) -> Option<GaspRangeRecord> {
        for rec in &self.0 {
            if rec.max_ppem >= ppem {
                return Some(*rec);
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GaspRangeRecord {
    pub(crate) max_ppem: u16,
    pub(crate) behavior: GaspBehavior,
}

bitflags! {
    pub(crate) struct GaspBehavior: u16 {
        const GRIDFIT             = 0x0001; // Use gridfitting
        const DO_GRAY             = 0x0001; // Use grayscale rendering
        const SYMMETRIC_GRIDFIT   = 0x0001; // Use gridfitting with ClearType symmetric smoothing
        const SYMMETRIC_SMOOTHING = 0x0001; // Smoothing along multiple axes with ClearType
    }
}

mod offsets {
    pub(super) const NUM_RANGES: usize = 2;
    pub(super) const RANGES: usize = 4;
}

mod sizes {
    pub(super) const RECORD: usize = 4;
}
