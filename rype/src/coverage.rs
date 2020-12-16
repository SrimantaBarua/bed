// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::error::*;
use crate::types::get_u16;

#[derive(Debug)]
pub(crate) enum Coverage {
    Format1 { glyphs: Vec<u16> },
    Format2 { ranges: Vec<RangeRecord> },
}

impl Coverage {
    pub(crate) fn load(data: &[u8]) -> Result<Coverage> {
        let format = get_u16(data, 0)?;
        match format {
            1 => {
                let glyph_count = get_u16(data, 2)? as usize;
                let mut glyphs = Vec::new();
                for off in (4..4 + glyph_count * 2).step_by(2) {
                    glyphs.push(get_u16(data, off)?);
                }
                Ok(Coverage::Format1 { glyphs })
            }
            2 => {
                let range_count = get_u16(data, 2)? as usize;
                let mut ranges = Vec::new();
                for off in (4..4 + range_count * 6).step_by(6) {
                    let start_glyph_id = get_u16(data, off)?;
                    let end_glyph_id = get_u16(data, off + 2)?;
                    let start_coverage_index = get_u16(data, off + 4)?;
                    ranges.push(RangeRecord {
                        start_glyph_id,
                        end_glyph_id,
                        start_coverage_index,
                    });
                }
                Ok(Coverage::Format2 { ranges })
            }
            _ => Err(Error::Invalid),
        }
    }
}

#[derive(Debug)]
pub(crate) struct RangeRecord {
    start_glyph_id: u16,
    end_glyph_id: u16,
    start_coverage_index: u16,
}
