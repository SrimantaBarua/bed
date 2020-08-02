// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::{Cell, RefCell};

use euclid::{point2, size2, vec2, Rect, Vector2D};
use ropey::{Rope, RopeSlice};
use unicode_segmentation::UnicodeSegmentation;

use crate::buffer::BufferBedHandle;
use crate::common::{
    rope_is_grapheme_boundary, rope_next_grapheme_boundary, rope_trim_newlines, split_text,
    PixelSize, RopeGraphemes, SplitCbRes,
};
use crate::painter::Painter;
use crate::style::{Color, TextStyle};
use crate::text::ShapedSpan;

const CUSROR_WIDTH: f32 = 2.0;

enum SpanOrSpace {
    Span(ShapedSpan),
    Space(usize),
}

pub(super) struct View {
    pub(super) cursor: ViewCursor,
    rect: Rect<f32, PixelSize>,
    off: Vector2D<f32, PixelSize>,
    start_line: usize,
    bed_handle: BufferBedHandle,
}

impl View {
    pub(super) fn new(bed_handle: BufferBedHandle, rect: Rect<f32, PixelSize>) -> View {
        View {
            rect,
            off: vec2(0.0, 0.0),
            start_line: 0,
            bed_handle,
            cursor: ViewCursor::default(),
        }
    }

    pub(super) fn set_rect(&mut self, rect: Rect<f32, PixelSize>, data: &Rope, tab_width: usize) {
        self.rect = rect;
        self.snap_to_cursor(data, tab_width);
    }

    pub(super) fn snap_to_cursor(&mut self, data: &Rope, tab_width: usize) {
        // Snap to Y
        self.snap_to_line(self.cursor.line_num, data, tab_width);

        // Snap to X
        let line = data.line(self.cursor.line_num);
        let mut text_font = self.bed_handle.text_font();
        let text_size = self.bed_handle.text_size();
        let text_style = TextStyle::default();
        let space_metrics = text_font.space_metrics(text_size, text_style);

        let cursor_x = Cell::new(0.0);
        let cursor_block_width = Cell::new(space_metrics.advance.width.to_f32());
        let line_gidx = self.cursor.line_gidx;
        let gidx = Cell::new(0);

        split_text(
            &line,
            tab_width,
            |n| {
                if (gidx.get()..gidx.get() + n).contains(&line_gidx) {
                    cursor_x.set(
                        cursor_x.get()
                            + space_metrics.advance.width.to_f32()
                                * (line_gidx - gidx.get()) as f32,
                    );
                    SplitCbRes::Stop
                } else {
                    cursor_x.set(cursor_x.get() + space_metrics.advance.width.to_f32() * n as f32);
                    gidx.set(gidx.get() + n);
                    SplitCbRes::Continue
                }
            },
            |text| {
                let shaped = text_font.shape(text, text_size, TextStyle::default());
                let mut gis = shaped.glyph_infos.iter().peekable();
                for (j, _) in text.grapheme_indices(true) {
                    loop {
                        if let Some(cluster) = gis.peek().map(|gi| gi.cluster) {
                            if cluster < j as u32 {
                                let gi = gis.next().unwrap();
                                cursor_x.set(cursor_x.get() + gi.advance.width.to_f32());
                                continue;
                            }
                        }
                        break;
                    }
                    if gidx.get() == line_gidx {
                        if let Some(gi) = gis.peek() {
                            cursor_block_width.set(gi.advance.width.to_f32());
                        }
                        return SplitCbRes::Stop;
                    }
                    gidx.set(gidx.get() + 1);
                }
                while let Some(gi) = gis.next() {
                    cursor_x.set(cursor_x.get() + gi.advance.width.to_f32());
                }
                SplitCbRes::Continue
            },
        );

        let cursor_width = if self.cursor.style == CursorStyle::Line {
            CUSROR_WIDTH
        } else {
            cursor_block_width.get()
        };
        let cursor_max_x = cursor_x.get() + cursor_width;
        let cursor_min_x = cursor_x.get();
        if self.off.x > cursor_min_x {
            self.off.x = cursor_min_x;
        } else if self.off.x + self.rect.size.width < cursor_max_x {
            self.off.x = cursor_max_x - self.rect.size.width;
        }
    }

    fn snap_to_line(&mut self, linum: usize, data: &Rope, tab_width: usize) {
        if linum <= self.start_line {
            self.start_line = linum;
            self.off.y = 0.0;
        } else {
            assert!(linum < data.len_lines());
            let mut iter = data.lines_at(linum + 1);
            let mut start_line = linum + 1;
            let mut height = 0.0;
            while let Some(line) = iter.prev() {
                start_line -= 1;
                let metrics = self.line_metrics(&line, tab_width);
                height += metrics.height;
                if height >= self.rect.size.height {
                    self.start_line = start_line;
                    self.off.y = height - self.rect.size.height;
                    return;
                }
                if start_line == self.start_line {
                    return;
                }
            }
            self.start_line = 0;
            self.off.y = 0.0;
        }
    }

    pub(super) fn scroll(
        &mut self,
        scroll: Vector2D<f32, PixelSize>,
        data: &Rope,
        tab_width: usize,
    ) {
        assert!(self.start_line < data.len_lines());

        self.off += scroll;

        // Scroll y
        while self.off.y < 0.0 && self.start_line > 0 {
            self.start_line -= 1;
            let metrics = self.line_metrics(&data.line(self.start_line), tab_width);
            self.off.y += metrics.height;
        }
        if self.off.y < 0.0 {
            self.off.y = 0.0;
        }
        while self.off.y > 0.0 {
            let metrics = self.line_metrics(&data.line(self.start_line), tab_width);
            if metrics.height > self.off.y {
                break;
            }
            if self.start_line == data.len_lines() - 1 {
                self.off.y = 0.0;
                break;
            }
            self.off.y -= metrics.height;
            self.start_line += 1;
        }

        // Scroll X
        if self.off.x <= 0.0 {
            self.off.x = 0.0;
        } else {
            let mut height = -self.off.y;
            let mut max_xoff = 0.0;
            for line in data.lines_at(self.start_line) {
                let metrics = self.line_metrics(&line, tab_width);
                if metrics.width > max_xoff {
                    max_xoff = metrics.width;
                }
                height += metrics.height;
                if height >= self.rect.size.height {
                    break;
                }
            }
            max_xoff -= self.rect.size.width;
            if max_xoff < 0.0 {
                max_xoff = 0.0;
            }
            if self.off.x > max_xoff {
                self.off.x = max_xoff;
            }
        }

        self.bed_handle.request_redraw();
    }

    pub(super) fn draw(&mut self, data: &Rope, painter: &mut Painter, tab_width: usize) {
        assert!(self.start_line < data.len_lines());
        let mut paint_ctx =
            painter.widget_ctx(self.rect, Color::new(0xff, 0xff, 0xff, 0xff), false);

        let mut text_font = self.bed_handle.text_font();
        let text_size = self.bed_handle.text_size();
        let text_style = TextStyle::default();
        let space_metrics = text_font.space_metrics(text_size, text_style);
        let mut origin = self.rect.origin - self.off;
        let spans = RefCell::new(Vec::new());
        let mut linum = self.start_line;
        let rect_max_x = self.rect.origin.x + self.rect.size.width;

        for rope_line in data.lines_at(self.start_line) {
            if origin.y >= self.rect.origin.y + self.rect.size.height {
                break;
            }
            let mut ascender = space_metrics.ascender;
            let mut descender = space_metrics.descender;
            let gidx = Cell::new(0);

            let cursor = &self.cursor;
            let cursor_x = Cell::new(None);
            let cursor_underline_height = Cell::new(1.0);
            let cursor_underline_pos = Cell::new(-1.0);
            let cursor_block_width = Cell::new(space_metrics.advance.width.to_f32());
            let current_x = Cell::new(origin.x);

            split_text(
                &rope_line,
                tab_width,
                |n| {
                    if linum == cursor.line_num
                        && (gidx.get()..gidx.get() + n).contains(&cursor.line_gidx)
                    {
                        cursor_x.set(Some(
                            current_x.get()
                                + space_metrics.advance.width.to_f32()
                                    * (cursor.line_gidx - gidx.get()) as f32,
                        ));
                    }
                    gidx.set(gidx.get() + n);
                    current_x
                        .set(current_x.get() + space_metrics.advance.width.to_f32() * n as f32);
                    let inner = &mut *spans.borrow_mut();
                    inner.push(SpanOrSpace::Space(n));
                    if current_x.get() >= rect_max_x {
                        SplitCbRes::Stop
                    } else {
                        SplitCbRes::Continue
                    }
                },
                |text| {
                    let shaped = text_font.shape(text, text_size, TextStyle::default());
                    let mut gis = shaped.glyph_infos.iter().peekable();
                    for (j, _) in text.grapheme_indices(true) {
                        loop {
                            if let Some(cluster) = gis.peek().map(|gi| gi.cluster) {
                                if cluster < j as u32 {
                                    let gi = gis.next().unwrap();
                                    current_x.set(current_x.get() + gi.advance.width.to_f32());
                                    continue;
                                }
                            }
                            break;
                        }
                        if linum == cursor.line_num && gidx.get() == cursor.line_gidx {
                            cursor_x.set(Some(current_x.get()));
                            cursor_underline_height.set(shaped.underline_thickness.to_f32());
                            cursor_underline_pos.set(shaped.underline_pos.to_f32());
                            if let Some(gi) = gis.peek() {
                                cursor_block_width.set(gi.advance.width.to_f32());
                            }
                        }
                        gidx.set(gidx.get() + 1);
                    }
                    while let Some(gi) = gis.next() {
                        current_x.set(current_x.get() + gi.advance.width.to_f32());
                    }
                    if shaped.ascender > ascender {
                        ascender = shaped.ascender;
                    }
                    if shaped.descender > descender {
                        descender = shaped.descender;
                    }
                    let inner = &mut *spans.borrow_mut();
                    inner.push(SpanOrSpace::Span(shaped));
                    if current_x.get() >= rect_max_x {
                        SplitCbRes::Stop
                    } else {
                        SplitCbRes::Continue
                    }
                },
            );
            let height = ascender - descender;

            let (cursor_width, cursor_height, cursor_y) = match cursor.style {
                CursorStyle::Line => (CUSROR_WIDTH, height.to_f32(), origin.y),
                CursorStyle::Block => (cursor_block_width.get(), height.to_f32(), origin.y),
                CursorStyle::Underline => (
                    cursor_block_width.get(),
                    cursor_underline_height.get(),
                    origin.y + ascender.to_f32() + cursor_underline_pos.get(),
                ),
            };

            if let Some(x) = cursor_x.get() {
                let rect: Rect<f32, PixelSize> =
                    Rect::new(point2(x, cursor_y), size2(cursor_width, cursor_height));
                paint_ctx.color_quad(rect, Color::new(0x88, 0x44, 0x22, 0x88), false);
            } else if linum == cursor.line_num {
                let rect: Rect<f32, PixelSize> = Rect::new(
                    point2(current_x.get(), cursor_y),
                    size2(cursor_width, cursor_height),
                );
                paint_ctx.color_quad(rect, Color::new(0x88, 0x44, 0x22, 0x88), false);
            }

            let spans = &mut *spans.borrow_mut();
            origin.y += ascender.to_f32();
            let mut pos = origin;
            for span_or_space in spans.iter() {
                if pos.x >= self.rect.origin.x + self.rect.size.width {
                    break;
                }
                match span_or_space {
                    SpanOrSpace::Space(n) => {
                        pos.x += (space_metrics.advance.width).to_f32() * (*n as f32);
                    }
                    SpanOrSpace::Span(shaped) => {
                        shaped.draw(pos, Color::new(0x00, 0x00, 0x00, 0xff));
                        pos.x += shaped.width.to_f32();
                    }
                }
            }
            spans.clear();
            origin.y -= descender.to_f32();
            linum += 1;
        }

        text_font.flush_glyphs();
    }

    pub(super) fn scroll_to_top(&mut self) {
        self.start_line = 0;
    }

    fn line_metrics(&self, line: &RopeSlice, tab_width: usize) -> LineMetrics {
        let mut text_font = self.bed_handle.text_font();
        let text_size = self.bed_handle.text_size();
        let text_style = TextStyle::default();
        let space_metrics = text_font.space_metrics(text_size, text_style);
        let state = RefCell::new((space_metrics.ascender, space_metrics.descender, 0.0));
        split_text(
            &line,
            tab_width,
            |n| {
                let inner = &mut *state.borrow_mut();
                inner.2 += space_metrics.advance.width.to_f32() * n as f32;
                SplitCbRes::Continue
            },
            |text| {
                let inner = &mut *state.borrow_mut();
                let shaped = text_font.shape(text, text_size, TextStyle::default());
                if shaped.ascender > inner.0 {
                    inner.0 = shaped.ascender;
                }
                if shaped.descender > inner.1 {
                    inner.1 = shaped.descender;
                }
                inner.2 += shaped.width.to_f32();
                SplitCbRes::Continue
            },
        );
        let state = &*state.borrow();
        LineMetrics {
            height: (state.0 - state.1).to_f32(),
            width: state.2,
        }
    }
}

struct LineMetrics {
    height: f32,
    width: f32,
}

#[derive(Eq, PartialEq)]
enum CursorStyle {
    Line,
    Block,
    Underline,
}

impl Default for CursorStyle {
    fn default() -> CursorStyle {
        CursorStyle::Block
    }
}

#[derive(Default)]
pub(super) struct ViewCursor {
    pub(super) cidx: usize,
    pub(super) line_num: usize,
    pub(super) line_cidx: usize,
    pub(super) line_gidx: usize,
    pub(super) line_global_x: usize,
    style: CursorStyle,
}

impl ViewCursor {
    pub(super) fn reset(&mut self) {
        *self = ViewCursor::default();
    }

    pub(super) fn sync_and_update_char_idx_left(&mut self, data: &Rope, tab_width: usize) {
        self.line_num = data.char_to_line(self.cidx);
        self.line_cidx = self.cidx - data.line_to_char(self.line_num);
        self.sync_line_cidx_gidx_left(data, tab_width);
    }

    pub(super) fn sync_global_x(&mut self, data: &Rope, tab_width: usize) {
        let trimmed = rope_trim_newlines(data.line(self.line_num));
        let (cidx, gidx) =
            cidx_gidx_from_global_x(&trimmed, self.line_global_x, tab_width, self.past_end());
        self.line_cidx = cidx;
        self.line_gidx = gidx;
        self.cidx = data.line_to_char(self.line_num) + self.line_cidx;
    }

    pub(super) fn sync_line_cidx_gidx_left(&mut self, data: &Rope, tab_width: usize) {
        let trimmed = rope_trim_newlines(data.line(self.line_num));
        let len_chars = trimmed.len_chars();
        if self.line_cidx >= len_chars {
            self.line_cidx = len_chars;
            if !self.past_end() && self.line_cidx > 0 {
                self.line_cidx -= 1;
            }
        }
        let (cidx, gidx) = cidx_gidx_from_cidx(&trimmed, self.line_cidx, tab_width);
        self.line_cidx = cidx;
        self.line_gidx = gidx;
        self.line_global_x = self.line_gidx;
        self.cidx = data.line_to_char(self.line_num) + self.line_cidx;
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
        if !self.past_end() && self.line_cidx == len_chars && self.line_cidx > 0 {
            self.line_cidx -= 1;
        }
        let (cidx, gidx) = cidx_gidx_from_cidx(&trimmed, self.line_cidx, tab_width);
        self.line_cidx = cidx;
        self.line_gidx = gidx;
        self.line_global_x = self.line_gidx;
        self.cidx = data.line_to_char(self.line_num) + self.line_cidx;
    }

    fn past_end(&self) -> bool {
        self.style == CursorStyle::Line
    }
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

pub(super) fn cidx_gidx_from_gidx(
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
