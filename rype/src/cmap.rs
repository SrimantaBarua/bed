// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::Ordering;

use fnv::FnvHashMap;

use crate::common::GlyphID;
use crate::error::*;
use crate::types::{get_u16, get_u32};

/// Wrapper around character to glyph index mapping table
#[derive(Debug)]
pub(crate) struct Cmap {
    subtables: Vec<Subtable>,
    active: usize, // Index of active subtable
}

impl Cmap {
    pub(crate) fn load(data: &[u8]) -> Result<Cmap> {
        let num_tables = get_u16(data, offsets::NUM_TABLES)?;
        let mut record_offset = offsets::ENCODING_RECORDS;
        let mut subtables = Vec::new();
        let mut active = None;

        for _ in 0..num_tables {
            let subtable_off = get_u32(data, record_offset + offsets::SUBTABLE_OFF)? as usize;
            let format = get_u16(data, subtable_off)?;
            match format {
                4 => {
                    if active.is_none() {
                        active = Some(subtables.len());
                    }
                    let end = subtable_off + get_u16(data, subtable_off + 2)? as usize;
                    subtables.push(Subtable::load_format_4(&data[subtable_off..end])?);
                }
                12 => {
                    active = Some(subtables.len());
                    let end = subtable_off + get_u32(data, subtable_off + 4)? as usize;
                    subtables.push(Subtable::load_format_12(&data[subtable_off..end])?);
                }
                // We're only interested in table format 4 and 12
                _ => continue,
            }
            record_offset += sizes::ENCODING_RECORD;
        }

        active
            .ok_or(Error::CmapNoTable)
            .map(|active| Cmap { subtables, active })
    }

    pub(crate) fn glyph_id_for_codepoint(&self, codepoint: u32) -> GlyphID {
        self.subtables[self.active].find(codepoint)
    }
}

#[derive(Debug)]
struct CmapEntry {
    start_codepoint: u32,
    end_codepoint: u32,
    start_glyph: u32,
}

#[derive(Debug)]
struct Subtable {
    map: FnvHashMap<u32, u32>,
    ranges: Vec<CmapEntry>,
}

impl Subtable {
    fn find(&self, codepoint: u32) -> GlyphID {
        self.map
            .get(&codepoint)
            .map(|u| GlyphID(*u))
            .or_else(|| {
                self.ranges
                    .binary_search_by(|entry| {
                        if entry.start_codepoint > codepoint {
                            Ordering::Greater
                        } else if entry.end_codepoint < codepoint {
                            Ordering::Less
                        } else {
                            Ordering::Equal
                        }
                    })
                    .ok()
                    .map(|index| {
                        GlyphID(
                            self.ranges[index].start_glyph
                                + (codepoint - self.ranges[index].start_codepoint),
                        )
                    })
            })
            .unwrap_or(GlyphID(0))
    }

    fn load_format_4(data: &[u8]) -> Result<Subtable> {
        let mut map = FnvHashMap::default();
        let mut ranges = Vec::new();
        let segcount_x2 = get_u16(data, offsets::SEGCOUNT_X2)? as usize;
        for offset in (0..segcount_x2).step_by(2) {
            let end_codepoint = get_u16(data, 14 + offset)? as u32;
            let start_codepoint = get_u16(data, 16 + segcount_x2 + offset)? as u32;
            let delta = get_u16(data, 16 + segcount_x2 * 2 + offset)? as u32;
            let range = get_u16(data, 16 + segcount_x2 * 3 + offset)? as u32;
            if range == 0 {
                let start_glyph = (start_codepoint + delta) & 0xffff;
                map.insert(start_codepoint, start_glyph);
                if start_codepoint == end_codepoint {
                } else {
                    ranges.push(CmapEntry {
                        start_codepoint,
                        end_codepoint,
                        start_glyph,
                    });
                }
            } else {
                let glyph_off = 16 + segcount_x2 * 3 + offset + range as usize;
                for i in start_codepoint..=end_codepoint {
                    let code_off = (i - start_codepoint) as usize;
                    let glyph_id = get_u16(data, glyph_off + code_off * 2)? as u32;
                    let start_glyph = if glyph_id == 0 {
                        0
                    } else {
                        (glyph_id + delta) & 0xffff
                    };
                    map.insert(i, start_glyph);
                }
            }
        }
        Ok(Subtable { map, ranges })
    }

    fn load_format_12(data: &[u8]) -> Result<Subtable> {
        let mut map = FnvHashMap::default();
        let mut ranges = Vec::new();
        let num_groups = get_u32(data, offsets::NUM_GROUPS)? as usize;
        for offset in (0..num_groups * sizes::TABLE_12_RECORD).step_by(sizes::TABLE_12_RECORD) {
            let start_codepoint = get_u32(data, offsets::REC_START_CODE + offset)?;
            let end_codepoint = get_u32(data, offsets::REC_END_CODE + offset)?;
            let start_glyph = get_u32(data, offsets::REC_START_GLYPH + offset)?;
            if start_codepoint == end_codepoint {
                map.insert(start_codepoint, start_glyph);
            } else {
                ranges.push(CmapEntry {
                    start_codepoint,
                    end_codepoint,
                    start_glyph,
                });
            }
        }
        Ok(Subtable { map, ranges })
    }
}

mod offsets {
    pub(super) const NUM_TABLES: usize = 2;
    pub(super) const ENCODING_RECORDS: usize = 4;
    pub(super) const SUBTABLE_OFF: usize = 4;

    pub(super) const SEGCOUNT_X2: usize = 6;

    pub(super) const NUM_GROUPS: usize = 12;
    pub(super) const REC_START_CODE: usize = 16;
    pub(super) const REC_END_CODE: usize = 20;
    pub(super) const REC_START_GLYPH: usize = 24;
}

mod sizes {
    pub(super) const ENCODING_RECORD: usize = 8;
    pub(super) const TABLE_12_RECORD: usize = 12;
}
