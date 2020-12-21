// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use geom::{bbox, point2, BBox};

use crate::common::GlyphID;
use crate::error::*;
use crate::loca::Loca;
use crate::types::{get_i16, get_u16};

#[derive(Debug)]
pub(crate) struct Glyf(Vec<Option<Glyph>>);

impl Glyf {
    pub(crate) fn load(data: &[u8], loca: &Loca) -> Result<Glyf> {
        let mut glyphs = Vec::new();
        for off in loca.offsets() {
            if let Some(off) = off {
                glyphs.push(Some(Glyph::load(&data[off..])?));
            } else {
                glyphs.push(None);
            }
        }
        Ok(Glyf(glyphs))
    }

    pub(crate) fn glyph_bbox(&self, glyph: GlyphID) -> BBox<i16> {
        self.0[glyph.0 as usize]
            .as_ref()
            .map(|g| g.bbox)
            .unwrap_or_else(|| bbox(point2(0, 0), point2(0, 0)))
    }
}

bitflags! {
    struct SimpleFlags : u8 {
        const ON_CURVE_POINT           = 0x01;
        const X_SHORT_VECTOR           = 0x02;
        const Y_SHORT_VECTOR           = 0x04;
        const REPEAT_FLAG              = 0x08;
        const X_SAME_OR_POSITIVE_SHORT = 0x10;
        const Y_SAME_OR_POSITIVE_SHORT = 0x20;
    }
}

bitflags! {
    struct CompositeFlags : u16 {
        const ARGS_ARE_WORDS         = 0x0001;
        const ARGS_ARE_XY_VALUES     = 0x0002;
        const ROUND_XY_TO_GRID       = 0x0004;
        const HAVE_SCALE             = 0x0008;
        const MORE_COMPONENTS        = 0x0020;
        const HAVE_XY_SCALE          = 0x0040;
        const HAVE_TWO_BY_TWO        = 0x0080;
        const HAVE_INSTR             = 0x0100;
        const USE_MY_METRICS         = 0x0200;
        const OVERLAP_COMPOUND       = 0x0400;
        const SCALED_COMPONENT_OFF   = 0x0800;
        const UNSCALED_COMPONENT_OFF = 0x1000;
    }
}

#[derive(Debug)]
enum GlyphTyp {
    Simple {
        num_points: u16,
        data: Vec<u8>,
        flags_len: u16,
        x_len: u16,
        y_len: u16,
    },
    Composite(Vec<u8>),
}

impl GlyphTyp {
    fn load_simple(data: &[u8], num_contours: usize) -> Result<GlyphTyp> {
        if num_contours == 0 {
            return Ok(GlyphTyp::Simple {
                num_points: 0,
                flags_len: 0,
                x_len: 0,
                y_len: 0,
                data: vec![],
            });
        }
        let mut contour_ends = Vec::new();
        for off in (0..num_contours * 2).step_by(2) {
            contour_ends.push(get_u16(data, off)?);
        }
        let num_points = *contour_ends.last().unwrap() + 1;
        let insn_len = get_u16(data, num_contours * 2)? as usize;
        let flags_off = num_contours * 2 + 2 + insn_len;
        let (mut flags_len, mut x_len, mut y_len, mut repeat_count) = (0, 0, 0, 0);
        let mut flags = Vec::new();
        let mut cur_flag = SimpleFlags::empty();
        for _ in 0..num_points {
            if repeat_count > 0 {
                repeat_count -= 1;
            } else {
                cur_flag = SimpleFlags::from_bits_truncate(data[flags_off + flags_len]);
                flags_len += 1;
                flags.push(cur_flag);
                if cur_flag.contains(SimpleFlags::REPEAT_FLAG) {
                    repeat_count = data[flags_off + flags_len];
                    flags_len += 1;
                }
            }
            if cur_flag.contains(SimpleFlags::X_SHORT_VECTOR) {
                x_len += 1;
            } else if !cur_flag.contains(SimpleFlags::X_SAME_OR_POSITIVE_SHORT) {
                x_len += 2;
            }
            if cur_flag.contains(SimpleFlags::Y_SHORT_VECTOR) {
                y_len += 1;
            } else if !cur_flag.contains(SimpleFlags::Y_SAME_OR_POSITIVE_SHORT) {
                y_len += 2;
            }
        }
        let data_len = flags_len + x_len + y_len;
        Ok(GlyphTyp::Simple {
            num_points,
            flags_len: flags_len as u16,
            x_len: x_len as u16,
            y_len: y_len as u16,
            data: data[flags_off..flags_off + data_len].to_vec(),
        })
    }

    fn load_composite(data: &[u8]) -> Result<GlyphTyp> {
        let mut off = 0;
        loop {
            let flags = CompositeFlags::from_bits_truncate(get_u16(data, off)?);
            off += 4;
            if flags.contains(CompositeFlags::ARGS_ARE_WORDS) {
                off += 4;
            } else {
                off += 2;
            };
            if flags.contains(CompositeFlags::HAVE_SCALE) {
                off += 1;
            } else if flags.contains(CompositeFlags::HAVE_XY_SCALE) {
                off += 2;
            } else if flags.contains(CompositeFlags::HAVE_TWO_BY_TWO) {
                off += 4;
            }
            if !flags.contains(CompositeFlags::MORE_COMPONENTS) {
                break;
            }
        }
        Ok(GlyphTyp::Composite(data[..off].to_vec()))
    }
}

#[derive(Debug)]
struct Glyph {
    bbox: BBox<i16>,
    typ: GlyphTyp,
}

impl Glyph {
    fn load(data: &[u8]) -> Result<Glyph> {
        let num_contours = get_i16(data, 0)?;
        let xmin = get_i16(data, 2)?;
        let ymin = get_i16(data, 4)?;
        let xmax = get_i16(data, 6)?;
        let ymax = get_i16(data, 8)?;
        let typ = if num_contours < 0 {
            GlyphTyp::load_composite(&data[10..])?
        } else {
            GlyphTyp::load_simple(&data[10..], num_contours as usize)?
        };
        Ok(Glyph {
            typ,
            bbox: bbox(point2(xmin, ymin), point2(xmax, ymax)),
        })
    }
}
