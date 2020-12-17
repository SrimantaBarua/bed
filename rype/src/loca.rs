// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::error::*;
use crate::head::IdxLocFmt;
use crate::types::{get_u16, get_u32};

#[derive(Debug)]
pub(crate) struct Loca(Vec<u32>);

impl Loca {
    pub(crate) fn load(data: &[u8], num_glyphs: usize, idx_loc_fmt: IdxLocFmt) -> Result<Loca> {
        let mut ret = Vec::new();
        let n = num_glyphs + 1;
        match idx_loc_fmt {
            IdxLocFmt::Off16 => {
                for off in (0..n * 2).step_by(2) {
                    ret.push(get_u16(data, off)? as u32);
                }
            }
            IdxLocFmt::Off32 => {
                for off in (0..n * 4).step_by(4) {
                    ret.push(get_u32(data, off)?);
                }
            }
        }
        Ok(Loca(ret))
    }

    pub(crate) fn offsets(&self) -> LocaOffIter {
        LocaOffIter {
            slice: &self.0,
            idx: 0,
        }
    }
}

pub(crate) struct LocaOffIter<'a> {
    slice: &'a [u32],
    idx: usize,
}

impl<'a> Iterator for LocaOffIter<'a> {
    type Item = Option<usize>;

    fn next(&mut self) -> Option<Option<usize>> {
        if self.idx + 1 >= self.slice.len() {
            None
        } else {
            let ret = self.slice[self.idx];
            self.idx += 1;
            if ret == self.slice[self.idx] {
                Some(None)
            } else {
                Some(Some(ret as usize))
            }
        }
    }
}
