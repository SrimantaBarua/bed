// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::min;

use crate::font::{harfbuzz::GlyphInfo, FaceKey};
use crate::style::{Color, TextSize, TextStyle};

pub(crate) struct ShapedTextSpan {
    pub(crate) face: FaceKey,
    pub(crate) style: TextStyle,
    glyphs: Vec<GlyphInfo>,
    colors: Vec<(usize, Color)>,
    underlines: Vec<(usize, Option<Color>)>,
}

impl ShapedTextSpan {
    pub(crate) fn styled_iter(&self) -> ShapedStyledTextIter {
        ShapedStyledTextIter {
            glyphs: &self.glyphs,
            colors: &self.colors,
            underlines: &self.underlines,
            idx: 0,
        }
    }
}

pub(crate) struct ShapedStyledTextIter<'a> {
    glyphs: &'a [GlyphInfo],
    colors: &'a [(usize, Color)],
    underlines: &'a [(usize, Option<Color>)],
    idx: usize,
}

impl<'a> Iterator for ShapedStyledTextIter<'a> {
    type Item = (&'a [GlyphInfo], Color, Option<Color>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.glyphs.len() {
            return None;
        }
        if self.colors[0].0 < self.underlines[0].0 {
            let glyphs = &self.glyphs[self.idx..self.colors[0].0];
            let color = self.colors[0].1;
            let underline = self.underlines[0].1;
            self.idx = self.colors[0].0;
            self.colors = &self.colors[1..];
            Some((glyphs, color, underline))
        } else {
            let glyphs = &self.glyphs[self.idx..self.underlines[0].0];
            let color = self.colors[0].1;
            let underline = self.underlines[0].1;
            self.idx = self.underlines[0].0;
            self.underlines = &self.underlines[1..];
            Some((glyphs, color, underline))
        }
    }
}

pub(crate) struct ShapedTextLine {
    pub(crate) spans: Vec<ShapedTextSpan>,
}
