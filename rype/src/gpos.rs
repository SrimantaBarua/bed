// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::collections::HashMap;

use geom::{point2, Point2D};

use crate::classdef::ClassDef;
use crate::coverage::Coverage;
use crate::ctx_lookup::{ChainedSequenceContextFormat, SequenceContextFormat};
use crate::error::*;
use crate::featurelist::FeatureList;
use crate::lookuplist::{LookupList, LookupSubtable};
use crate::scriptlist::ScriptList;
use crate::types::{get_i16, get_u16, get_u32};

/// Wrapper around glyph substitution table
#[derive(Debug)]
pub(crate) struct Gpos {
    scriptlist: ScriptList,
    featurelist: FeatureList,
    lookuplist: LookupList<Subtable>,
}

impl Gpos {
    pub(crate) fn load(data: &[u8]) -> Result<Gpos> {
        //let minor_version = get_u16(slice, 2)?;
        let scriptlist_off = get_u16(data, 4)? as usize;
        let featurelist_off = get_u16(data, 6)? as usize;
        let lookuplist_off = get_u16(data, 8)? as usize;
        let scriptlist = ScriptList::load(&data[scriptlist_off..])?;
        let featurelist = FeatureList::load(&data[featurelist_off..])?;
        let lookuplist = LookupList::load(&data[lookuplist_off..])?;
        Ok(Gpos {
            scriptlist,
            featurelist,
            lookuplist,
        })
    }
}

#[derive(Debug)]
enum SingleFormat {
    Format1(ValueRecord),
    Format2(Vec<ValueRecord>),
}

#[derive(Debug)]
enum PairFormat {
    Format1(Vec<HashMap<u16, (ValueRecord, ValueRecord)>>),
    Format2 {
        class1: ClassDef,
        class2: ClassDef,
        records: Vec<Vec<(ValueRecord, ValueRecord)>>,
    },
}

#[derive(Debug)]
enum AnchorTable {
    Format1 {
        coord: Point2D<i16>,
    },
    Format2 {
        coord: Point2D<i16>,
        anchor_point: u16, // Index to glyph contour point
    },
    Format3 {
        coord: Point2D<i16>,
        // TODO: Device tables
    },
}

impl AnchorTable {
    fn load(data: &[u8]) -> Result<AnchorTable> {
        match get_u16(data, 0)? {
            1 => Ok(AnchorTable::Format1 {
                coord: point2(get_i16(data, 2)?, get_i16(data, 4)?),
            }),
            2 => Ok(AnchorTable::Format2 {
                coord: point2(get_i16(data, 2)?, get_i16(data, 4)?),
                anchor_point: get_u16(data, 6)?,
            }),
            3 => {
                Ok(AnchorTable::Format3 {
                    coord: point2(get_i16(data, 2)?, get_i16(data, 4)?),
                    // TODO: Device tables
                })
            }
            _ => Err(Error::Invalid),
        }
    }
}

#[derive(Debug)]
struct MarkRecord {
    class: u16,
    anchor: AnchorTable,
}

impl MarkRecord {
    fn load_array(data: &[u8]) -> Result<Vec<MarkRecord>> {
        let count = get_u16(data, 0)? as usize;
        let mut ret = Vec::new();
        for off in (2..2 + count * 4).step_by(4) {
            let class = get_u16(data, off)?;
            let anchor_off = get_u16(data, off + 2)? as usize;
            let anchor = AnchorTable::load(&data[anchor_off..])?;
            ret.push(MarkRecord { class, anchor });
        }
        Ok(ret)
    }
}

#[derive(Debug)]
enum Subtable {
    SingleAdjustment {
        coverage: Coverage,
        format: SingleFormat,
    },
    PairAdjustment {
        coverage: Coverage,
        format: PairFormat,
    },
    CursiveAttachment {
        coverage: Coverage,
        records: Vec<(Option<AnchorTable>, Option<AnchorTable>)>, // Entry-exit records
    },
    MarkToBaseAttachment {
        mark_coverage: Coverage,
        base_coverage: Coverage,
        mark_array: Vec<MarkRecord>,
        base_array: Vec<Vec<Option<AnchorTable>>>,
    },
    MarkToLigatureAttachment {
        mark_coverage: Coverage,
        ligature_coverage: Coverage,
        mark_array: Vec<MarkRecord>,
        ligature_array: Vec<Vec<Vec<Option<AnchorTable>>>>,
    },
    MarkToMarkAttachment {
        mark1_coverage: Coverage,
        mark2_coverage: Coverage,
        mark1_array: Vec<MarkRecord>,
        mark2_array: Vec<Vec<Option<AnchorTable>>>,
    },
    Context(SequenceContextFormat),
    ChainedContext(ChainedSequenceContextFormat),
}

impl LookupSubtable for Subtable {
    fn load(data: &[u8], lookup_type: u16) -> Result<Subtable> {
        match lookup_type {
            1 => Subtable::load_single_adjustment(data),
            2 => Subtable::load_pair_adjustment(data),
            3 => Subtable::load_cursive_attachment(data),
            4 => Subtable::load_mark_to_base_attachment(data),
            5 => Subtable::load_mark_to_ligature_attachment(data),
            6 => Subtable::load_mark_to_mark_attachment(data),
            7 => Subtable::load_contextual_position(data),
            8 => Subtable::load_chained_contextual_positioning(data),
            9 => Subtable::load_extension_positioning(data),
            _ => Err(Error::Invalid),
        }
    }
}

impl Subtable {
    fn load_single_adjustment(data: &[u8]) -> Result<Subtable> {
        let coverage_offset = get_u16(data, 2)? as usize;
        let coverage = Coverage::load(&data[coverage_offset..])?;
        let value_format = ValueFormat::from_bits_truncate(data[4]);
        let format = match get_u16(data, 0)? {
            1 => SingleFormat::Format1(ValueRecord::load(data, 6, value_format)?),
            2 => {
                let value_count = get_u16(data, 6)? as usize;
                let size = value_format.record_size();
                if size == 0 {
                    SingleFormat::Format2(vec![ValueRecord::default(); value_count])
                } else {
                    let mut records = Vec::new();
                    for off in (8..8 + value_count * size).step_by(size) {
                        records.push(ValueRecord::load(data, off, value_format)?);
                    }
                    SingleFormat::Format2(records)
                }
            }
            _ => return Err(Error::Invalid),
        };
        Ok(Subtable::SingleAdjustment { coverage, format })
    }

    fn load_pair_adjustment(data: &[u8]) -> Result<Subtable> {
        let coverage_offset = get_u16(data, 2)? as usize;
        let coverage = Coverage::load(&data[coverage_offset..])?;
        let value_format1 = ValueFormat::from_bits_truncate(data[4]);
        let value_format2 = ValueFormat::from_bits_truncate(data[6]);
        let (size1, size2) = (value_format1.record_size(), value_format2.record_size());
        let format = match get_u16(data, 0)? {
            1 => {
                let pair_set_count = get_u16(data, 8)? as usize;
                let mut pair_sets = Vec::new();
                for off in (10..10 + pair_set_count * 2).step_by(2) {
                    let pair_set_off = get_u16(data, off)? as usize;
                    let data = &data[pair_set_off..];
                    let pair_val_count = get_u16(data, 0)? as usize;
                    let size = 2 + size1 + size2;
                    let mut records = HashMap::new();
                    for off in (2..2 + pair_val_count * size).step_by(size) {
                        let glyph = get_u16(data, off)?;
                        let rec1 = ValueRecord::load(data, off + 2, value_format1)?;
                        let rec2 = ValueRecord::load(data, off + 2 + size1, value_format2)?;
                        records.insert(glyph, (rec1, rec2));
                    }
                    pair_sets.push(records);
                }
                PairFormat::Format1(pair_sets)
            }
            2 => {
                let class1_off = get_u16(data, 8)? as usize;
                let class1 = ClassDef::load(&data[class1_off..])?;
                let class2_off = get_u16(data, 10)? as usize;
                let class2 = ClassDef::load(&data[class2_off..])?;
                let class1_count = get_u16(data, 12)? as usize;
                let class2_count = get_u16(data, 14)? as usize;
                let class2_size = size1 + size2;
                let class1_size = class2_size * class2_count;
                let records = if class1_size == 0 {
                    vec![
                        vec![(ValueRecord::default(), ValueRecord::default()); class2_count];
                        class1_count
                    ]
                } else {
                    let mut records = Vec::new();
                    for off1 in (16..16 + class1_count * class1_size).step_by(class1_size) {
                        let mut sub_records = Vec::new();
                        for off2 in (off1..off1 + class2_count * class2_size).step_by(class2_size) {
                            let rec1 = ValueRecord::load(data, off2, value_format1)?;
                            let rec2 = ValueRecord::load(data, off2 + size1, value_format2)?;
                            sub_records.push((rec1, rec2));
                        }
                        records.push(sub_records);
                    }
                    records
                };
                PairFormat::Format2 {
                    class1,
                    class2,
                    records,
                }
            }
            _ => return Err(Error::Invalid),
        };
        Ok(Subtable::PairAdjustment { coverage, format })
    }

    fn load_cursive_attachment(data: &[u8]) -> Result<Subtable> {
        if get_u16(data, 0)? != 1 {
            return Err(Error::Invalid);
        }
        let coverage_offset = get_u16(data, 2)? as usize;
        let coverage = Coverage::load(&data[coverage_offset..])?;
        let count = get_u16(data, 4)? as usize;
        let mut records = Vec::new();
        for off in (6..6 + count * 4).step_by(4) {
            let off1 = get_u16(data, off)? as usize;
            let anchor1 = if off1 == 0 {
                None
            } else {
                Some(AnchorTable::load(&data[off1..])?)
            };
            let off2 = get_u16(data, off + 2)? as usize;
            let anchor2 = if off2 == 0 {
                None
            } else {
                Some(AnchorTable::load(&data[off2..])?)
            };
            records.push((anchor1, anchor2));
        }
        Ok(Subtable::CursiveAttachment { coverage, records })
    }

    fn load_mark_to_base_attachment(data: &[u8]) -> Result<Subtable> {
        if get_u16(data, 0)? != 1 {
            return Err(Error::Invalid);
        }
        let mark_coverage_offset = get_u16(data, 2)? as usize;
        let mark_coverage = Coverage::load(&data[mark_coverage_offset..])?;
        let base_coverage_offset = get_u16(data, 4)? as usize;
        let base_coverage = Coverage::load(&data[base_coverage_offset..])?;
        let mark_class_count = get_u16(data, 6)? as usize;
        let mark_offset = get_u16(data, 8)? as usize;
        let mark_array = MarkRecord::load_array(&data[mark_offset..])?;
        let base_off = get_u16(data, 10)? as usize;
        let data = &data[base_off..];
        let record_count = get_u16(data, 0)? as usize;
        let record_size = mark_class_count * 2;
        let mut base_array = Vec::new();
        for record_off in (2..2 + record_count * record_size).step_by(record_size) {
            let mut anchors = Vec::new();
            for off in (record_off..record_off + record_size).step_by(2) {
                let anchor_off = get_u16(data, off)? as usize;
                if anchor_off == 0 {
                    anchors.push(None);
                } else {
                    anchors.push(Some(AnchorTable::load(&data[anchor_off..])?));
                }
            }
            base_array.push(anchors);
        }
        Ok(Subtable::MarkToBaseAttachment {
            mark_coverage,
            base_coverage,
            mark_array,
            base_array,
        })
    }

    fn load_mark_to_ligature_attachment(data: &[u8]) -> Result<Subtable> {
        if get_u16(data, 0)? != 1 {
            return Err(Error::Invalid);
        }
        let mark_coverage_offset = get_u16(data, 2)? as usize;
        let mark_coverage = Coverage::load(&data[mark_coverage_offset..])?;
        let ligature_coverage_offset = get_u16(data, 4)? as usize;
        let ligature_coverage = Coverage::load(&data[ligature_coverage_offset..])?;
        let mark_class_count = get_u16(data, 6)? as usize;
        let mark_offset = get_u16(data, 8)? as usize;
        let mark_array = MarkRecord::load_array(&data[mark_offset..])?;

        let ligature_arr_off = get_u16(data, 10)? as usize;
        let data = &data[ligature_arr_off..];
        let ligature_count = get_u16(data, 0)? as usize;
        let mut ligature_array = Vec::new();

        for off in (2..2 + ligature_count * 2).step_by(2) {
            let ligature_off = get_u16(data, off)? as usize;
            let data = &data[ligature_off..];

            let mut components = Vec::new();
            let component_count = get_u16(data, 0)? as usize;
            let component_size = mark_class_count * 2;
            for component_off in (2..2 + component_count * component_size).step_by(component_size) {
                let mut anchors = Vec::new();
                for off in (component_off..component_off + component_size).step_by(2) {
                    let anchor_off = get_u16(data, off)? as usize;
                    if anchor_off == 0 {
                        anchors.push(None);
                    } else {
                        anchors.push(Some(AnchorTable::load(&data[anchor_off..])?));
                    }
                }
                components.push(anchors);
            }
            ligature_array.push(components);
        }
        Ok(Subtable::MarkToLigatureAttachment {
            mark_coverage,
            ligature_coverage,
            mark_array,
            ligature_array,
        })
    }

    fn load_mark_to_mark_attachment(data: &[u8]) -> Result<Subtable> {
        if get_u16(data, 0)? != 1 {
            return Err(Error::Invalid);
        }
        let mark1_coverage_offset = get_u16(data, 2)? as usize;
        let mark1_coverage = Coverage::load(&data[mark1_coverage_offset..])?;
        let mark2_coverage_offset = get_u16(data, 4)? as usize;
        let mark2_coverage = Coverage::load(&data[mark2_coverage_offset..])?;
        let mark1_class_count = get_u16(data, 6)? as usize;
        let mark1_offset = get_u16(data, 8)? as usize;
        let mark1_array = MarkRecord::load_array(&data[mark1_offset..])?;
        let mark2_off = get_u16(data, 10)? as usize;
        let data = &data[mark2_off..];
        let record_count = get_u16(data, 0)? as usize;
        let record_size = mark1_class_count * 2;
        let mut mark2_array = Vec::new();
        for record_off in (2..2 + record_count * record_size).step_by(record_size) {
            let mut anchors = Vec::new();
            for off in (record_off..record_off + record_size).step_by(2) {
                let anchor_off = get_u16(data, off)? as usize;
                if anchor_off == 0 {
                    anchors.push(None);
                } else {
                    anchors.push(Some(AnchorTable::load(&data[anchor_off..])?));
                }
            }
            mark2_array.push(anchors);
        }
        Ok(Subtable::MarkToMarkAttachment {
            mark1_coverage,
            mark2_coverage,
            mark1_array,
            mark2_array,
        })
    }

    fn load_contextual_position(data: &[u8]) -> Result<Subtable> {
        SequenceContextFormat::load(data).map(|f| Subtable::Context(f))
    }

    fn load_chained_contextual_positioning(data: &[u8]) -> Result<Subtable> {
        ChainedSequenceContextFormat::load(data).map(|f| Subtable::ChainedContext(f))
    }

    fn load_extension_positioning(data: &[u8]) -> Result<Subtable> {
        if get_u16(data, 0)? != 1 {
            return Err(Error::Invalid);
        }
        let typ = get_u16(data, 2)?;
        let offset = get_u32(data, 4)? as usize;
        Subtable::load(&data[offset..], typ)
    }
}

bitflags! {
    struct ValueFormat : u8 {
        const X_PLACEMENT        = 0x01;
        const Y_PLACEMENT        = 0x02;
        const X_ADVANCE          = 0x04;
        const Y_ADVANCE          = 0x08;
        const X_PLACEMENT_DEVICE = 0x10;
        const Y_PLACEMENT_DEVICE = 0x20;
        const X_ADVANCE_DEVICE   = 0x40;
        const Y_ADVANCE_DEVICE   = 0x80;
    }
}

impl ValueFormat {
    fn record_size(&self) -> usize {
        let raw = self.bits();
        let mut ret = 0;
        for i in 0..8 {
            if raw & (1 << i) != 0 {
                ret += 2;
            }
        }
        ret
    }
}

#[derive(Clone, Debug, Default)]
struct ValueRecord {
    x_placement: i16,
    y_placement: i16,
    x_advance: i16,
    y_advance: i16,
    // TODO: Device
}

impl ValueRecord {
    fn load(data: &[u8], mut offset: usize, format: ValueFormat) -> Result<ValueRecord> {
        let mut get = |flag| -> Result<i16> {
            if format.contains(flag) {
                let ret = get_i16(data, offset)?;
                offset += 2;
                Ok(ret)
            } else {
                Ok(0)
            }
        };
        let x_placement = get(ValueFormat::X_PLACEMENT)?;
        let y_placement = get(ValueFormat::Y_PLACEMENT)?;
        let x_advance = get(ValueFormat::X_ADVANCE)?;
        let y_advance = get(ValueFormat::Y_ADVANCE)?;
        Ok(ValueRecord {
            x_placement,
            y_placement,
            x_advance,
            y_advance,
        })
    }
}
