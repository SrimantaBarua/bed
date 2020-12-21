// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::common::GlyphID;
use crate::error::*;
use crate::types::{get_i16, get_u16};

/// Horizontal metrics for a glyph
#[derive(Debug)]
pub(crate) struct GlyphHorMetrics {
    pub(crate) advance_width: u16,
    pub(crate) lsb: i16,
}

#[derive(Debug)]
struct LongHorMetrics {
    advance_width: u16,
    lsb: i16,
}

/// Wrapper around horizontal metrics table
#[derive(Debug)]
pub(crate) struct Hmtx {
    long_hor_metrics: Vec<LongHorMetrics>,
    lsbs: Vec<i16>,
}

impl Hmtx {
    pub(crate) fn load(data: &[u8], num_glyphs: usize, num_h_metrics: usize) -> Result<Hmtx> {
        assert!(num_h_metrics > 0, "can num_h_metrics be zero?");
        let size = num_glyphs * 2 + num_h_metrics * 2;
        if data.len() < size {
            Err(Error::Invalid)
        } else {
            let mut long_hor_metrics = Vec::new();
            for i in 0..num_h_metrics {
                let advance_width = get_u16(data, i * 4)?;
                let lsb = get_i16(data, i * 4 + 2)?;
                long_hor_metrics.push(LongHorMetrics { advance_width, lsb });
            }
            let mut lsbs = Vec::new();
            for i in num_h_metrics..num_glyphs {
                lsbs.push(get_i16(data, i * 2 + num_h_metrics * 2)?);
            }
            Ok(Hmtx {
                long_hor_metrics,
                lsbs,
            })
        }
    }

    pub(crate) fn get_metrics(&self, glyph_id: GlyphID) -> GlyphHorMetrics {
        let mut glyph_id = glyph_id.0 as usize;
        if glyph_id < self.long_hor_metrics.len() {
            return GlyphHorMetrics {
                advance_width: self.long_hor_metrics[glyph_id].advance_width,
                lsb: self.long_hor_metrics[glyph_id].lsb,
            };
        }
        glyph_id -= self.long_hor_metrics.len();
        assert!(glyph_id < self.lsbs.len(), "glyph ID out of bounds");
        GlyphHorMetrics {
            advance_width: self.long_hor_metrics.last().unwrap().advance_width,
            lsb: self.lsbs[glyph_id],
        }
    }
}
