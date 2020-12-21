// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::rc::Rc;

use fnv::FnvHashSet;

use crate::common::GlyphID;
use crate::coverage::Coverage;
use crate::ctx_lookup::{
    ChainedSequenceContextFormat, SequenceContextFormat, SequenceLookupRecord,
};
use crate::error::*;
use crate::featurelist::FeatureList;
use crate::gdef::Gdef;
use crate::lookuplist::{GlyphData, LookupList, LookupSubtable};
use crate::scriptlist::ScriptList;
use crate::types::{get_i16, get_u16, get_u32, Tag};
use crate::Script;

/// Wrapper around glyph substitution table
#[derive(Debug)]
pub(crate) struct Gsub {
    scriptlist: ScriptList,
    featurelist: FeatureList,
    lookuplist: LookupList<Subtable>,
    gdef: Option<Rc<Gdef>>,
}

impl Gsub {
    pub(crate) fn load(data: &[u8], gdef: Option<Rc<Gdef>>) -> Result<Gsub> {
        //let minor_version = get_u16(slice, 2)?;
        let scriptlist_off = get_u16(data, 4)? as usize;
        let featurelist_off = get_u16(data, 6)? as usize;
        let lookuplist_off = get_u16(data, 8)? as usize;
        let scriptlist = ScriptList::load(&data[scriptlist_off..])?;
        let featurelist = FeatureList::load(&data[featurelist_off..])?;
        let lookuplist = LookupList::load(&data[lookuplist_off..])?;
        Ok(Gsub {
            scriptlist,
            featurelist,
            lookuplist,
            gdef,
        })
    }

    pub(crate) fn substitute(
        &self,
        glyphs: &mut Vec<GlyphID>,
        script: Script,
        enabled_features: &FnvHashSet<Tag>,
    ) {
        // Get feature indices
        let feature_indices = self.scriptlist.feature_indices(script);
        let features = feature_indices
            .iter()
            .map(|i| &self.featurelist[*i as usize])
            .filter(|(tag, _)| enabled_features.contains(tag));
        // Select lookups
        let mut lookuplist_indices = features.flat_map(|(_, is)| is).collect::<Vec<_>>();
        lookuplist_indices.sort();
        lookuplist_indices.dedup();

        let gdef_ref = self.gdef.as_ref().map(|g| g.as_ref());
        // Apply all lookups
        for idx in lookuplist_indices {
            let lookup = &self.lookuplist[*idx as usize];
            lookup.apply(glyphs, gdef_ref, &self.lookuplist);
        }
    }
}

#[derive(Debug)]
enum SingleFormat {
    Format1 { delta: i16 },
    Format2 { subst: Vec<u16> },
}

#[derive(Debug)]
struct LigatureTable {
    ligature_glyph: u16,
    component_glyphs: Vec<u16>,
}

impl LigatureTable {
    fn load(data: &[u8]) -> Result<LigatureTable> {
        let ligature_glyph = get_u16(data, 0)?;
        let count = get_u16(data, 2)? as usize;
        let mut component_glyphs = Vec::new();
        for off in (4..4 + count * 2).step_by(2) {
            component_glyphs.push(get_u16(data, off)?);
        }
        Ok(LigatureTable {
            ligature_glyph,
            component_glyphs,
        })
    }
}

#[derive(Debug)]
enum Subtable {
    Single {
        coverage: Coverage,
        format: SingleFormat,
    },
    Multiple {
        coverage: Coverage,
        sequences: Vec<Vec<u16>>,
    },
    Alternate {
        coverage: Coverage,
        alternate_sets: Vec<Vec<u16>>,
    },
    Ligature {
        coverage: Coverage,
        ligature_sets: Vec<Vec<LigatureTable>>,
    },
    Context(SequenceContextFormat),
    ChainedContext(ChainedSequenceContextFormat),
    ReverseChainedContextSingle {
        coverage: Coverage,
        backtrack_coverages: Vec<Coverage>,
        lookahead_coverages: Vec<Coverage>,
        subst_glyphs: Vec<u16>,
    },
}

impl GlyphData for GlyphID {
    fn glyph(&self) -> GlyphID {
        *self
    }
}

impl LookupSubtable for Subtable {
    type GlyphData = GlyphID;

    fn is_recursive(lookup_type: u16) -> bool {
        match lookup_type {
            5 | 6 => true,
            _ => false,
        }
    }

    fn load(data: &[u8], lookup_type: u16) -> Result<Subtable> {
        match lookup_type {
            1 => Subtable::load_single(data),
            2 => Subtable::load_multiple(data),
            3 => Subtable::load_alternate(data),
            4 => Subtable::load_ligature(data),
            5 => Subtable::load_context(data),
            6 => Subtable::load_chained_context(data),
            7 => Subtable::load_extension_substitution(data),
            8 => Subtable::load_reverse_chained_context_single(data),
            _ => Err(Error::Invalid),
        }
    }

    fn apply(&self, glyph_seq: &mut Vec<GlyphID>, idx: usize) -> Option<usize> {
        let glyph = glyph_seq[idx];
        let next_idx = idx + 1;
        let rest = &glyph_seq[next_idx..];
        match self {
            Subtable::Single { coverage, format } => {
                coverage.for_glyph(glyph).map(|ci| match format {
                    SingleFormat::Format1 { delta } => {
                        glyph_seq[idx] = GlyphID((glyph.0 as i32 + *delta as i32) as u32);
                        1
                    }
                    SingleFormat::Format2 { subst } => {
                        glyph_seq[idx] = GlyphID(subst[ci] as u32);
                        1
                    }
                })
            }
            Subtable::Multiple {
                coverage,
                sequences,
            } => coverage.for_glyph(glyph).map(|ci| {
                glyph_seq.splice(idx..=idx, sequences[ci].iter().map(|x| GlyphID(*x as u32)));
                sequences[ci].len()
            }),
            Subtable::Alternate {
                coverage,
                alternate_sets,
            } => {
                // TODO: Do something here?
                None
            }
            Subtable::Ligature {
                coverage,
                ligature_sets,
            } => {
                if let Some(ci) = coverage.for_glyph(glyph) {
                    'outer: for option in &ligature_sets[ci] {
                        if rest.len() < option.component_glyphs.len() {
                            continue;
                        }
                        for (g, tgt) in rest.iter().zip(&option.component_glyphs) {
                            if g.0 != *tgt as u32 {
                                continue 'outer;
                            }
                        }
                        glyph_seq[idx] = GlyphID(option.ligature_glyph as u32);
                        return Some(1);
                    }
                    None
                } else {
                    None
                }
            }
            Subtable::Context(_) | Subtable::ChainedContext(_) => None,
            Subtable::ReverseChainedContextSingle {
                coverage,
                backtrack_coverages,
                lookahead_coverages,
                subst_glyphs,
            } => {
                if backtrack_coverages.len() > idx || lookahead_coverages.len() > rest.len() {
                    None
                } else {
                    if let Some(ci) = coverage.for_glyph(glyph) {
                        for i in 0..backtrack_coverages.len() {
                            if backtrack_coverages[i]
                                .for_glyph(glyph_seq[idx - i - 1])
                                .is_none()
                            {
                                return None;
                            }
                        }
                        for i in 0..lookahead_coverages.len() {
                            if lookahead_coverages[i].for_glyph(rest[i]).is_none() {
                                return None;
                            }
                        }
                        glyph_seq[idx] = GlyphID(subst_glyphs[ci] as u32);
                        Some(1)
                    } else {
                        None
                    }
                }
            }
        }
    }

    fn apply_recursive(
        &self,
        glyph_seq: &[Self::GlyphData],
        cur_idx: usize,
    ) -> Option<(&[SequenceLookupRecord], usize)> {
        match self {
            Subtable::Context(fmt) => fmt.apply(glyph_seq, cur_idx),
            Subtable::ChainedContext(fmt) => fmt.apply(glyph_seq, cur_idx),
            _ => None,
        }
    }
}

impl Subtable {
    fn load_single(data: &[u8]) -> Result<Subtable> {
        let coverage_offset = get_u16(data, 2)? as usize;
        let coverage = Coverage::load(&data[coverage_offset..])?;
        let format = match get_u16(data, 0)? {
            1 => SingleFormat::Format1 {
                delta: get_i16(data, 4)?,
            },
            2 => {
                let glyph_count = get_u16(data, 4)? as usize;
                let mut subst = Vec::new();
                for off in (6..6 + glyph_count * 2).step_by(2) {
                    subst.push(get_u16(data, off)?);
                }
                SingleFormat::Format2 { subst }
            }
            _ => return Err(Error::Invalid),
        };
        Ok(Subtable::Single { coverage, format })
    }

    fn load_multiple(data: &[u8]) -> Result<Subtable> {
        let coverage_offset = get_u16(data, 2)? as usize;
        let coverage = Coverage::load(&data[coverage_offset..])?;
        if get_u16(data, 0)? != 1 {
            return Err(Error::Invalid);
        }
        let mut sequences = Vec::new();
        let seq_tab_count = get_u16(data, 4)? as usize;
        for seq_tab_off_off in (6..6 + seq_tab_count * 2).step_by(2) {
            let seq_tab_off = get_u16(data, seq_tab_off_off)? as usize;
            let data = &data[seq_tab_off..];
            let glyph_count = get_u16(data, 0)? as usize;
            let mut glyphs = Vec::new();
            for off in (2..2 + glyph_count * 2).step_by(2) {
                glyphs.push(get_u16(data, off)?);
            }
            sequences.push(glyphs);
        }
        Ok(Subtable::Multiple {
            coverage,
            sequences,
        })
    }

    fn load_alternate(data: &[u8]) -> Result<Subtable> {
        let coverage_offset = get_u16(data, 2)? as usize;
        let coverage = Coverage::load(&data[coverage_offset..])?;
        if get_u16(data, 0)? != 1 {
            return Err(Error::Invalid);
        }
        let mut alternate_sets = Vec::new();
        let alt_set_count = get_u16(data, 4)? as usize;
        for alt_set_off_off in (6..6 + alt_set_count * 2).step_by(2) {
            let alt_set_off = get_u16(data, alt_set_off_off)? as usize;
            let data = &data[alt_set_off..];
            let glyph_count = get_u16(data, 0)? as usize;
            let mut glyphs = Vec::new();
            for off in (2..2 + glyph_count * 2).step_by(2) {
                glyphs.push(get_u16(data, off)?);
            }
            alternate_sets.push(glyphs);
        }
        Ok(Subtable::Alternate {
            coverage,
            alternate_sets,
        })
    }

    fn load_ligature(data: &[u8]) -> Result<Subtable> {
        let coverage_offset = get_u16(data, 2)? as usize;
        let coverage = Coverage::load(&data[coverage_offset..])?;
        if get_u16(data, 0)? != 1 {
            return Err(Error::Invalid);
        }
        let lig_set_count = get_u16(data, 4)? as usize;
        let mut ligature_sets = Vec::new();
        for lig_set_off_off in (6..6 + lig_set_count * 2).step_by(2) {
            let lig_set_off = get_u16(data, lig_set_off_off)? as usize;

            let data = &data[lig_set_off..];
            let lig_tab_count = get_u16(data, 0)? as usize;
            let mut lig_tables = Vec::new();
            for lig_tab_off_off in (2..2 + lig_tab_count * 2).step_by(2) {
                let lig_tab_off = get_u16(data, lig_tab_off_off)? as usize;
                lig_tables.push(LigatureTable::load(&data[lig_tab_off..])?);
            }
            ligature_sets.push(lig_tables);
        }
        Ok(Subtable::Ligature {
            coverage,
            ligature_sets,
        })
    }

    fn load_context(data: &[u8]) -> Result<Subtable> {
        SequenceContextFormat::load(data).map(|f| Subtable::Context(f))
    }

    fn load_chained_context(data: &[u8]) -> Result<Subtable> {
        ChainedSequenceContextFormat::load(data).map(|f| Subtable::ChainedContext(f))
    }

    fn load_extension_substitution(data: &[u8]) -> Result<Subtable> {
        if get_u16(data, 0)? != 1 {
            return Err(Error::Invalid);
        }
        let typ = get_u16(data, 2)?;
        let offset = get_u32(data, 4)? as usize;
        Subtable::load(&data[offset..], typ)
    }

    fn load_reverse_chained_context_single(data: &[u8]) -> Result<Subtable> {
        if get_u16(data, 0)? != 1 {
            return Err(Error::Invalid);
        }
        let coverage_offset = get_u16(data, 2)? as usize;
        let coverage = Coverage::load(&data[coverage_offset..])?;

        let backtrack_glyph_count = get_u16(data, 4)? as usize;
        let mut backtrack_coverages = Vec::new();
        for off in (6..6 + backtrack_glyph_count * 2).step_by(2) {
            let cov_off = get_u16(data, off)? as usize;
            backtrack_coverages.push(Coverage::load(&data[cov_off..])?);
        }

        let lookahead_off = 6 + backtrack_glyph_count * 2;
        let lookahead_glyph_count = get_u16(data, lookahead_off)? as usize;
        let mut lookahead_coverages = Vec::new();
        for off in (lookahead_off + 2..lookahead_off + 2 + lookahead_glyph_count * 2).step_by(2) {
            let cov_off = get_u16(data, off)? as usize;
            lookahead_coverages.push(Coverage::load(&data[cov_off..])?);
        }

        let glyph_off = lookahead_off + 2 + lookahead_glyph_count * 2;
        let glyph_count = get_u16(data, glyph_off)? as usize;
        let mut subst_glyphs = Vec::new();
        for off in (glyph_off + 2..glyph_off + 2 + glyph_count * 2).step_by(2) {
            subst_glyphs.push(get_u16(data, off)?);
        }

        Ok(Subtable::ReverseChainedContextSingle {
            coverage,
            backtrack_coverages,
            lookahead_coverages,
            subst_glyphs,
        })
    }
}
