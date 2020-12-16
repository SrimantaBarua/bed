// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::error::*;
use crate::head::IdxLocFmt;
use crate::types::{get_u16, get_u32};

#[derive(Debug)]
pub(crate) struct Loca(Vec<u32>);

impl Loca {
    pub(crate) fn load(data: &[u8], num_glyphs: usize, idx_loc_fmt: IdxLocFmt) -> Result<Loca> {
        let mut ret = Vec::new();
        match idx_loc_fmt {
            IdxLocFmt::Off16 => {
                for off in (0..num_glyphs * 2).step_by(2) {
                    ret.push(get_u16(data, off)? as u32);
                }
            }
            IdxLocFmt::Off32 => {
                for off in (0..num_glyphs * 4).step_by(4) {
                    ret.push(get_u32(data, off)?);
                }
            }
        }
        Ok(Loca(ret))
    }
}
