// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::min;

use crate::font::{
    harfbuzz::{GlyphInfo, GlyphInfoIter},
    FaceKey, ScaledFaceMetrics,
};
use crate::style::{Color, TextSize, TextStyle};

mod shaper;
pub(crate) use shaper::TextShaper;

pub(crate) struct ShapedText {
    pub(crate) metrics: ShapedTextMetrics,
    glyphs: Vec<GlyphInfo>,
    cursor_positions: Vec<usize>,
    faces: Vec<(usize, FaceKey)>,
    styles: Vec<(usize, TextStyle)>,
    sizes: Vec<(usize, TextSize)>,
    colors: Vec<(usize, Color)>,
    unders: Vec<(usize, Option<Color>)>,
}

impl ShapedText {
    pub(crate) fn styled_iter(&self) -> ShapedStyledTextIter {
        ShapedStyledTextIter {
            glyphs: &self.glyphs,
            cursor_positions: &self.cursor_positions,
            faces: &self.faces,
            styles: &self.styles,
            sizes: &self.sizes,
            colors: &self.colors,
            unders: &self.unders,
            idx: 0,
        }
    }

    pub(crate) fn height(&self) -> i32 {
        self.metrics.ascender - self.metrics.descender
    }

    fn default() -> ShapedText {
        ShapedText {
            metrics: ShapedTextMetrics::default(),
            glyphs: Vec::new(),
            cursor_positions: Vec::new(),
            faces: Vec::new(),
            styles: Vec::new(),
            sizes: Vec::new(),
            colors: Vec::new(),
            unders: Vec::new(),
        }
    }

    fn push(
        &mut self,
        gis: GlyphInfoIter,
        face: FaceKey,
        style: TextStyle,
        size: TextSize,
        color: Color,
        under: Option<Color>,
    ) {
        for gi in gis {
            self.glyphs.push(gi);
        }
        let glyph_len = self.glyphs.len();
        let face_len = self.faces.len();
        let style_len = self.styles.len();
        let size_len = self.sizes.len();
        let color_len = self.colors.len();
        let under_len = self.unders.len();
        if face_len > 0 && self.faces[face_len - 1].1 == face {
            self.faces[face_len - 1].0 = glyph_len;
        } else {
            self.faces.push((glyph_len, face));
        }

        if style_len > 0 && self.styles[style_len - 1].1 == style {
            self.styles[style_len - 1].0 = glyph_len;
        } else {
            self.styles.push((glyph_len, style));
        }

        if size_len > 0 && self.sizes[size_len - 1].1 == size {
            self.sizes[size_len - 1].0 = glyph_len;
        } else {
            self.sizes.push((glyph_len, size));
        }

        if color_len > 0 && self.colors[color_len - 1].1 == color {
            self.colors[color_len - 1].0 = glyph_len;
        } else {
            self.colors.push((glyph_len, color));
        }

        if under_len > 0 && self.unders[under_len - 1].1 == under {
            self.unders[under_len - 1].0 = glyph_len;
        } else {
            self.unders.push((glyph_len, under));
        }
    }
}

pub(crate) struct ShapedStyledTextIter<'a> {
    glyphs: &'a [GlyphInfo],
    cursor_positions: &'a [usize],
    faces: &'a [(usize, FaceKey)],
    styles: &'a [(usize, TextStyle)],
    sizes: &'a [(usize, TextSize)],
    colors: &'a [(usize, Color)],
    unders: &'a [(usize, Option<Color>)],
    idx: usize,
}

impl<'a> Iterator for ShapedStyledTextIter<'a> {
    type Item = (
        ShapedClusterIter<'a>,
        FaceKey,
        TextStyle,
        TextSize,
        Color,
        Option<Color>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.glyphs.len() {
            return None;
        }
        let face = self.faces[0].1;
        let style = self.styles[0].1;
        let size = self.sizes[0].1;
        let color = self.colors[0].1;
        let under = self.unders[0].1;
        let minidx = min(
            self.faces[0].0,
            min(
                self.styles[0].0,
                min(self.sizes[0].0, min(self.colors[0].0, self.unders[0].0)),
            ),
        );
        let glyphs = &self.glyphs[self.idx..minidx];
        self.idx = minidx;
        if self.faces[0].0 == minidx {
            self.faces = &self.faces[1..];
        }
        if self.styles[0].0 == minidx {
            self.styles = &self.styles[1..];
        }
        if self.sizes[0].0 == minidx {
            self.sizes = &self.sizes[1..];
        }
        if self.colors[0].0 == minidx {
            self.colors = &self.colors[1..];
        }
        if self.unders[0].0 == minidx {
            self.unders = &self.unders[1..];
        }
        let cii = if minidx == self.glyphs.len() {
            self.cursor_positions.len()
        } else {
            let mut cii = 0;
            while cii < self.cursor_positions.len()
                && self.cursor_positions[cii] < glyphs[minidx].cluster as usize
            {
                cii += 1;
            }
            cii
        };
        let cluster_iter = ShapedClusterIter {
            cursor_positions: &self.cursor_positions[..cii],
            cpi: 0,
            glyph_infos: glyphs,
            gii: 0,
        };
        self.cursor_positions = &self.cursor_positions[cii..];
        Some((cluster_iter, face, style, size, color, under))
    }
}

pub(crate) struct ShapedClusterIter<'a> {
    cursor_positions: &'a [usize],
    cpi: usize,
    glyph_infos: &'a [GlyphInfo],
    gii: usize,
}

impl<'a> Iterator for ShapedClusterIter<'a> {
    type Item = ShapedCluster<'a>;

    fn next(&mut self) -> Option<ShapedCluster<'a>> {
        if self.cpi == self.cursor_positions.len() || self.gii == self.glyph_infos.len() {
            return None;
        }
        let mut i = self.gii + 1;
        while i < self.glyph_infos.len()
            && self.glyph_infos[i].cluster == self.glyph_infos[self.gii].cluster
        {
            i += 1;
        }
        if i == self.glyph_infos.len() {
            let ret = Some(ShapedCluster {
                num_graphemes: self.cursor_positions.len() - self.cpi,
                glyph_infos: &self.glyph_infos[self.gii..],
            });
            self.cpi = self.cursor_positions.len();
            self.gii = self.glyph_infos.len();
            ret
        } else {
            let mut count = 0;
            while self.cpi < self.cursor_positions.len()
                && self.cursor_positions[self.cpi] != self.glyph_infos[i].cluster as usize
            {
                self.cpi += 1;
                count += 1;
            }
            let ret = Some(ShapedCluster {
                num_graphemes: count,
                glyph_infos: &self.glyph_infos[self.gii..i],
            });
            self.gii = i;
            ret
        }
    }
}

pub(crate) struct ShapedCluster<'a> {
    pub(crate) num_graphemes: usize,
    pub(crate) glyph_infos: &'a [GlyphInfo],
}

pub(crate) struct ShapedTextMetrics {
    pub(crate) ascender: i32,
    pub(crate) descender: i32,
    pub(crate) underline_position: i32,
    pub(crate) underline_thickness: i32,
}

impl ShapedTextMetrics {
    fn default() -> ShapedTextMetrics {
        ShapedTextMetrics {
            ascender: 0,
            descender: 0,
            underline_position: 0,
            underline_thickness: 0,
        }
    }

    fn from_font_metrics(metrics: &ScaledFaceMetrics) -> ShapedTextMetrics {
        ShapedTextMetrics {
            ascender: metrics.ascender,
            descender: metrics.descender,
            underline_position: metrics.underline_pos,
            underline_thickness: metrics.underline_thickness,
        }
    }
}
