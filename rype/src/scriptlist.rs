// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use crate::error::*;
use crate::rcbuffer::RcBuf;
use crate::types::{get_tag, get_u16, Tag};

/// Wrapper around ScriptList table common to GSUB and GPOS
pub(crate) struct ScriptList(RcBuf);

impl ScriptList {
    pub(crate) fn load(data: RcBuf) -> Result<ScriptList> {
        Ok(ScriptList(data))
    }
}

impl fmt::Debug for ScriptList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.0;
        let count = get_u16(slice, 0).unwrap() as usize;
        f.debug_struct("ScriptList")
            .field("scriptCount", &count)
            .field(
                "scriptRecords",
                &(2..count * 6 + 2)
                    .step_by(6)
                    .map(|off| {
                        let tag = get_tag(slice, off).unwrap();
                        let offset = get_u16(slice, off + 4).unwrap() as usize;
                        let data = self.0.slice(offset..);
                        ScriptTable { tag, data }
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

struct ScriptTable {
    tag: Tag,
    data: RcBuf,
}

impl fmt::Debug for ScriptTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.data;
        let count = get_u16(slice, 2).unwrap() as usize;
        f.debug_struct("ScriptTable")
            .field("tag", &self.tag)
            .field("defaultLangSysOffset", &get_u16(slice, 0).unwrap())
            .field("langSysCount", &count)
            .field(
                "langSysRecords",
                &(4..count * 6 + 4)
                    .step_by(6)
                    .map(|off| {
                        let tag = get_tag(slice, off).unwrap();
                        let offset = get_u16(slice, off + 4).unwrap() as usize;
                        let data = self.data.slice(offset..);
                        LangSysTable { tag, data }
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

struct LangSysTable {
    tag: Tag,
    data: RcBuf,
}

impl fmt::Debug for LangSysTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.data;
        let count = get_u16(slice, 4).unwrap() as usize;
        f.debug_struct("LangSysTable")
            .field("tag", &self.tag)
            .field("lookupOrderOffset", &get_u16(slice, 0).unwrap())
            .field("requiredFeatureIndex", &get_u16(slice, 2).unwrap())
            .field("featureIndexCount", &count)
            .field(
                "featureIndices",
                &(6..6 + count * 2)
                    .step_by(2)
                    .map(|off| get_u16(slice, off).unwrap())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
