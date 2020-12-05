// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use crate::{Error, Result};

/// Get big-endian u32
pub(crate) fn get_u32(b: &[u8], offset: usize) -> Result<u32> {
    if b.len() < offset + 4 {
        Err(Error::Invalid)
    } else {
        Ok(((b[offset] as u32) << 24)
            | ((b[offset + 1] as u32) << 16)
            | ((b[offset + 2] as u32) << 8)
            | (b[offset + 3] as u32))
    }
}

/// Load big-endian tag
pub(crate) fn get_tag(b: &[u8], offset: usize) -> Result<Tag> {
    get_u32(b, offset).map(|u| Tag(u))
}

/// OpenType "tag"s are used to uniquely identify resources like tables etc.
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct Tag(pub(crate) u32);

impl Tag {
    /// Create tag from string representation
    pub(crate) fn from_str(s: &str) -> Result<Tag> {
        get_u32(s.as_bytes(), 0).map(|u| Tag(u))
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let a = (self.0 >> 24) as u8 as char;
        let b = ((self.0 >> 16) & 0xff) as u8 as char;
        let c = ((self.0 >> 8) & 0xff) as u8 as char;
        let d = (self.0 & 0xff) as u8 as char;
        write!(f, "{}{}{}{}", a, b, c, d)
    }
}
