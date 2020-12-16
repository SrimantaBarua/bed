// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::collections::HashMap;

use crate::error::*;
use crate::types::{get_i16, get_u16};

#[derive(Debug)]
struct Table {
    coverage: Coverage,
    map: HashMap<(u16, u16), i16>,
}

#[derive(Debug)]
pub(crate) struct Kern(Vec<Table>);

impl Kern {
    pub(crate) fn load(data: &[u8]) -> Result<Kern> {
        let n_tables = get_u16(data, 2)?;
        let mut tables = Vec::new();
        let mut off = 4;
        for _ in 0..n_tables {
            if data.len() < off + 6 {
                return Err(Error::Invalid);
            }
            let length = get_u16(data, off + 2)? as usize;
            let coverage = Coverage::from_bits_truncate(data[5]);
            let format = data[4];
            if format != 0 {
                // FIXME: format 2 unsupported
                off += length;
                continue;
            }
            let npairs = get_u16(data, off + 6)? as usize;
            let mut map = HashMap::new();
            for rec_off in (off + 14..off + 14 + npairs * 6).step_by(6) {
                let left = get_u16(data, rec_off)?;
                let right = get_u16(data, rec_off + 2)?;
                let value = get_i16(data, rec_off + 4)?;
                map.insert((left, right), value);
            }
            tables.push(Table { coverage, map });
            off += length;
        }
        Ok(Kern(tables))
    }
}

bitflags! {
    struct Coverage : u8 {
        const HORIZONTAL   = 0x0001;
        const MINIMUM      = 0x0002;
        const CROSS_STREAM = 0x0004;
        const OVERRIDE     = 0x0008;
    }
}
