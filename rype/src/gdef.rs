// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::classdef::ClassDef;
use crate::coverage::Coverage;
use crate::error::*;
use crate::types::{get_i16, get_u16, get_u32};

#[derive(Debug)]
struct GlyphClassDef(ClassDef);

#[derive(Debug)]
struct MarkAttachmentClassDef(ClassDef);

#[derive(Debug)]
struct AttachmentPoints {
    coverage: Coverage,
    contour_points: Vec<Vec<u16>>,
}

impl AttachmentPoints {
    fn load(data: &[u8]) -> Result<AttachmentPoints> {
        let coverage_off = get_u16(data, 0)? as usize;
        let coverage = Coverage::load(&data[coverage_off..])?;
        let glyph_count = get_u16(data, 2)? as usize;
        let mut contour_points = Vec::new();
        for off in (4..4 + glyph_count * 2).step_by(2) {
            let attach_off = get_u16(data, off)? as usize;
            let data = &data[attach_off..];
            let point_count = get_u16(data, 0)? as usize;
            let mut point_indices = Vec::new();
            for off in (2..2 + point_count * 2).step_by(2) {
                point_indices.push(get_u16(data, off)?);
            }
            contour_points.push(point_indices);
        }
        Ok(AttachmentPoints {
            coverage,
            contour_points,
        })
    }
}

#[derive(Debug)]
enum CaretValue {
    Format1 { coord: i16 },
    Format2 { index: u16 },
    Format3 { coord: i16, /* TODO device table */ },
}

impl CaretValue {
    fn load(data: &[u8]) -> Result<CaretValue> {
        match get_u16(data, 0)? {
            1 => {
                let coord = get_i16(data, 2)?;
                Ok(CaretValue::Format1 { coord })
            }
            2 => {
                let index = get_u16(data, 2)?;
                Ok(CaretValue::Format2 { index })
            }
            3 => {
                let coord = get_i16(data, 2)?;
                // TODO: Device table
                Ok(CaretValue::Format3 { coord })
            }
            _ => Err(Error::Invalid),
        }
    }
}

#[derive(Debug)]
struct LigatureCarets {
    coverage: Coverage,
    caret_values: Vec<Vec<CaretValue>>,
}

impl LigatureCarets {
    fn load(data: &[u8]) -> Result<LigatureCarets> {
        let coverage_off = get_u16(data, 0)? as usize;
        let coverage = Coverage::load(&data[coverage_off..])?;
        let glyph_count = get_u16(data, 2)? as usize;
        let mut ligatures = Vec::new();
        for off in (4..4 + glyph_count * 2).step_by(2) {
            let tab_off = get_u16(data, off)? as usize;
            let data = &data[tab_off..];
            let caret_count = get_u16(data, 0)? as usize;
            let mut carets = Vec::new();
            for off in (2..2 + caret_count * 2).step_by(2) {
                let caret_off = get_u16(data, off)? as usize;
                carets.push(CaretValue::load(&data[caret_off..])?);
            }
            ligatures.push(carets);
        }
        Ok(LigatureCarets {
            coverage,
            caret_values: ligatures,
        })
    }
}

#[derive(Debug)]
struct MarkGlyphSets(Vec<Coverage>);

impl MarkGlyphSets {
    fn load(data: &[u8]) -> Result<MarkGlyphSets> {
        if get_u16(data, 0)? != 1 {
            return Err(Error::Invalid);
        }
        let count = get_u16(data, 2)? as usize;
        let mut coverages = Vec::new();
        for off in (4..4 + count * 4).step_by(4) {
            let cov_off = get_u32(data, off)? as usize;
            coverages.push(Coverage::load(&data[cov_off..])?);
        }
        Ok(MarkGlyphSets(coverages))
    }
}

#[derive(Debug)]
pub(crate) struct Gdef {
    glyph_class_def: Option<GlyphClassDef>,
    attachment_points: Option<AttachmentPoints>,
    ligature_carets: Option<LigatureCarets>,
    mark_attachment_class_def: Option<MarkAttachmentClassDef>,
    mark_glyph_sets: Option<MarkGlyphSets>,
    // TODO: Item variation store
}

impl Gdef {
    pub(crate) fn load(data: &[u8]) -> Result<Gdef> {
        let minor_version = get_u16(data, 2)?;
        let glyph_class_def = match get_u16(data, 4)? as usize {
            0 => None,
            off => Some(GlyphClassDef(ClassDef::load(&data[off..])?)),
        };
        let attachment_points = match get_u16(data, 6)? as usize {
            0 => None,
            off => Some(AttachmentPoints::load(&data[off..])?),
        };
        let ligature_carets = match get_u16(data, 8)? as usize {
            0 => None,
            off => Some(LigatureCarets::load(&data[off..])?),
        };
        let mark_attachment_class_def = match get_u16(data, 10)? as usize {
            0 => None,
            off => Some(MarkAttachmentClassDef(ClassDef::load(&data[off..])?)),
        };
        let mark_glyph_sets = if minor_version >= 2 {
            match get_u16(data, 12)? as usize {
                0 => None,
                off => Some(MarkGlyphSets::load(&data[off..])?),
            }
        } else {
            None
        };
        Ok(Gdef {
            glyph_class_def,
            attachment_points,
            ligature_carets,
            mark_attachment_class_def,
            mark_glyph_sets,
        })
    }
}
