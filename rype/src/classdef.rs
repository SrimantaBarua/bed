// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use fnv::FnvHashMap;

use crate::common::GlyphID;
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
    Fmt2 {
        map: FnvHashMap<u16, u16>,
        ranges: Vec<ClassRangeRecord>,
    },
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
                let mut map = FnvHashMap::default();
                let mut ranges = Vec::new();
                for off in (4..4 + range_count * 6).step_by(6) {
                    let start_glyph = get_u16(data, off)?;
                    let end_glyph = get_u16(data, off + 2)?;
                    let class = get_u16(data, off + 4)?;
                    if start_glyph == end_glyph {
                        map.insert(start_glyph, class);
                    } else {
                        ranges.push(ClassRangeRecord {
                            start_glyph,
                            end_glyph,
                            class,
                        });
                    }
                }
                Ok(ClassDef::Fmt2 { map, ranges })
            }
            _ => Err(Error::Invalid),
        }
    }

    pub(crate) fn glyph_class(&self, glyph: GlyphID) -> Option<u32> {
        match self {
            ClassDef::Fmt1 {
                start_glyph,
                class_values,
            } => {
                if glyph.0 < *start_glyph as u32
                    || glyph.0 >= *start_glyph as u32 + class_values.len() as u32
                {
                    None
                } else {
                    Some(class_values[(glyph.0 - *start_glyph as u32) as usize] as u32)
                }
            }
            ClassDef::Fmt2 { map, ranges } => {
                if let Some(g) = map.get(&(glyph.0 as u16)) {
                    return Some(*g as u32);
                }
                for range in ranges {
                    if (range.end_glyph as u32) < glyph.0 {
                        break;
                    }
                    if (range.start_glyph as u32) > glyph.0 {
                        continue;
                    }
                    return Some((glyph.0 - range.start_glyph as u32) + range.class as u32);
                }
                None
            }
        }
    }
}
