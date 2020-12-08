// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::error::*;
use crate::rcbuffer::RcBuf;
use crate::scriptlist::ScriptList;
use crate::types::get_u16;

/// Wrapper around glyph substitution table
#[derive(Debug)]
pub(crate) struct Gsub {
    scriptlist: ScriptList,
}

impl Gsub {
    pub(crate) fn load(data: RcBuf) -> Result<Gsub> {
        let slice = &data;
        //let minor_version = get_u16(slice, 2)?;
        let scriptlist_off = get_u16(slice, 4)? as usize;
        let scriptlist = ScriptList::load(data.slice(scriptlist_off..))?;
        Ok(Gsub { scriptlist })
    }
}
