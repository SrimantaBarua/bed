// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use crate::error::*;
use crate::rcbuffer::RcBuf;
use crate::types::{get_tag, get_u16, Tag};

pub(crate) struct FeatureList(RcBuf);

impl FeatureList {
    pub(crate) fn load(data: RcBuf) -> Result<FeatureList> {
        Ok(FeatureList(data))
    }
}

impl fmt::Debug for FeatureList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.0;
        let count = get_u16(slice, 0).unwrap() as usize;
        f.debug_struct("FeatureList")
            .field("featureCount", &count)
            .field(
                "featureRecords",
                &(2..count * 6 + 2)
                    .step_by(6)
                    .map(|offset| {
                        let tag = get_tag(slice, offset).unwrap();
                        let offset = get_u16(slice, offset + 4).unwrap() as usize;
                        let data = self.0.slice(offset..);
                        FeatureTable { tag, data }
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

struct FeatureTable {
    tag: Tag,
    data: RcBuf,
}

impl fmt::Debug for FeatureTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.data;
        let count = get_u16(slice, 2).unwrap() as usize;
        f.debug_struct("FeatureTable")
            .field("tag", &self.tag)
            .field("featureParamsOffset", &get_u16(slice, 0).unwrap())
            .field("lookupIndexCount", &count)
            .field(
                "lookupListIndices",
                &(4..4 + count * 2)
                    .step_by(2)
                    .map(|off| get_u16(slice, off).unwrap())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
