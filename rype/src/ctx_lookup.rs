// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::classdef::ClassDef;
use crate::coverage::Coverage;
use crate::error::*;
use crate::types::get_u16;

#[derive(Debug)]
pub(crate) struct SequenceLookupRecord {
    sequence_index: u16,
    lookup_list_index: u16,
}

impl SequenceLookupRecord {
    fn load(data: &[u8]) -> Result<SequenceLookupRecord> {
        let sequence_index = get_u16(data, 0)?;
        let lookup_list_index = get_u16(data, 2)?;
        Ok(SequenceLookupRecord {
            sequence_index,
            lookup_list_index,
        })
    }
}

#[derive(Debug)]
pub(crate) struct SequenceRuleTable {
    input_glyph_seq: Vec<u16>,
    lookup_records: Vec<SequenceLookupRecord>,
}

impl SequenceRuleTable {
    fn load(data: &[u8]) -> Result<SequenceRuleTable> {
        let glyph_count = get_u16(data, 0)? as usize - 1;
        let seq_count = get_u16(data, 2)? as usize;
        let mut input_glyph_seq = Vec::new();
        for off in (4..4 + glyph_count * 2).step_by(2) {
            input_glyph_seq.push(get_u16(data, off)?);
        }
        let start = 4 + glyph_count * 2;
        let mut lookup_records = Vec::new();
        for off in (start..start + seq_count * 4).step_by(4) {
            lookup_records.push(SequenceLookupRecord::load(&data[off..])?);
        }
        Ok(SequenceRuleTable {
            input_glyph_seq,
            lookup_records,
        })
    }
}

#[derive(Debug)]
pub(crate) enum SequenceContextFormat {
    Format1 {
        coverage: Coverage,
        seq_rules: Vec<Option<Vec<SequenceRuleTable>>>,
    },
    Format2 {
        coverage: Coverage,
        classdef: ClassDef,
        seq_rules: Vec<Option<Vec<SequenceRuleTable>>>,
    },
    Format3 {
        coverages: Vec<Coverage>,
        lookup_records: Vec<SequenceLookupRecord>,
    },
}

impl SequenceContextFormat {
    pub(crate) fn load(data: &[u8]) -> Result<SequenceContextFormat> {
        match get_u16(data, 0)? {
            1 => {
                let coverage_offset = get_u16(data, 2)? as usize;
                let coverage = Coverage::load(&data[coverage_offset..])?;
                let rule_set_count = get_u16(data, 4)? as usize;
                let mut rule_sets = Vec::new();
                for rule_set_off_off in (6..6 + rule_set_count * 2).step_by(2) {
                    let rule_set_off = get_u16(data, rule_set_off_off)? as usize;
                    if rule_set_off == 0 {
                        rule_sets.push(None);
                    } else {
                        let data = &data[rule_set_off..];
                        let rule_count = get_u16(data, 0)? as usize;
                        let mut rules = Vec::new();
                        for rule_off_off in (2..2 + rule_count * 2).step_by(2) {
                            let rule_off = get_u16(data, rule_off_off)? as usize;
                            rules.push(SequenceRuleTable::load(&data[rule_off..])?);
                        }
                        rule_sets.push(Some(rules));
                    }
                }
                Ok(SequenceContextFormat::Format1 {
                    coverage,
                    seq_rules: rule_sets,
                })
            }
            2 => {
                let coverage_offset = get_u16(data, 2)? as usize;
                let coverage = Coverage::load(&data[coverage_offset..])?;
                let classdef_offset = get_u16(data, 4)? as usize;
                let classdef = ClassDef::load(&data[classdef_offset..])?;
                let rule_set_count = get_u16(data, 6)? as usize;
                let mut rule_sets = Vec::new();
                for rule_set_off_off in (8..8 + rule_set_count * 2).step_by(2) {
                    let rule_set_off = get_u16(data, rule_set_off_off)? as usize;
                    if rule_set_off == 0 {
                        rule_sets.push(None);
                    } else {
                        let data = &data[rule_set_off..];
                        let rule_count = get_u16(data, 0)? as usize;
                        let mut rules = Vec::new();
                        for rule_off_off in (2..2 + rule_count * 2).step_by(2) {
                            let rule_off = get_u16(data, rule_off_off)? as usize;
                            rules.push(SequenceRuleTable::load(&data[rule_off..])?);
                        }
                        rule_sets.push(Some(rules));
                    }
                }
                Ok(SequenceContextFormat::Format2 {
                    coverage,
                    classdef,
                    seq_rules: rule_sets,
                })
            }
            3 => {
                let glyph_count = get_u16(data, 2)? as usize;
                let seq_count = get_u16(data, 4)? as usize;
                let mut coverages = Vec::new();
                for off in (6..6 + glyph_count * 2).step_by(2) {
                    let cov_off = get_u16(data, off)? as usize;
                    coverages.push(Coverage::load(&data[cov_off..])?);
                }
                let start = 6 + glyph_count * 2;
                let mut lookup_records = Vec::new();
                for off in (start..start + seq_count * 4).step_by(4) {
                    lookup_records.push(SequenceLookupRecord::load(&data[off..])?);
                }
                Ok(SequenceContextFormat::Format3 {
                    coverages,
                    lookup_records,
                })
            }
            _ => {
                unimplemented!()
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct ChainedSequenceRuleTable {
    backtrack_glyphs: Vec<u16>,
    input_glyphs: Vec<u16>,
    lookahead_glyphs: Vec<u16>,
    lookup_records: Vec<SequenceLookupRecord>,
}

impl ChainedSequenceRuleTable {
    fn load(data: &[u8]) -> Result<ChainedSequenceRuleTable> {
        let backtrack_glyph_count = get_u16(data, 0)? as usize;
        let mut backtrack_glyphs = Vec::new();
        for off in (2..2 + backtrack_glyph_count * 2).step_by(2) {
            backtrack_glyphs.push(get_u16(data, off)?);
        }
        let input_off = 2 + backtrack_glyph_count * 2;
        let input_glyph_count = get_u16(data, input_off)? as usize - 1;
        let mut input_glyphs = Vec::new();
        for off in (input_off + 2..input_off + 2 + input_glyph_count * 2).step_by(2) {
            input_glyphs.push(get_u16(data, off)?);
        }
        let lookahead_off = input_off + 2 + input_glyph_count * 2;
        let lookahead_glyph_count = get_u16(data, lookahead_off)? as usize;
        let mut lookahead_glyphs = Vec::new();
        for off in (lookahead_off + 2..lookahead_off + 2 + lookahead_glyph_count * 2).step_by(2) {
            lookahead_glyphs.push(get_u16(data, off)?);
        }
        let seq_off = lookahead_off + 2 + lookahead_glyph_count * 2;
        let seq_count = get_u16(data, seq_off)? as usize;
        let mut lookup_records = Vec::new();
        for off in (seq_off + 2..seq_off + 2 + seq_count * 4).step_by(4) {
            lookup_records.push(SequenceLookupRecord::load(&data[off..])?);
        }
        Ok(ChainedSequenceRuleTable {
            backtrack_glyphs,
            input_glyphs,
            lookahead_glyphs,
            lookup_records,
        })
    }
}

#[derive(Debug)]
pub(crate) enum ChainedSequenceContextFormat {
    Format1 {
        coverage: Coverage,
        seq_rules: Vec<Option<Vec<ChainedSequenceRuleTable>>>,
    },
    Format2 {
        coverage: Coverage,
        backtrack_classdef: ClassDef,
        input_classdef: ClassDef,
        lookahead_classdef: ClassDef,
        class_seq_rules: Vec<Option<Vec<ChainedSequenceRuleTable>>>,
    },
    Format3 {
        backtrack_coverages: Vec<Coverage>,
        input_coverages: Vec<Coverage>,
        lookahead_coverages: Vec<Coverage>,
        lookup_records: Vec<SequenceLookupRecord>,
    },
}

impl ChainedSequenceContextFormat {
    pub(crate) fn load(data: &[u8]) -> Result<ChainedSequenceContextFormat> {
        match get_u16(data, 0)? {
            1 => {
                let coverage_offset = get_u16(data, 2)? as usize;
                let coverage = Coverage::load(&data[coverage_offset..])?;
                let rule_set_count = get_u16(data, 4)? as usize;
                let mut rule_sets = Vec::new();
                for rule_set_off_off in (6..6 + rule_set_count * 2).step_by(2) {
                    let rule_set_off = get_u16(data, rule_set_off_off)? as usize;
                    if rule_set_off == 0 {
                        rule_sets.push(None);
                    } else {
                        let data = &data[rule_set_off..];
                        let rule_count = get_u16(data, 0)? as usize;
                        let mut rules = Vec::new();
                        for rule_off_off in (2..2 + rule_count * 2).step_by(2) {
                            let rule_off = get_u16(data, rule_off_off)? as usize;
                            rules.push(ChainedSequenceRuleTable::load(&data[rule_off..])?);
                        }
                        rule_sets.push(Some(rules));
                    }
                }
                Ok(ChainedSequenceContextFormat::Format1 {
                    coverage,
                    seq_rules: rule_sets,
                })
            }
            2 => {
                let coverage_offset = get_u16(data, 2)? as usize;
                let coverage = Coverage::load(&data[coverage_offset..])?;
                let backtrack_classdef_offset = get_u16(data, 4)? as usize;
                let backtrack_classdef = ClassDef::load(&data[backtrack_classdef_offset..])?;
                let input_classdef_offset = get_u16(data, 6)? as usize;
                let input_classdef = ClassDef::load(&data[input_classdef_offset..])?;
                let lookahead_classdef_offset = get_u16(data, 8)? as usize;
                let lookahead_classdef = ClassDef::load(&data[lookahead_classdef_offset..])?;
                let rule_set_count = get_u16(data, 10)? as usize;
                let mut rule_sets = Vec::new();
                for rule_set_off_off in (12..12 + rule_set_count * 2).step_by(2) {
                    let rule_set_off = get_u16(data, rule_set_off_off)? as usize;
                    if rule_set_off == 0 {
                        rule_sets.push(None);
                    } else {
                        let data = &data[rule_set_off..];
                        let rule_count = get_u16(data, 0)? as usize;
                        let mut rules = Vec::new();
                        for rule_off_off in (2..2 + rule_count * 2).step_by(2) {
                            let rule_off = get_u16(data, rule_off_off)? as usize;
                            rules.push(ChainedSequenceRuleTable::load(&data[rule_off..])?);
                        }
                        rule_sets.push(Some(rules));
                    }
                }
                Ok(ChainedSequenceContextFormat::Format2 {
                    coverage,
                    backtrack_classdef,
                    input_classdef,
                    lookahead_classdef,
                    class_seq_rules: rule_sets,
                })
            }
            3 => {
                let backtrack_glyph_count = get_u16(data, 2)? as usize;
                let mut backtrack_coverages = Vec::new();
                for off in (4..4 + backtrack_glyph_count * 2).step_by(2) {
                    let cov_off = get_u16(data, off)? as usize;
                    backtrack_coverages.push(Coverage::load(&data[cov_off..])?);
                }

                let input_off = 4 + backtrack_glyph_count * 2;
                let input_glyph_count = get_u16(data, input_off)? as usize;
                let mut input_coverages = Vec::new();
                for off in (input_off + 2..input_off + 2 + input_glyph_count * 2).step_by(2) {
                    let cov_off = get_u16(data, off)? as usize;
                    input_coverages.push(Coverage::load(&data[cov_off..])?);
                }

                let lookahead_off = input_off + 2 + input_glyph_count * 2;
                let lookahead_glyph_count = get_u16(data, input_off)? as usize;
                let mut lookahead_coverages = Vec::new();
                for off in (lookahead_off + 2..input_off + 2 + input_glyph_count * 2).step_by(2) {
                    let cov_off = get_u16(data, off)? as usize;
                    lookahead_coverages.push(Coverage::load(&data[cov_off..])?);
                }

                let seq_off = lookahead_off + 2 + lookahead_glyph_count * 2;
                let seq_count = get_u16(data, seq_off)? as usize;
                let mut lookup_records = Vec::new();
                for off in (seq_off + 2..seq_off + 2 + seq_count * 4).step_by(4) {
                    lookup_records.push(SequenceLookupRecord::load(&data[off..])?);
                }
                Ok(ChainedSequenceContextFormat::Format3 {
                    backtrack_coverages,
                    input_coverages,
                    lookahead_coverages,
                    lookup_records,
                })
            }
            _ => {
                unimplemented!()
            }
        }
    }
}
