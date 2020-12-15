// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::collections::HashMap;

use crate::error::*;
use crate::types::{get_tag, get_u16, Tag};

/// Wrapper around ScriptList table common to GSUB and GPOS
#[derive(Debug)]
pub(crate) struct ScriptList(HashMap<Tag, ScriptTable>);

impl ScriptList {
    pub(crate) fn load(data: &[u8]) -> Result<ScriptList> {
        let mut table = HashMap::new();
        let record_count = get_u16(data, 0)? as usize;
        for record_off in (2..2 + record_count * 6).step_by(6) {
            let tag = get_tag(data, record_off)?;
            let table_off = get_u16(data, record_off + 4)? as usize;
            let script_table = ScriptTable::load(&data[table_off..])?;
            table.insert(tag, script_table);
        }
        Ok(ScriptList(table))
    }
}

#[derive(Debug)]
struct ScriptTable {
    default_lang_sys: Option<LangSysTable>,
    lang_sys_records: HashMap<Tag, LangSysTable>,
}

impl ScriptTable {
    fn load(data: &[u8]) -> Result<ScriptTable> {
        let mut lang_sys_records = HashMap::new();
        let default_lang_sys = match get_u16(data, 0)? as usize {
            0 => None,
            off => Some(LangSysTable::load(&data[off..])?),
        };
        let lang_sys_rec_count = get_u16(data, 2)? as usize;
        for lang_sys_rec_off in (4..4 + lang_sys_rec_count * 6).step_by(6) {
            let tag = get_tag(data, lang_sys_rec_off)?;
            let lang_sys_off = get_u16(data, lang_sys_rec_off + 4)? as usize;
            let lang_sys_table = LangSysTable::load(&data[lang_sys_off..])?;
            lang_sys_records.insert(tag, lang_sys_table);
        }
        Ok(ScriptTable {
            default_lang_sys,
            lang_sys_records,
        })
    }
}

#[derive(Debug)]
struct LangSysTable {
    required_feature_index: Option<u16>,
    feature_indices: Vec<u16>,
}

impl LangSysTable {
    fn load(data: &[u8]) -> Result<LangSysTable> {
        let mut feature_indices = Vec::new();
        let required_feature_index = match get_u16(data, 2)? {
            0xffff => None,
            i => Some(i),
        };
        let count = get_u16(data, 4).unwrap() as usize;
        for off in (6..6 + count * 2).step_by(2) {
            feature_indices.push(get_u16(data, off)?);
        }
        Ok(LangSysTable {
            required_feature_index,
            feature_indices,
        })
    }
}
