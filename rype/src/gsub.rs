// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use crate::coverage::Coverage;
use crate::error::*;
use crate::featurelist::FeatureList;
use crate::lookuplist::{LookupList, LookupSubtable};
use crate::rcbuffer::RcBuf;
use crate::scriptlist::ScriptList;
use crate::types::{get_i16, get_u16};

/// Wrapper around glyph substitution table
#[derive(Debug)]
pub(crate) struct Gsub {
    scriptlist: ScriptList,
    featurelist: FeatureList,
    lookuplist: LookupList<Subtable>,
}

impl Gsub {
    pub(crate) fn load(data: RcBuf) -> Result<Gsub> {
        let slice = &data;
        //let minor_version = get_u16(slice, 2)?;
        let scriptlist_off = get_u16(slice, 4)? as usize;
        let featurelist_off = get_u16(slice, 6)? as usize;
        let lookuplist_off = get_u16(slice, 8)? as usize;
        let scriptlist = ScriptList::load(data.slice(scriptlist_off..))?;
        let featurelist = FeatureList::load(data.slice(featurelist_off..))?;
        let lookuplist = LookupList::load(data.slice(lookuplist_off..))?;
        Ok(Gsub {
            scriptlist,
            featurelist,
            lookuplist,
        })
    }
}

enum Subtable {
    Single(RcBuf),
    Multiple(RcBuf),
    Alternate(RcBuf),
    Ligature(RcBuf),
    Context(RcBuf),
    ChainingContext(RcBuf),
    ExtensionSubstitution(RcBuf),
    ReverseChainingContextSingle(RcBuf),
}

impl LookupSubtable for Subtable {
    fn load(data: RcBuf, lookup_type: u16) -> Result<Subtable> {
        match lookup_type {
            1 => Ok(Subtable::Single(data)),
            2 => Ok(Subtable::Multiple(data)),
            3 => Ok(Subtable::Alternate(data)),
            4 => Ok(Subtable::Ligature(data)),
            5 => Ok(Subtable::Context(data)),
            6 => Ok(Subtable::ChainingContext(data)),
            7 => Ok(Subtable::ExtensionSubstitution(data)),
            8 => Ok(Subtable::ReverseChainingContextSingle(data)),
            _ => Err(Error::Invalid),
        }
    }
}

impl fmt::Debug for Subtable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Subtable::Single(data) => fmt_single(data, f),
            Subtable::Multiple(data) => fmt_multiple(data, f),
            Subtable::Alternate(data) => fmt_alternate(data, f),
            Subtable::Ligature(data) => fmt_ligature(data, f),
            Subtable::Context(data) => fmt_context(data, f),
            Subtable::ChainingContext(data) => fmt_chainingcontext(data, f),
            Subtable::ExtensionSubstitution(data) => fmt_extensionsubstitution(data, f),
            Subtable::ReverseChainingContextSingle(data) => {
                fmt_reversechainingcontextsingle(data, f)
            }
        }
    }
}

fn fmt_single(data: &RcBuf, f: &mut fmt::Formatter) -> fmt::Result {
    let slice = &data;
    let format = get_u16(slice, 0).unwrap();
    let mut tmp = f.debug_struct("Single");
    tmp.field("format", &format).field("coverage", {
        let offset = get_u16(slice, 2).unwrap() as usize;
        &Coverage::load(data.slice(offset..)).unwrap()
    });
    match format {
        1 => {
            tmp.field("deltaGlyphID", &get_i16(slice, 4).unwrap());
        }
        2 => {
            let glyph_count = get_u16(slice, 4).unwrap() as usize;
            tmp.field("glyphCount", &glyph_count).field(
                "substituteGlyphIDs",
                &(6..6 + glyph_count * 2)
                    .step_by(2)
                    .map(|off| get_u16(slice, off).unwrap())
                    .collect::<Vec<_>>(),
            );
        }
        _ => unreachable!("invalid subtable format"),
    }
    tmp.finish()
}

// Sequence table for multiple substitution subtable
struct SequenceTable(RcBuf);

impl fmt::Debug for SequenceTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.0;
        let count = get_u16(slice, 0).unwrap() as usize;
        f.debug_struct("SequenceTable")
            .field("glyphCount", &count)
            .field(
                "substituteGlyphIDs",
                &(2..2 + count * 2)
                    .step_by(2)
                    .map(|off| get_u16(slice, off).unwrap())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

fn fmt_multiple(data: &RcBuf, f: &mut fmt::Formatter) -> fmt::Result {
    let slice = &data;
    let format = get_u16(slice, 0).unwrap();
    if format != 1 {
        unreachable!("invalid subtable format");
    }
    let count = get_u16(slice, 4).unwrap() as usize;
    f.debug_struct("Multiple")
        .field("format", &format)
        .field("coverage", {
            let offset = get_u16(slice, 2).unwrap() as usize;
            &Coverage::load(data.slice(offset..)).unwrap()
        })
        .field("sequnceCount", &count)
        .field(
            "sequences",
            &(6..6 + count * 2)
                .step_by(2)
                .map(|off| {
                    let offset = get_u16(slice, off).unwrap() as usize;
                    SequenceTable(data.slice(offset..))
                })
                .collect::<Vec<_>>(),
        )
        .finish()
}

struct AlternateSetTable(RcBuf);

impl fmt::Debug for AlternateSetTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.0;
        let count = get_u16(slice, 0).unwrap() as usize;
        f.debug_struct("AlternateSetTable")
            .field("glyphCount", &count)
            .field(
                "alternateGlyphIDs",
                &(2..2 + count * 2)
                    .step_by(2)
                    .map(|off| get_u16(slice, off).unwrap())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

fn fmt_alternate(data: &RcBuf, f: &mut fmt::Formatter) -> fmt::Result {
    let slice = &data;
    let format = get_u16(slice, 0).unwrap();
    if format != 1 {
        unreachable!("invalid subtable format");
    }
    let count = get_u16(slice, 4).unwrap() as usize;
    f.debug_struct("Alternate")
        .field("format", &format)
        .field("coverage", {
            let offset = get_u16(slice, 2).unwrap() as usize;
            &Coverage::load(data.slice(offset..)).unwrap()
        })
        .field("alternateSetCount", &count)
        .field(
            "alternateSets",
            &(6..6 + count * 2)
                .step_by(2)
                .map(|off| {
                    let offset = get_u16(slice, off).unwrap() as usize;
                    AlternateSetTable(data.slice(offset..))
                })
                .collect::<Vec<_>>(),
        )
        .finish()
}

struct LigatureTable(RcBuf);

impl fmt::Debug for LigatureTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.0;
        let count = get_u16(slice, 2).unwrap() as usize;
        f.debug_struct("LigatureTable")
            .field("ligatureGlyph", &get_u16(slice, 0).unwrap())
            .field("componentCount", &count)
            .field(
                "componentGlyphIDs",
                &(4..4 + count * 2)
                    .step_by(2)
                    .map(|off| get_u16(slice, off).unwrap())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

struct LigatureSetTable(RcBuf);

impl fmt::Debug for LigatureSetTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.0;
        let count = get_u16(slice, 0).unwrap() as usize;
        f.debug_struct("LigatureSetTable")
            .field("ligatureCount", &count)
            .field(
                "ligatureTables",
                &(2..2 + count * 2)
                    .step_by(2)
                    .map(|off| {
                        let offset = get_u16(slice, off).unwrap() as usize;
                        LigatureTable(self.0.slice(offset..))
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

fn fmt_ligature(data: &RcBuf, f: &mut fmt::Formatter) -> fmt::Result {
    let slice = &data;
    let format = get_u16(slice, 0).unwrap();
    if format != 1 {
        unreachable!("invalid subtable format");
    }
    let count = get_u16(slice, 4).unwrap() as usize;
    f.debug_struct("Ligature")
        .field("format", &format)
        .field("coverage", {
            let offset = get_u16(slice, 2).unwrap() as usize;
            &Coverage::load(data.slice(offset..)).unwrap()
        })
        .field("ligatureSetCount", &count)
        .field(
            "ligatureSets",
            &(6..6 + count * 2)
                .step_by(2)
                .map(|off| {
                    let offset = get_u16(slice, off).unwrap() as usize;
                    LigatureSetTable(data.slice(offset..))
                })
                .collect::<Vec<_>>(),
        )
        .finish()
}

#[derive(Debug)]
struct SequenceLookupRecord {
    sequence_index: u16,
    lookup_list_index: u16,
}

struct SequenceRuleTable(RcBuf);

impl fmt::Debug for SequenceRuleTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.0;
        let glyph_count = get_u16(slice, 0).unwrap() as usize;
        let seq_lookup_count = get_u16(slice, 2).unwrap() as usize;
        f.debug_struct("SequenceRuleTable")
            .field("glyphCount", &glyph_count)
            .field("seqLookupCount", &seq_lookup_count)
            .field(
                "inputSequence",
                &(4..4 + glyph_count * 2)
                    .step_by(2)
                    .map(|off| get_u16(slice, off).unwrap())
                    .collect::<Vec<_>>(),
            )
            .field(
                "seqLookupRecords",
                &(4 + glyph_count * 2..4 + glyph_count * 2 + seq_lookup_count * 4)
                    .step_by(4)
                    .map(|off| {
                        let sequence_index = get_u16(slice, off).unwrap();
                        let lookup_list_index = get_u16(slice, off + 2).unwrap();
                        SequenceLookupRecord {
                            sequence_index,
                            lookup_list_index,
                        }
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

struct SequenceRuleSetTable(RcBuf);

impl fmt::Debug for SequenceRuleSetTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.0;
        let count = get_u16(slice, 0).unwrap() as usize;
        f.debug_struct("SequenceRuleSetTable")
            .field("seqRuleCount", &count)
            .field(
                "seqRules",
                &(2..2 + count * 2)
                    .step_by(2)
                    .map(|off| {
                        let offset = get_u16(slice, off).unwrap() as usize;
                        SequenceRuleTable(self.0.slice(offset..))
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

fn fmt_context(data: &RcBuf, f: &mut fmt::Formatter) -> fmt::Result {
    let slice = &data;
    let format = get_u16(slice, 0).unwrap();
    let mut tmp = f.debug_struct("Context");
    tmp.field("format", &format).field("coverage", {
        let offset = get_u16(slice, 2).unwrap() as usize;
        &Coverage::load(data.slice(offset..)).unwrap()
    });
    match format {
        1 => {
            let count = get_u16(slice, 4).unwrap() as usize;
            tmp.field("seqRuleSetCount", &count).field(
                "seqRuleSets",
                &(6..6 + count * 2)
                    .step_by(2)
                    .map(|off| {
                        let offset = get_u16(slice, off).unwrap() as usize;
                        SequenceRuleSetTable(data.slice(offset..))
                    })
                    .collect::<Vec<_>>(),
            );
        }
        2 => {}
        3 => {}
        _ => unreachable!("invalid subtable format"),
    }
    tmp.finish()
}

fn fmt_chainingcontext(data: &RcBuf, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("ChainingContext").finish()
}

fn fmt_extensionsubstitution(data: &RcBuf, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("ExtensionSubstitution").finish()
}

fn fmt_reversechainingcontextsingle(data: &RcBuf, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("ReverseChainingContextSingle").finish()
}
