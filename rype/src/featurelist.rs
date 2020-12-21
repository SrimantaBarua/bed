// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ops::Index;

use crate::error::*;
use crate::types::{get_tag, get_u16, Tag};

#[derive(Debug)]
pub(crate) struct FeatureList(Vec<(Tag, Vec<u16>)>);

impl FeatureList {
    pub(crate) fn load(data: &[u8]) -> Result<FeatureList> {
        let mut vec = Vec::new();
        let record_count = get_u16(data, 0)? as usize;
        for record_off in (2..record_count * 6).step_by(6) {
            let tag = get_tag(data, record_off)?;
            let table_off = get_u16(data, record_off + 4)? as usize;

            let data = &data[table_off..];
            let mut lookup_list_indices = Vec::new();
            let lookup_count = get_u16(data, 2)? as usize;
            for lookup_off in (4..4 + lookup_count * 2).step_by(2) {
                lookup_list_indices.push(get_u16(data, lookup_off)?);
            }
            vec.push((tag, lookup_list_indices));
        }
        Ok(FeatureList(vec))
    }
}

impl Index<usize> for FeatureList {
    type Output = (Tag, Vec<u16>);

    fn index(&self, idx: usize) -> &(Tag, Vec<u16>) {
        &self.0[idx]
    }
}
