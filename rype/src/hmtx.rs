// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use crate::error::*;
use crate::rcbuffer::RcBuf;
use crate::types::{get_i16, get_u16};

/// Wrapper around horizontal metrics table
pub(crate) struct Hmtx {
    num_glyphs: usize,
    num_h_metrics: usize,
    data: RcBuf,
}

impl Hmtx {
    pub(crate) fn load(data: RcBuf, num_glyphs: usize, num_h_metrics: usize) -> Result<Hmtx> {
        if num_h_metrics == 0 {
            unimplemented!("can num_h_metrics be zero?");
        }
        let size = num_glyphs * 2 + num_h_metrics * 2;
        if data.len() < size {
            Err(Error::Invalid)
        } else {
            Ok(Hmtx {
                num_glyphs,
                num_h_metrics,
                data,
            })
        }
    }

    pub(crate) fn get_metrics(&self, glyph_id: u32) -> GlyphHorMetrics {
        let glyph_id = glyph_id as usize;
        let slice = &self.data;
        assert!(glyph_id < self.num_glyphs, "glyph ID out of bounds");
        if glyph_id < self.num_h_metrics {
            let offset = glyph_id * 4;
            GlyphHorMetrics {
                advance_width: get_u16(slice, offset).unwrap(),
                lsb: get_i16(slice, offset + 2).unwrap(),
            }
        } else {
            let aw_offset = (self.num_h_metrics - 1) * 2;
            let lsb_offset = glyph_id * 2 + self.num_h_metrics * 2;
            GlyphHorMetrics {
                advance_width: get_u16(slice, aw_offset).unwrap(),
                lsb: get_i16(slice, lsb_offset).unwrap(),
            }
        }
    }
}

impl fmt::Debug for Hmtx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.data;
        f.debug_struct("Hmtx")
            .field(
                "h_metrics",
                &(0..self.num_h_metrics)
                    .map(|i| {
                        let aw = get_u16(slice, i * 4).unwrap();
                        let lsb = get_i16(slice, i * 4 + 2).unwrap();
                        (aw, lsb)
                    })
                    .collect::<Vec<_>>(),
            )
            .field(
                "lsb",
                &(self.num_h_metrics..self.num_glyphs)
                    .map(|i| get_i16(slice, i * 2 + self.num_h_metrics * 2).unwrap())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

/// Horizontal metrics for a glyph
#[derive(Debug)]
pub(crate) struct GlyphHorMetrics {
    pub(crate) advance_width: u16,
    pub(crate) lsb: i16,
}
