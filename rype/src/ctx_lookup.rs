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

#[derive(Debug)]
pub(crate) struct SequenceRuleTable {
    input_glyph_seq: Vec<u16>,
    lookup_records: Vec<SequenceLookupRecord>,
}

impl SequenceRuleTable {
    pub(crate) fn load(data: &[u8]) -> Result<SequenceRuleTable> {
        let glyph_count = get_u16(data, 0)? as usize - 1;
        let seq_count = get_u16(data, 2)? as usize;
        let mut input_glyph_seq = Vec::new();
        for off in (4..4 + glyph_count * 2).step_by(2) {
            input_glyph_seq.push(get_u16(data, off)?);
        }
        let start = 4 + glyph_count * 2;
        let mut lookup_records = Vec::new();
        for off in (start..start + seq_count * 4).step_by(4) {
            let sequence_index = get_u16(data, off)?;
            let lookup_list_index = get_u16(data, off + 2)?;
            lookup_records.push(SequenceLookupRecord {
                sequence_index,
                lookup_list_index,
            });
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
        seq_rules: Vec<Vec<SequenceRuleTable>>,
    },
    Format2 {
        coverage: Coverage,
        classdef: ClassDef,
        seq_rules: Vec<Vec<SequenceRuleTable>>,
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

                    let data = &data[rule_set_off..];
                    let rule_count = get_u16(data, 0)? as usize;
                    let mut rules = Vec::new();
                    for rule_off_off in (2..2 + rule_count * 2).step_by(2) {
                        let rule_off = get_u16(data, rule_off_off)? as usize;
                        rules.push(SequenceRuleTable::load(&data[rule_off..])?);
                    }
                    rule_sets.push(rules);
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

                    let data = &data[rule_set_off..];
                    let rule_count = get_u16(data, 0)? as usize;
                    let mut rules = Vec::new();
                    for rule_off_off in (2..2 + rule_count * 2).step_by(2) {
                        let rule_off = get_u16(data, rule_off_off)? as usize;
                        rules.push(SequenceRuleTable::load(&data[rule_off..])?);
                    }
                    rule_sets.push(rules);
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
                    let sequence_index = get_u16(data, off)?;
                    let lookup_list_index = get_u16(data, off + 2)?;
                    lookup_records.push(SequenceLookupRecord {
                        sequence_index,
                        lookup_list_index,
                    });
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
