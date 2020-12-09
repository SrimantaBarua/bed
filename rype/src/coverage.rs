// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use crate::error::*;
use crate::rcbuffer::RcBuf;
use crate::types::get_u16;

pub(crate) struct Coverage(RcBuf);

impl Coverage {
    pub(crate) fn load(data: RcBuf) -> Result<Coverage> {
        Ok(Coverage(data))
    }
}

impl fmt::Debug for Coverage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.0;
        let format = get_u16(slice, 0).unwrap();
        let mut tmp = f.debug_struct("Coverage");
        tmp.field("format", &format);
        match format {
            1 => {
                let glyph_count = get_u16(slice, 2).unwrap() as usize;
                tmp.field("glyphCount", &glyph_count).field(
                    "glyphArray",
                    &(4..4 + glyph_count * 2)
                        .step_by(2)
                        .map(|off| get_u16(slice, off).unwrap())
                        .collect::<Vec<_>>(),
                );
            }
            2 => {
                let range_count = get_u16(slice, 2).unwrap() as usize;
                tmp.field("rangeCount", &range_count).field(
                    "rangeRecords",
                    &(4..4 + range_count * 6)
                        .step_by(6)
                        .map(|off| {
                            let start_glyph_id = get_u16(slice, off).unwrap();
                            let end_glyph_id = get_u16(slice, off + 2).unwrap();
                            let start_coverage_index = get_u16(slice, off + 4).unwrap();
                            RangeRecord {
                                start_glyph_id,
                                end_glyph_id,
                                start_coverage_index,
                            }
                        })
                        .collect::<Vec<_>>(),
                );
            }
            _ => unreachable!("invalid coverage format"),
        }
        tmp.finish()
    }
}

#[derive(Debug)]
struct RangeRecord {
    start_glyph_id: u16,
    end_glyph_id: u16,
    start_coverage_index: u16,
}
