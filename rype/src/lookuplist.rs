// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::error::*;
use crate::types::get_u16;

#[derive(Debug)]
pub(crate) struct LookupList<T: LookupSubtable>(Vec<LookupTable<T>>);

impl<T: LookupSubtable> LookupList<T> {
    pub(crate) fn load(data: &[u8]) -> Result<LookupList<T>> {
        let count = get_u16(data, 0)? as usize;
        let mut tables = Vec::new();
        for off in (2..2 + count * 2).step_by(2) {
            let offset = get_u16(data, off)? as usize;
            tables.push(LookupTable::load(&data[offset..])?);
        }
        Ok(LookupList(tables))
    }
}

#[derive(Debug)]
struct LookupTable<T: LookupSubtable> {
    lookup_type: u16,
    lookup_flag: LookupFlag,
    mark_attachment_type_mask: u8,
    mark_filtering_set: Option<u16>,
    subtables: Vec<T>,
}

impl<T: LookupSubtable> LookupTable<T> {
    fn load(data: &[u8]) -> Result<LookupTable<T>> {
        if data.len() < 6 {
            return Err(Error::Invalid);
        }
        let mut subtables = Vec::new();
        let lookup_type = get_u16(data, 0)?;
        let mark_attachment_type_mask = data[2];
        let lookup_flag = LookupFlag::from_bits_truncate(data[3]);
        let count = get_u16(data, 4)? as usize;
        for off in (6..6 + count * 2).step_by(2) {
            let offset = get_u16(data, off)? as usize;
            subtables.push(T::load(&data[offset..], lookup_type)?);
        }
        let mark_filtering_set = if lookup_flag.contains(LookupFlag::USE_MARK_FILTERING_SET) {
            Some(get_u16(data, 6 + count * 2)?)
        } else {
            None
        };
        Ok(LookupTable {
            lookup_type,
            lookup_flag,
            mark_attachment_type_mask,
            mark_filtering_set,
            subtables,
        })
    }
}

bitflags! {
    struct LookupFlag : u8 {
        const RIGHT_TO_LEFT          = 0x01;
        const IGNORE_BASE_GLYPHS     = 0x02;
        const IGNORE_LIGATURES       = 0x04;
        const IGNORE_MARKS           = 0x08;
        const USE_MARK_FILTERING_SET = 0x10;
    }
}

pub(crate) trait LookupSubtable: Sized + std::fmt::Debug {
    fn load(data: &[u8], lookup_type: u16) -> Result<Self>;
}
