// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::min;

use euclid::Size2D;
use ropey::RopeSlice;

use crate::common::DPI;
use crate::font::{harfbuzz, FaceKey, FontCore, RasterFace};
use crate::style::{Color, TextSize, TextStyle};

use super::{ShapedText, ShapedTextMetrics};

// TODO: Evaluate performance on caching shaped words

pub(crate) struct TextShaper {
    font_core: FontCore,
}

impl TextShaper {
    pub(crate) fn new(font_core: FontCore) -> TextShaper {
        TextShaper {
            font_core: font_core,
        }
    }

    pub(crate) fn get_raster(
        &mut self,
        face_key: FaceKey,
        style: TextStyle,
    ) -> Option<&mut RasterFace> {
        self.font_core
            .get(face_key, style)
            .map(|(_, f)| &mut f.raster)
    }

    // All indices here are codepoint/char indices
    // TODO: This is one place that needs MAJOR improvement
    pub(crate) fn shape_line_rope(
        &mut self,
        line: RopeSlice,
        dpi: Size2D<u32, DPI>,
        tab_width: usize,
        faces: &[(usize, FaceKey)],
        styles: &[(usize, TextStyle)],
        sizes: &[(usize, TextSize)],
        colors: &[(usize, Color)],
        unders: &[(usize, Option<Color>)],
    ) -> ShapedText {
        // We need this information even to shape an empty line (with a space)
        assert!(faces.len() > 0 && styles.len() > 0 && sizes.len() > 0);

        if line.len_chars() == 0 {
            return self.shape_empty_rope(dpi, faces[0].1, styles[0].1, sizes[0].1);
        }

        let mut ret = ShapedText::default();
        let mut input_iter = InputRangesIter {
            slice: line,
            faces: faces,
            styles: styles,
            sizes: sizes,
            colors: colors,
            unders: unders,
            cidx: 0,
        };

        let mut cidx = 0;

        'outer: for (slice, base_face, style, size, color, under) in
            input_iter.filter(|x| x.0.len_chars() > 0)
        {
            let mut chars = slice.chars().peekable();
            let first_char = chars.next().unwrap();
            let face_key = self
                .font_core
                .find_for_char(base_face, first_char)
                .unwrap_or(base_face);

            let (buf, font) = self.font_core.get(face_key, style).unwrap();
            buf.clear_contents();
            buf.add(first_char, cidx);
            cidx += 1;

            let face_metrics = font.raster.get_metrics(size, dpi);

            while let Some(c) = chars.peek() {
                if font.raster.has_glyph_for_char(*c) {
                    buf.add(*c, cidx);
                    cidx += 1;
                    chars.next();
                    continue;
                }
                font.shaper.set_scale(size, dpi);
                buf.guess_segment_properties();
                let gis = harfbuzz::shape(&font.shaper, buf);
                ret.push(gis, face_key, style, size, color, under);
                continue 'outer;
            }
            font.shaper.set_scale(size, dpi);
            buf.guess_segment_properties();
            let gis = harfbuzz::shape(&font.shaper, buf);
            ret.push(gis, face_key, style, size, color, under);
        }

        ret
    }

    // For an empty line, just shape a ' ' character
    fn shape_empty_rope(
        &mut self,
        dpi: Size2D<u32, DPI>,
        face: FaceKey,
        style: TextStyle,
        size: TextSize,
    ) -> ShapedText {
        let face = self.font_core.find_for_char(face, ' ').unwrap();
        let (buf, font) = self.font_core.get(face, style).unwrap();
        let metrics = ShapedTextMetrics::from_font_metrics(&font.raster.get_metrics(size, dpi));
        buf.clear_contents();
        buf.add(' ', 0);
        buf.guess_segment_properties();
        font.shaper.set_scale(size, dpi);
        let glyphs = harfbuzz::shape(&font.shaper, buf).collect();
        ShapedText {
            metrics: metrics,
            glyphs: glyphs,
            faces: vec![(1, face)],
            styles: vec![(1, style)],
            sizes: vec![(1, size)],
            colors: vec![(1, Color::new(0, 0, 0, 0xff))],
            unders: vec![(1, None)],
        }
    }
}

struct InputRangesIter<'a> {
    slice: RopeSlice<'a>,
    faces: &'a [(usize, FaceKey)],
    styles: &'a [(usize, TextStyle)],
    sizes: &'a [(usize, TextSize)],
    colors: &'a [(usize, Color)],
    unders: &'a [(usize, Option<Color>)],
    cidx: usize,
}

impl<'a> Iterator for InputRangesIter<'a> {
    type Item = (
        RopeSlice<'a>,
        FaceKey,
        TextStyle,
        TextSize,
        Color,
        Option<Color>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if self.cidx >= self.slice.len_chars() {
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
        let slice = self.slice.slice(self.cidx..minidx);
        self.cidx = minidx;
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
        Some((slice, face, style, size, color, under))
    }
}
