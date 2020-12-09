// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use crate::error::*;
use crate::rcbuffer::RcBuf;
use crate::types::get_u16;

/// Wrapper around class definition table
pub(crate) enum ClassDef {
    Fmt1(RcBuf),
    Fmt2(RcBuf),
    Fmt3(RcBuf),
}

impl ClassDef {
    pub(crate) fn load(data: RcBuf) -> Result<ClassDef> {
        let slice = &data;
        let format = get_u16(slice, 0)?;
        match format {
            1 => Ok(ClassDef::Fmt1(data)),
            2 => Ok(ClassDef::Fmt2(data)),
            3 => Ok(ClassDef::Fmt3(data)),
            _ => Err(Error::Invalid),
        }
    }
}
