// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ops::Index;

use crate::common::GlyphID;
use crate::ctx_lookup::SequenceLookupRecord;
use crate::error::*;
use crate::gdef::{Gdef, GlyphClass};
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

impl<T: LookupSubtable> Index<usize> for LookupList<T> {
    type Output = LookupTable<T>;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

#[derive(Debug)]
pub(crate) struct LookupTable<T: LookupSubtable> {
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

    pub(crate) fn apply(
        &self,
        glyphs: &mut Vec<T::GlyphData>,
        gdef: Option<&Gdef>,
        lookups: &LookupList<T>,
    ) {
        // Go over all current glyphs
        let mut i = 0;
        while i < glyphs.len() {
            let g = glyphs[i].glyph();
            if !self.is_applicable_for_glyph(g, gdef) {
                i += 1;
                continue;
            }
            // Go over all subtables and check applicability for glyph
            // Is this a recursive lookup?
            let mut applied = false;
            if T::is_recursive(self.lookup_type) {
                for subtable in &self.subtables {
                    if let Some((records, len)) = subtable.apply_recursive(glyphs, i) {
                        for record in records {
                            let lookup = &lookups[record.lookup_list_index as usize];
                            let idx = i + record.sequence_index as usize;
                            assert!(idx < glyphs.len(), "index out of bounds");
                            lookup.apply_recursive(glyphs, idx, gdef);
                        }
                        applied = true;
                        i += len;
                    }
                }
            } else {
                for subtable in &self.subtables {
                    if let Some(len) = subtable.apply(glyphs, i) {
                        i += len;
                        applied = true;
                        break;
                    }
                }
            }
            if !applied {
                i += 1;
            }
        }
    }

    fn apply_recursive(&self, glyphs: &mut Vec<T::GlyphData>, idx: usize, gdef: Option<&Gdef>) {
        let g = glyphs[idx].glyph();
        if !self.is_applicable_for_glyph(g, gdef) {
            return;
        }
        // Is this a recursive lookup?
        if T::is_recursive(self.lookup_type) {
            panic!("We should not get here");
        } else {
            for subtable in &self.subtables {
                if subtable.apply(glyphs, idx).is_some() {
                    return;
                }
            }
        }
    }

    fn is_applicable_for_glyph(&self, glyph: GlyphID, gdef: Option<&Gdef>) -> bool {
        let glyph_class = gdef.and_then(|gdef| gdef.glyph_class(glyph));
        // Check lookup applicability for glyph class
        if self.lookup_flag.contains(LookupFlag::IGNORE_BASE_GLYPHS)
            && glyph_class == Some(GlyphClass::Base)
            || self.lookup_flag.contains(LookupFlag::IGNORE_LIGATURES)
                && glyph_class == Some(GlyphClass::Ligature)
            || self.lookup_flag.contains(LookupFlag::IGNORE_MARKS)
                && glyph_class == Some(GlyphClass::Mark)
        {
            return false;
        }
        // If glyph is a mark, apply mark filtering if required
        if glyph_class == Some(GlyphClass::Mark) {
            if self.mark_attachment_type_mask != 0 {
                if let Some(mark_attachment) =
                    gdef.and_then(|gdef| gdef.mark_attachment_class(glyph))
                {
                    if mark_attachment == self.mark_attachment_type_mask as u32 {
                        return false;
                    }
                }
            }
            if self
                .lookup_flag
                .contains(LookupFlag::USE_MARK_FILTERING_SET)
            {
                if let Some(idx) = self.mark_filtering_set {
                    if !gdef
                        .map(|gdef| gdef.glyph_in_mark_set(idx as usize, glyph))
                        .unwrap_or(true)
                    {
                        return false;
                    }
                }
            }
        }
        true
    }
}

bitflags! {
    pub(crate) struct LookupFlag : u8 {
        const RIGHT_TO_LEFT          = 0x01;
        const IGNORE_BASE_GLYPHS     = 0x02;
        const IGNORE_LIGATURES       = 0x04;
        const IGNORE_MARKS           = 0x08;
        const USE_MARK_FILTERING_SET = 0x10;
    }
}

pub(crate) trait GlyphData: std::fmt::Debug {
    fn glyph(&self) -> GlyphID;
}

pub(crate) trait LookupSubtable: Sized + std::fmt::Debug {
    type GlyphData: GlyphData;

    fn is_recursive(lookup_type: u16) -> bool;
    fn load(data: &[u8], lookup_type: u16) -> Result<Self>;
    fn apply(&self, glyph_seq: &mut Vec<Self::GlyphData>, cur_idx: usize) -> Option<usize>;
    fn apply_recursive(
        &self,
        glyph_seq: &[Self::GlyphData],
        cur_idx: usize,
    ) -> Option<(&[SequenceLookupRecord], usize)>;
}
