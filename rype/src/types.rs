// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use crate::{Error, Result};

/// Get big-endian u16
pub(crate) fn get_u16(b: &[u8], offset: usize) -> Result<u16> {
    if b.len() < offset + 2 {
        Err(Error::Invalid)
    } else {
        unsafe { Ok(get_u16_unchecked(b, offset)) }
    }
}

/// Get big-endian i16
pub(crate) fn get_i16(b: &[u8], offset: usize) -> Result<i16> {
    if b.len() < offset + 2 {
        Err(Error::Invalid)
    } else {
        unsafe { Ok(get_i16_unchecked(b, offset)) }
    }
}

/// Get big-endian u32
pub(crate) fn get_u32(b: &[u8], offset: usize) -> Result<u32> {
    if b.len() < offset + 4 {
        Err(Error::Invalid)
    } else {
        unsafe { Ok(get_u32_unchecked(b, offset)) }
    }
}

/// Get big-endian u16 without checking
pub(crate) unsafe fn get_u16_unchecked(b: &[u8], offset: usize) -> u16 {
    ((b[offset] as u16) << 8) | (b[offset + 1] as u16)
}

/// Get big-endian i16 without checking
pub(crate) unsafe fn get_i16_unchecked(b: &[u8], offset: usize) -> i16 {
    ((b[offset] as i16) << 8) | (b[offset + 1] as i16)
}

/// Get big-endian u32 without checking
pub(crate) unsafe fn get_u32_unchecked(b: &[u8], offset: usize) -> u32 {
    ((b[offset] as u32) << 24)
        | ((b[offset + 1] as u32) << 16)
        | ((b[offset + 2] as u32) << 8)
        | (b[offset + 3] as u32)
}
/// Load big-endian tag
pub(crate) fn get_tag(b: &[u8], offset: usize) -> Result<Tag> {
    get_u32(b, offset).map(|u| Tag(u))
}

/// OpenType "tag"s are used to uniquely identify resources like tables etc.
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct Tag(pub(crate) u32);

impl Tag {
    /// Create tag from 4 bytes
    pub(crate) const fn from(b: &[u8; 4]) -> Tag {
        Tag(((b[0] as u32) << 24) | ((b[1] as u32) << 16) | ((b[2] as u32) << 8) | (b[3] as u32))
    }
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let a = (self.0 >> 24) as u8 as char;
        let b = ((self.0 >> 16) & 0xff) as u8 as char;
        let c = ((self.0 >> 8) & 0xff) as u8 as char;
        let d = (self.0 & 0xff) as u8 as char;
        write!(f, "{}{}{}{}", a, b, c, d)
    }
}
