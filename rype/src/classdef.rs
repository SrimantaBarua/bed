// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::error::*;
use crate::types::get_u16;

#[derive(Debug)]
pub(crate) struct ClassRangeRecord {
    start_glyph: u16,
    end_glyph: u16,
    class: u16,
}

/// Wrapper around class definition table
#[derive(Debug)]
pub(crate) enum ClassDef {
    Fmt1 {
        start_glyph: u16,
        class_values: Vec<u16>,
    },
    Fmt2(Vec<ClassRangeRecord>),
}

impl ClassDef {
    pub(crate) fn load(data: &[u8]) -> Result<ClassDef> {
        match get_u16(data, 0)? {
            1 => {
                let start_glyph = get_u16(data, 2)?;
                let glyph_count = get_u16(data, 4)? as usize;
                let mut class_values = Vec::new();
                for off in (6..6 + glyph_count * 2).step_by(2) {
                    class_values.push(get_u16(data, off)?);
                }
                Ok(ClassDef::Fmt1 {
                    start_glyph,
                    class_values,
                })
            }
            2 => {
                let range_count = get_u16(data, 2)? as usize;
                let mut ranges = Vec::new();
                for off in (4..4 + range_count * 6).step_by(6) {
                    let start_glyph = get_u16(data, off)?;
                    let end_glyph = get_u16(data, off + 2)?;
                    let class = get_u16(data, off + 4)?;
                    ranges.push(ClassRangeRecord {
                        start_glyph,
                        end_glyph,
                        class,
                    });
                }
                Ok(ClassDef::Fmt2(ranges))
            }
            _ => Err(Error::Invalid),
        }
    }
}
