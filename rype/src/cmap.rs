// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use crate::error::*;
use crate::rcbuffer::RcBuf;
use crate::types::{get_i16, get_u16, get_u32};

/// Wrapper around character to glyph index mapping table
#[derive(Debug)]
pub(crate) struct Cmap {
    subtables: Vec<Subtable>, // Offsets to suitable subtables
    active: usize,            // Index of active subtable
}

impl Cmap {
    pub(crate) fn load(data: RcBuf) -> Result<Cmap> {
        let slice = &data;
        let num_tables = get_u16(slice, offsets::NUM_TABLES)?;
        let mut record_offset = offsets::ENCODING_RECORDS;
        let mut subtables = Vec::new();
        let mut active = None;

        for _ in 0..num_tables {
            let subtable_off = get_u32(slice, record_offset + offsets::SUBTABLE_OFF)? as usize;
            let format = get_u16(slice, subtable_off)?;
            match format {
                4 => {
                    if active.is_none() {
                        active = Some(subtables.len());
                    }
                    let end = subtable_off + get_u16(slice, subtable_off + 2)? as usize;
                    subtables.push(Subtable::Format4(data.slice(subtable_off..end)));
                }
                12 => {
                    active = Some(subtables.len());
                    let end = subtable_off + get_u32(slice, subtable_off + 4)? as usize;
                    subtables.push(Subtable::Format12(data.slice(subtable_off..end)));
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

    pub(crate) fn glyph_id_for_codepoint(&self, codepoint: u32) -> Result<u32> {
        self.subtables[self.active].gid_for_cp(codepoint)
    }
}

enum Subtable {
    Format4(RcBuf),
    Format12(RcBuf),
}

impl Subtable {
    fn gid_for_cp(&self, cp: u32) -> Result<u32> {
        match self {
            Subtable::Format4(data) => gid_for_cp_format_4(data, cp),
            Subtable::Format12(data) => gid_for_cp_format_12(data, cp),
        }
    }
}

fn gid_for_cp_format_4(data: &[u8], cp: u32) -> Result<u32> {
    let segcount_x2 = get_u16(data, offsets::SEGCOUNT_X2)? as usize;
    for offset in (0..segcount_x2).step_by(2) {
        let end_cp = get_u16(data, offsets::BASE_ENDCODE + offset)? as u32;
        if cp > end_cp {
            continue;
        }
        let start_cp = get_u16(data, offsets::BASE_STARTCODE + segcount_x2 + offset)? as u32;
        if cp < start_cp {
            break;
        }
        let delta_off = offsets::BASE_DELTA + segcount_x2 * 2 + offset;
        let range_off = offsets::BASE_RANGE_OFF + segcount_x2 * 3 + offset;
        let delta = get_u16(data, delta_off)? as u32;
        let range = get_u16(data, range_off)? as u32;
        if range == 0 {
            return Ok((cp + delta) & 0xffff);
        }
        let glyph_off = offsets::BASE_GLYPH_OFF
            + segcount_x2 * 3
            + offset
            + range as usize
            + (cp - start_cp) as usize * 2;
        return get_u16(data, glyph_off).map(|u| (u as u32 + delta) & 0xffff);
    }
    Ok(0)
}

fn gid_for_cp_format_12(data: &[u8], cp: u32) -> Result<u32> {
    let num_groups = get_u32(data, offsets::NUM_GROUPS)? as usize;
    for offset in (0..num_groups * sizes::TABLE_12_RECORD).step_by(sizes::TABLE_12_RECORD) {
        let start_cp = get_u32(data, offsets::REC_START_CODE + offset)?;
        if cp < start_cp {
            break;
        }
        let end_cp = get_u32(data, offsets::REC_END_CODE + offset)?;
        if cp > end_cp {
            continue;
        }
        let start_glyph = get_u32(data, offsets::REC_START_GLYPH + offset)?;
        return Ok((cp - start_cp) + start_glyph);
    }
    Ok(0)
}

// Struct for debug printing
#[allow(non_snake_case)]
#[derive(Debug)]
struct SequentialMapGroup {
    startCharCode: u32,
    endCharCode: u32,
    startGlyphID: u32,
}

impl std::fmt::Debug for Subtable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Subtable::Format4(data) => {
                let slice = &data;
                let segcount_x2 = get_u16(slice, offsets::SEGCOUNT_X2).unwrap() as usize;
                f.debug_struct("Subtable")
                    .field("format", &get_u16(slice, 0).unwrap())
                    .field("length", &get_u16(slice, 2).unwrap())
                    .field("segCountX2", &segcount_x2)
                    /*
                    .field(
                        "endCode",
                        &(0..segcount_x2)
                            .step_by(2)
                            .map(|offset| get_u16(slice, offsets::BASE_ENDCODE + offset).unwrap())
                            .collect::<Vec<_>>(),
                    )
                    .field(
                        "startCode",
                        &(segcount_x2..segcount_x2 * 2)
                            .step_by(2)
                            .map(|offset| get_u16(slice, offsets::BASE_STARTCODE + offset).unwrap())
                            .collect::<Vec<_>>(),
                    )
                    .field(
                        "idDelta",
                        &(segcount_x2 * 2..segcount_x2 * 3)
                            .step_by(2)
                            .map(|offset| get_i16(slice, offsets::BASE_DELTA + offset).unwrap())
                            .collect::<Vec<_>>(),
                    )
                    .field(
                        "idRangeOffsets",
                        &(segcount_x2 * 3..segcount_x2 * 4)
                            .step_by(2)
                            .map(|offset| get_u16(slice, offsets::BASE_RANGE_OFF + offset).unwrap())
                            .collect::<Vec<_>>(),
                    )
                    */
                    .finish()
            }
            Subtable::Format12(data) => {
                let slice = &data;
                let num_groups = get_u32(slice, offsets::NUM_GROUPS).unwrap() as usize;
                f.debug_struct("Subtable")
                    .field("format", &get_u16(slice, 0).unwrap())
                    .field("length", &get_u32(slice, 4).unwrap())
                    .field("num_groups", &num_groups)
                    /*
                    .field(
                        "groups",
                        &(0..num_groups * sizes::TABLE_12_RECORD)
                            .step_by(sizes::TABLE_12_RECORD)
                            .map(|offset| SequentialMapGroup {
                                startCharCode: get_u32(slice, offsets::REC_START_CODE + offset)
                                    .unwrap(),
                                endCharCode: get_u32(slice, offsets::REC_END_CODE + offset)
                                    .unwrap(),
                                startGlyphID: get_u32(slice, offsets::REC_START_GLYPH + offset)
                                    .unwrap(),
                            })
                            .collect::<Vec<_>>(),
                    )
                    */
                    .finish()
            }
        }
    }
}

mod offsets {
    pub(super) const NUM_TABLES: usize = 2;
    pub(super) const ENCODING_RECORDS: usize = 4;
    pub(super) const SUBTABLE_OFF: usize = 4;

    pub(super) const SEGCOUNT_X2: usize = 6;
    pub(super) const BASE_ENDCODE: usize = 14;
    pub(super) const BASE_STARTCODE: usize = 16;
    pub(super) const BASE_DELTA: usize = 16;
    pub(super) const BASE_RANGE_OFF: usize = 16;
    pub(super) const BASE_GLYPH_OFF: usize = 16;

    pub(super) const NUM_GROUPS: usize = 12;
    pub(super) const REC_START_CODE: usize = 16;
    pub(super) const REC_END_CODE: usize = 20;
    pub(super) const REC_START_GLYPH: usize = 24;
}

mod sizes {
    pub(super) const ENCODING_RECORD: usize = 8;

    pub(super) const TABLE_4_CONSTANT: usize = 16;
    pub(super) const TABLE_4_NONCONST_MULT: usize = 4;

    pub(super) const TABLE_12_HEADER: usize = 16;
    pub(super) const TABLE_12_RECORD: usize = 12;
}
