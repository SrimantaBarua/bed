// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use crate::error::*;
use crate::rcbuffer::RcBuf;
use crate::types::{get_u16, get_u16_unchecked};

/// Wrapper around grid-fitting and scan-conversion procedure table
pub(crate) struct Gasp(RcBuf);

impl Gasp {
    pub(crate) fn load(data: RcBuf) -> Result<Gasp> {
        let slice = &data;
        let num_ranges = get_u16(slice, offsets::NUM_RANGES)? as usize;
        if slice.len() < num_ranges * sizes::RECORD + offsets::RANGES {
            Err(Error::Invalid)
        } else {
            Ok(Gasp(data))
        }
    }

    pub(crate) fn get_for_ppem(&self, ppem: u16) -> Option<GaspRangeRecord> {
        let slice = &self.0;
        let num_ranges = unsafe { get_u16_unchecked(slice, offsets::NUM_RANGES) as usize };
        for offset in (0..num_ranges * sizes::RECORD).step_by(sizes::RECORD) {
            let max_ppem = unsafe { get_u16_unchecked(slice, offset + offsets::RANGES) };
            if max_ppem >= ppem {
                let behavior = unsafe { get_u16_unchecked(slice, offset + offsets::RANGES + 2) };
                let behavior = GaspBehavior::from_bits_truncate(behavior);
                return Some(GaspRangeRecord { max_ppem, behavior });
            }
        }
        None
    }
}

impl fmt::Debug for Gasp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.0;
        let num_ranges = unsafe { get_u16_unchecked(slice, offsets::NUM_RANGES) as usize };
        f.debug_struct("Gasp")
            .field("numRanges", &num_ranges)
            .field("gaspRanges", &(0..num_ranges * sizes::RECORD).step_by(sizes::RECORD)
                .map(|off| {
                    let max_ppem = unsafe { get_u16_unchecked(slice, off + offsets::RANGES) };
                    let behavior = unsafe { get_u16_unchecked(slice, off + offsets::RANGES + 2) };
                    let behavior = GaspBehavior::from_bits_truncate(behavior);
                    GaspRangeRecord { max_ppem, behavior }
                })
                .collect::<Vec<_>>())
            .finish()
    }
}

#[derive(Debug)]
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
