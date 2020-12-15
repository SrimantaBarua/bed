// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::coverage::Coverage;
use crate::ctx_lookup::{ChainedSequenceContextFormat, SequenceContextFormat};
use crate::error::*;
use crate::featurelist::FeatureList;
use crate::lookuplist::{LookupList, LookupSubtable};
use crate::scriptlist::ScriptList;
use crate::types::{get_i16, get_u16, get_u32};

/// Wrapper around glyph substitution table
#[derive(Debug)]
pub(crate) struct Gsub {
    scriptlist: ScriptList,
    featurelist: FeatureList,
    lookuplist: LookupList<Subtable>,
}

impl Gsub {
    pub(crate) fn load(data: &[u8]) -> Result<Gsub> {
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
        })
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

impl LookupSubtable for Subtable {
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
                let glyph_count = get_u16(data, 4).unwrap() as usize;
                let mut subst = Vec::new();
                for off in (6..6 + glyph_count * 2).step_by(2) {
                    subst.push(get_u16(data, off)?);
                }
                SingleFormat::Format2 { subst }
            }
            _ => panic!("invalid subtable format"),
        };
        Ok(Subtable::Single { coverage, format })
    }

    fn load_multiple(data: &[u8]) -> Result<Subtable> {
        let coverage_offset = get_u16(data, 2)? as usize;
        let coverage = Coverage::load(&data[coverage_offset..])?;
        if get_u16(data, 0)? != 1 {
            panic!("invalid subtable format");
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
            panic!("invalid subtable format");
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
            panic!("invalid subtable format");
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
            panic!("invalid subtable format");
        }
        let typ = get_u16(data, 2)?;
        let offset = get_u32(data, 4)? as usize;
        Subtable::load(&data[offset..], typ)
    }

    fn load_reverse_chained_context_single(data: &[u8]) -> Result<Subtable> {
        if get_u16(data, 0)? != 1 {
            panic!("invalid subtable format");
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
