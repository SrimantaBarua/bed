// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::error::Result;
use crate::rcbuffer::RcBuf;
use crate::types::get_u16;

/// Wrapper around Maximum Profile table
#[derive(Debug)]
pub(crate) struct Maxp {
    pub(crate) num_glyphs: u16, // Number of glyphs in the font
}

impl Maxp {
    pub(crate) fn load(data: RcBuf) -> Result<Maxp> {
        let num_glyphs = get_u16(&data, offsets::NUM_GLYPHS)?;
        Ok(Maxp { num_glyphs })
    }
}

mod offsets {
    pub(super) const NUM_GLYPHS: usize = 4;
}
