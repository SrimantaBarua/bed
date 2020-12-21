// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use geom::{Size2D, Vector2D};

/// Wrapper around glyphs
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct GlyphID(pub(crate) u32);

impl fmt::Debug for GlyphID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Shaped glyph information
#[derive(Debug)]
pub struct GlyphInfo {
    pub glyph: GlyphID,
    pub bearing: Vector2D<i16>,
    pub size: Size2D<u16>,
    pub offset: Vector2D<i16>,
    pub advance: Vector2D<u16>,
}
