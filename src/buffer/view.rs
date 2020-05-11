// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::{point2, size2, Rect};
use ropey::{Rope, RopeSlice};

use crate::common::{
    rope_is_grapheme_boundary, rope_next_grapheme_boundary, rope_trim_newlines, PixelSize,
    RopeGraphemes,
};
use crate::painter::WidgetPainter;
use crate::style::Color;
use crate::text::{ShapedText, TextShaper};

pub(super) struct Cursor {
    pub(super) char_idx: usize,
    pub(super) line_num: usize,
    pub(super) line_cidx: usize,
    pub(super) line_gidx: usize,
    pub(super) line_global_x: usize,
    pub(super) past_end: bool,
}

impl Cursor {
    pub(super) fn sync_and_update_char_idx_left(&mut self, data: &Rope, tab_width: usize) {
        self.line_num = data.char_to_line(self.char_idx);
        self.line_cidx = self.char_idx - data.line_to_char(self.line_num);
        self.sync_line_cidx_gidx_left(data, tab_width);
    }

    pub(super) fn sync_and_update_char_idx_right(&mut self, data: &Rope, tab_width: usize) {
        self.line_num = data.char_to_line(self.char_idx);
        self.line_cidx = self.char_idx - data.line_to_char(self.line_num);
        self.sync_line_cidx_gidx_right(data, tab_width);
    }

    pub(super) fn sync_line_cidx_gidx_left(&mut self, data: &Rope, tab_width: usize) {
        let trimmed = rope_trim_newlines(data.line(self.line_num));
        let len_chars = trimmed.len_chars();
        if self.line_cidx >= len_chars {
            self.line_cidx = len_chars;
            if !self.past_end && self.line_cidx > 0 {
                self.line_cidx -= 1;
            }
        }
        let (cidx, gidx) = cidx_gidx_from_cidx(&trimmed, self.line_cidx, tab_width);
        self.line_cidx = cidx;
        self.line_gidx = gidx;
        self.line_global_x = self.line_gidx;
        self.char_idx = data.line_to_char(self.line_num) + self.line_cidx;
    }

    pub(super) fn sync_line_cidx_gidx_right(&mut self, data: &Rope, tab_width: usize) {
        let trimmed = rope_trim_newlines(data.line(self.line_num));
        let len_chars = trimmed.len_chars();
        if self.line_cidx > len_chars {
            self.line_cidx = len_chars;
        }
        if !rope_is_grapheme_boundary(&trimmed, self.line_cidx) {
            self.line_cidx = rope_next_grapheme_boundary(&trimmed, self.line_cidx);
        }
        if !self.past_end && self.line_cidx == len_chars && self.line_cidx > 0 {
            self.line_cidx -= 1;
        }
        let (cidx, gidx) = cidx_gidx_from_cidx(&trimmed, self.line_cidx, tab_width);
        self.line_cidx = cidx;
        self.line_gidx = gidx;
        self.line_global_x = self.line_gidx;
        self.char_idx = data.line_to_char(self.line_num) + self.line_cidx;
    }

    pub(super) fn sync_global_x(&mut self, data: &Rope, tab_width: usize) {
        let trimmed = rope_trim_newlines(data.line(self.line_num));
        let (cidx, gidx) =
            cidx_gidx_from_global_x(&trimmed, self.line_global_x, tab_width, self.past_end);
        self.line_cidx = cidx;
        self.line_gidx = gidx;
        self.char_idx = data.line_to_char(self.line_num) + self.line_cidx;
    }

    pub(super) fn sync_gidx(&mut self, data: &Rope, tab_width: usize) {
        let trimmed = rope_trim_newlines(data.line(self.line_num));
        let (cidx, gidx) = cidx_gidx_from_gidx(&trimmed, self.line_gidx, tab_width, self.past_end);
        self.line_cidx = cidx;
        self.line_gidx = gidx;
        self.line_global_x = self.line_gidx;
        self.char_idx = data.line_to_char(self.line_num) + self.line_cidx;
    }

    fn default() -> Cursor {
        Cursor {
            char_idx: 0,
            line_num: 0,
            line_cidx: 0,
            line_gidx: 0,
            line_global_x: 0,
            past_end: true,
        }
    }
}

pub(super) struct BufferView {
    pub(super) rect: Rect<u32, PixelSize>,
    pub(super) cursor: Cursor,
    pub(super) start_line: usize,
    pub(super) yoff: u32,
}

impl BufferView {
    pub(super) fn new(rect: Rect<u32, PixelSize>) -> BufferView {
        BufferView {
            rect: rect,
            cursor: Cursor::default(),
            start_line: 0,
            yoff: 0,
        }
    }

    pub(super) fn snap_to_cursor(&mut self, shaped_lines: &[ShapedText]) {
        if self.start_line >= self.cursor.line_num {
            self.start_line = self.cursor.line_num;
            self.yoff = 0;
            return;
        }
        let mut i = self.cursor.line_num;
        let mut height = shaped_lines[i].height();
        while i > self.start_line {
            if height >= self.rect.size.height as i32 {
                self.start_line = i;
                self.yoff = height as u32 - self.rect.size.height;
                return;
            }
            i -= 1;
            height += shaped_lines[i].height();
        }
    }

    pub(super) fn draw(
        &self,
        shaped_lines: &[ShapedText],
        shaper: &mut TextShaper,
        painter: &mut WidgetPainter,
    ) {
        let mut pos = point2(0, -(self.yoff as i32));
        let mut linum = 0;
        let (cline, cgidx) = (
            self.cursor.line_num - self.start_line,
            self.cursor.line_gidx,
        );
        for line in &shaped_lines[self.start_line..] {
            pos.y += line.metrics.ascender;
            let mut gidx = 0;

            for (clusters, face, style, size, color, opt_under) in line.styled_iter() {
                for cluster in clusters {
                    if pos.x as u32 >= self.rect.size.width {
                        break;
                    }
                    let raster = shaper.get_raster(face, style).unwrap();
                    let start_x = pos.x;
                    for gi in cluster.glyph_infos {
                        painter.glyph(pos + gi.offset, face, gi.gid, size, color, style, raster);
                        pos.x += gi.advance.width;
                    }
                    let width = pos.x - start_x;
                    if linum == cline && gidx <= cgidx && gidx + cluster.num_graphemes > cgidx {
                        let cwidth = 2;
                        let cheight = line.metrics.ascender - line.metrics.descender;
                        let mut cx = (width * (cgidx - gidx) as i32) / cluster.num_graphemes as i32;
                        cx += start_x;
                        let cy = pos.y - line.metrics.ascender;
                        painter.color_quad(
                            Rect::new(point2(cx, cy), size2(cwidth, cheight)),
                            Color::new(0xff, 0x88, 0x22, 0xff),
                        );
                    }
                    if let Some(under) = opt_under {
                        painter.color_quad(
                            Rect::new(
                                point2(start_x, pos.y - line.metrics.underline_position),
                                size2(width, line.metrics.underline_thickness),
                            ),
                            under,
                        );
                    }
                    gidx += cluster.num_graphemes;
                }
            }
            if linum == cline && gidx == cgidx {
                let cwidth = 2;
                let cheight = line.metrics.ascender - line.metrics.descender;
                let cy = pos.y - line.metrics.ascender;
                painter.color_quad(
                    Rect::new(point2(pos.x, cy), size2(cwidth, cheight)),
                    Color::new(0xff, 0x88, 0x22, 0xff),
                );
            }
            pos.y -= line.metrics.descender;
            pos.x = 0;
            if pos.y as u32 >= self.rect.size.height {
                break;
            }
            linum += 1;
        }
    }
}

fn gidx_from_cidx(line: &RopeSlice, cidx: usize, tab_width: usize) -> usize {
    let (mut gidx, mut ccount) = (0, 0);
    for g in RopeGraphemes::new(line) {
        ccount += g.chars().count();
        if ccount > cidx {
            return gidx;
        }
        if g == "\t" {
            gidx = (gidx / tab_width) * tab_width + tab_width;
        } else {
            gidx += 1;
        }
    }
    gidx
}

fn cidx_gidx_from_cidx(slice: &RopeSlice, cidx: usize, tab_width: usize) -> (usize, usize) {
    let (mut gidx, mut ccount) = (0, 0);
    for g in RopeGraphemes::new(slice) {
        let count_here = g.chars().count();
        if ccount + count_here > cidx {
            return (ccount, gidx);
        }
        ccount += count_here;
        if g == "\t" {
            gidx = (gidx / tab_width) * tab_width + tab_width;
        } else {
            gidx += 1;
        }
    }
    (ccount, gidx)
}

fn cidx_gidx_from_gidx(
    slice: &RopeSlice,
    gidx: usize,
    tab_width: usize,
    past_end: bool,
) -> (usize, usize) {
    let (mut gcount, mut cidx) = (0, 0);
    let mut len_chars = slice.len_chars();
    if !past_end && len_chars > 0 {
        len_chars -= 1;
    }
    for g in RopeGraphemes::new(slice) {
        let count_here = g.chars().count();
        if gcount >= gidx || cidx + count_here > len_chars {
            return (cidx, gcount);
        }
        cidx += count_here;
        if g == "\t" {
            gcount = (gcount / tab_width) * tab_width + tab_width;
        } else {
            gcount += 1;
        }
    }
    (cidx, gcount)
}

fn cidx_gidx_from_global_x(
    slice: &RopeSlice,
    global_x: usize,
    tab_width: usize,
    past_end: bool,
) -> (usize, usize) {
    let (mut gidx, mut ccount) = (0, 0);
    let mut len_chars = slice.len_chars();
    if !past_end && len_chars > 0 {
        len_chars -= 1;
    }
    for g in RopeGraphemes::new(slice) {
        let count_here = g.chars().count();
        if gidx >= global_x || ccount + count_here > len_chars {
            return (ccount, gidx);
        }
        ccount += count_here;
        if g == "\t" {
            gidx = (gidx / tab_width) * tab_width + tab_width;
        } else {
            gidx += 1;
        }
    }
    (ccount, gidx)
}
