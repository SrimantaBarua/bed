// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::types::get_u32;
use crate::{Error, Result};

/// TTC Header
pub(crate) struct TTC {
    offsets: Vec<u32>, // Array of offsets to the TableDirectory for each font
}

impl TTC {
    pub(crate) fn load_from(data: &[u8]) -> Result<TTC> {
        // Don't need to verify "ttcf" tag since we're trusting this was called after checking
        let num_fonts = get_u32(data, 8)? as usize;
        let mut offsets = Vec::new();
        offsets.reserve(num_fonts);
        for i in 0..num_fonts {
            let off = 12 + i * 4;
            offsets.push(get_u32(data, off)?);
        }
        Ok(TTC { offsets })
    }

    pub(crate) fn offset(&self, index: usize) -> Result<usize> {
        if index >= self.offsets.len() {
            Err(Error::FaceIndexOutOfBounds)
        } else {
            Ok(self.offsets[index] as usize)
        }
    }
}
