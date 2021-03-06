// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::{max, min};

use euclid::Size2D;
use unicode_segmentation::UnicodeSegmentation;

use crate::common::DPI;
use crate::font::{harfbuzz, FaceKey, FontCore, RasterFace};
use crate::style::{Color, TextSize, TextStyle};

use super::{RopeOrStr, ShapedText, ShapedTextMetrics, TextAlignment};

// TODO: Evaluate performance on caching shaped words

pub(crate) struct TextShaper {
    font_core: FontCore,
}

impl TextShaper {
    pub(crate) fn new(font_core: FontCore) -> TextShaper {
        TextShaper { font_core }
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
    pub(crate) fn shape_line(
        &mut self,
        line: RopeOrStr,
        dpi: Size2D<u32, DPI>,
        tab_width: usize,
        faces: &[(usize, FaceKey)],
        styles: &[(usize, TextStyle)],
        sizes: &[(usize, TextSize)],
        colors: &[(usize, Color)],
        unders: &[(usize, Option<Color>)],
        alignments: &[(usize, TextAlignment)],
    ) -> ShapedText {
        // We need this information even to shape an empty line (with a space)
        assert!(faces.len() > 0 && styles.len() > 0 && sizes.len() > 0);

        if line.len_chars() == 0 {
            return self.shape_empty_rope(dpi, faces[0].1, styles[0].1, sizes[0].1);
        }

        let mut ret = ShapedText::default();

        let mut last_cursor_position = 0;
        for g in line.graphemes() {
            if g == "\t".into() {
                let next_tab = (last_cursor_position / tab_width) * tab_width + tab_width;
                while last_cursor_position < next_tab {
                    ret.cursor_positions.push(last_cursor_position);
                    last_cursor_position += 1;
                }
            } else {
                ret.cursor_positions.push(last_cursor_position);
                last_cursor_position += g.len_chars();
            }
        }

        let input_iter = InputRangesIter {
            slice: line,
            faces,
            styles,
            sizes,
            colors,
            unders,
            alignments,
            cidx: 0,
        };

        let mut cidx = 0;
        let mut x = 0;

        for (slice, base_face, style, size, color, under, align) in
            input_iter.filter(|x| x.0.len_chars() > 0)
        {
            let mut chars = slice.chars().peekable();

            'outer: loop {
                let first_char = chars.next().unwrap();
                let face_key = if first_char == '\t' {
                    self.font_core
                        .find_for_char(base_face, ' ')
                        .unwrap_or(base_face)
                } else {
                    self.font_core
                        .find_for_char(base_face, first_char)
                        .unwrap_or(base_face)
                };

                let (buf, font) = self.font_core.get(face_key, style).unwrap();
                buf.clear_contents();
                if first_char == '\t' {
                    let next_tab = (x / tab_width) * tab_width + tab_width;
                    while x < next_tab {
                        buf.add(' ', cidx);
                        cidx += 1;
                        x += 1;
                    }
                } else {
                    buf.add(first_char, cidx);
                    cidx += 1;
                    x += 1;
                }

                let face_metrics = font.raster.get_metrics(size, dpi);
                ret.metrics.ascender = max(ret.metrics.ascender, face_metrics.ascender);
                ret.metrics.descender = min(ret.metrics.descender, face_metrics.descender);
                ret.metrics.underline_position =
                    min(ret.metrics.underline_position, face_metrics.underline_pos);
                ret.metrics.underline_thickness = max(
                    ret.metrics.underline_thickness,
                    face_metrics.underline_thickness,
                );

                while let Some(c) = chars.peek() {
                    if *c == '\t' {
                        if font.raster.has_glyph_for_char(' ') {
                            let next_tab = (x / tab_width) * tab_width + tab_width;
                            while x < next_tab {
                                buf.add(' ', cidx);
                                cidx += 1;
                                x += 1;
                            }
                            chars.next();
                            continue;
                        }
                    } else {
                        if font.raster.has_glyph_for_char(*c) {
                            buf.add(*c, cidx);
                            cidx += 1;
                            x += 1;
                            chars.next();
                            continue;
                        }
                    }
                    font.shaper.set_scale(size, dpi);
                    buf.guess_segment_properties();
                    let gis = harfbuzz::shape(&font.shaper, buf);
                    ret.push(gis, face_key, style, size, color, under, align);
                    continue 'outer;
                }
                font.shaper.set_scale(size, dpi);
                buf.guess_segment_properties();
                let gis = harfbuzz::shape(&font.shaper, buf);
                ret.push(gis, face_key, style, size, color, under, align);
                break;
            }
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
            metrics,
            glyphs,
            word_boundaries: vec![1],
            cursor_positions: vec![0],
            faces: vec![(1, face)],
            styles: vec![(1, style)],
            sizes: vec![(1, size)],
            colors: vec![(1, Color::new(0, 0, 0, 0xff))],
            unders: vec![(1, None)],
            alignments: vec![(1, TextAlignment::Left)],
        }
    }
}

struct InputRangesIter<'a> {
    slice: RopeOrStr<'a>,
    faces: &'a [(usize, FaceKey)],
    styles: &'a [(usize, TextStyle)],
    sizes: &'a [(usize, TextSize)],
    colors: &'a [(usize, Color)],
    unders: &'a [(usize, Option<Color>)],
    alignments: &'a [(usize, TextAlignment)],
    cidx: usize,
}

impl<'a> Iterator for InputRangesIter<'a> {
    type Item = (
        RopeOrStr<'a>,
        FaceKey,
        TextStyle,
        TextSize,
        Color,
        Option<Color>,
        TextAlignment,
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
        let align = self.alignments[0].1;
        let mut minidx = min(
            self.faces[0].0,
            min(
                self.styles[0].0,
                min(
                    self.sizes[0].0,
                    min(
                        self.colors[0].0,
                        min(self.unders[0].0, self.alignments[0].0),
                    ),
                ),
            ),
        );
        let mut ret_slice = self.slice.slice(self.cidx..minidx);
        // Break words. TODO: Optimize, this will suck for long lines
        let len_chars = ret_slice
            .to_string()
            .split_word_bounds()
            .next()
            .unwrap()
            .chars()
            .count();
        minidx = self.cidx + len_chars;
        ret_slice = ret_slice.slice(0..len_chars);
        // Update iterator
        self.cidx = minidx;
        if self.faces[0].0 == minidx {
            self.faces = &self.faces[1..];
            assert!(self.faces.len() == 0 || self.faces[0].0 > minidx);
        }
        if self.styles[0].0 == minidx {
            self.styles = &self.styles[1..];
            assert!(self.styles.len() == 0 || self.styles[0].0 > minidx);
        }
        if self.sizes[0].0 == minidx {
            self.sizes = &self.sizes[1..];
            assert!(self.sizes.len() == 0 || self.sizes[0].0 > minidx);
        }
        if self.colors[0].0 == minidx {
            self.colors = &self.colors[1..];
            assert!(self.colors.len() == 0 || self.colors[0].0 > minidx);
        }
        if self.unders[0].0 == minidx {
            self.unders = &self.unders[1..];
            assert!(self.unders.len() == 0 || self.unders[0].0 > minidx);
        }
        if self.alignments[0].0 == minidx {
            self.alignments = &self.alignments[1..];
            assert!(self.alignments.len() == 0 || self.alignments[0].0 > minidx);
        }
        Some((ret_slice, face, style, size, color, under, align))
    }
}
