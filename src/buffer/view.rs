// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::{Cell, RefCell};
use std::fmt::Write;

use euclid::{point2, vec2, Point2D, Rect, Vector2D};
use ropey::{Rope, RopeSlice};

use crate::buffer::BufferBedHandle;
use crate::common::{
    rope_is_grapheme_boundary, rope_next_grapheme_boundary, rope_trim_newlines, split_text,
    PixelSize, RopeGraphemes, RopeOrStr, SplitCbRes,
};
use crate::painter::Painter;
use crate::style::{StyleRanges, TextSize, TextStyle};
use crate::text::{f26_6, CursorStyle, StyleType, TextAlign, TextCursor, CURSOR_WIDTH};

pub(super) struct View {
    pub(super) cursor: ViewCursor,
    text_size: TextSize,
    rect: Rect<u32, PixelSize>,
    off: Vector2D<i32, PixelSize>,
    start_line: usize,
    bed_handle: BufferBedHandle,
    show_gutter: bool,
}

impl View {
    pub(super) fn new(bed_handle: BufferBedHandle, rect: Rect<u32, PixelSize>) -> View {
        View {
            rect,
            text_size: bed_handle.text_font_size(),
            off: vec2(0, 0),
            start_line: 0,
            bed_handle,
            cursor: ViewCursor::default(),
            show_gutter: true,
        }
    }

    pub(super) fn update_text_size(&mut self, diff: i16) {
        self.text_size += diff;
    }

    pub(super) fn set_rect(
        &mut self,
        rect: Rect<u32, PixelSize>,
        data: &Rope,
        tab_width: usize,
        styles: &StyleRanges,
    ) {
        self.rect = rect;
        self.snap_to_cursor(data, tab_width, styles, true);
    }

    pub(super) fn move_cursor_to_point(
        &mut self,
        mut point: Point2D<i32, PixelSize>,
        data: &Rope,
        tab_width: usize,
        styles: &StyleRanges,
    ) {
        let rect = self.rect.cast();
        assert!(rect.contains(point));
        point -= rect.origin.to_vector();
        point.x -= self.gutter_width(data) as i32;
        if point.x < 0 {
            // TODO: Handle click in gutter
            return;
        }
        self.sanity_check(data);

        // Find line
        self.cursor.line_num = self.start_line;
        let mut height = -self.off.y;
        for linum in self.start_line..data.len_lines() {
            let metrics = self.line_metrics(linum, data, tab_width, styles);
            height += metrics.height as i32;
            if height >= point.y {
                if height > rect.height() {
                    self.off.y += height - rect.height();
                }
                break;
            }
            self.cursor.line_num += 1;
            assert!(height < rect.height());
        }
        if self.cursor.line_num >= data.len_lines() {
            self.cursor.line_num = data.len_lines() - 1;
        }
        // Trim lines from the top if we need to
        for linum in self.start_line..data.len_lines() {
            let metrics = self.line_metrics(linum, data, tab_width, styles);
            if self.off.y < metrics.height as i32 {
                break;
            }
            self.off.y -= metrics.height as i32;
            self.start_line += 1;
        }

        // Check cursor position on the line
        let mut text_font = self.bed_handle.text_font();
        let cursor_x = Cell::new(-self.off.x as f32);
        let gidx = Cell::new(0);
        let point = point.cast::<f32>();

        let line = data.line(self.cursor.line_num);
        let start_cidx = data.line_to_char(self.cursor.line_num);
        let end_cidx = start_cidx + line.len_chars();

        for (range, &style) in styles.style.iter_range(start_cidx..end_cidx).unwrap() {
            let range = range.start - start_cidx..range.end - start_cidx;
            let space_metrics = text_font.space_metrics(self.text_size, style);
            let sp_awidth = space_metrics.advance.width.to_f32();
            split_text(
                &line.slice(range),
                tab_width,
                |n| {
                    let start = cursor_x.get();
                    let end = start + sp_awidth * n as f32;
                    if point.x < end {
                        let frac = ((point.x - start) * (n as f32) / (end - start)) as usize;
                        gidx.set(gidx.get() + frac);
                        SplitCbRes::Stop
                    } else {
                        cursor_x.set(cursor_x.get() + sp_awidth * n as f32);
                        gidx.set(gidx.get() + n);
                        SplitCbRes::Continue
                    }
                },
                |text| {
                    let shaped = text_font.shape(text, self.text_size, style);
                    let mut gis = shaped.glyph_infos.iter().peekable();
                    for j in text.grapheme_idxs() {
                        while let Some(cluster) = gis.peek().map(|gi| gi.cluster) {
                            if cluster >= j as u32 {
                                break;
                            }
                            let gi = gis.next().unwrap();
                            cursor_x.set(cursor_x.get() + gi.advance.width.to_f32());
                        }
                        if let Some(gi) = gis.peek() {
                            let aw = gi.advance.width.to_f32();
                            let x = cursor_x.get() + aw;
                            if point.x <= x {
                                return SplitCbRes::Stop;
                            }
                        }
                        gidx.set(gidx.get() + 1);
                    }
                    while let Some(gi) = gis.next() {
                        cursor_x.set(cursor_x.get() + gi.advance.width.to_f32());
                    }
                    return SplitCbRes::Continue;
                },
            );
        }

        self.cursor.line_gidx = gidx.get();
        self.cursor.sync_gidx(data, tab_width);
        self.snap_to_cursor(data, tab_width, styles, true);
    }

    pub(super) fn snap_to_cursor(
        &mut self,
        data: &Rope,
        tab_width: usize,
        styles: &StyleRanges,
        update_global_x: bool,
    ) {
        // Limit cursor based on cursor style
        self.cursor
            .limit_for_style(data, tab_width, update_global_x);

        // Snap to Y
        self.snap_to_line(self.cursor.line_num, data, tab_width, styles);

        // Snap to X
        let mut text_font = self.bed_handle.text_font();
        let space_metrics = text_font.space_metrics(self.text_size, TextStyle::default());
        let cursor_x = Cell::new(0.0);
        let cursor_block_width = Cell::new(space_metrics.advance.width.to_f32());
        let line_gidx = self.cursor.line_gidx;
        let gidx = Cell::new(0);

        let line = data.line(self.cursor.line_num);
        let start_cidx = data.line_to_char(self.cursor.line_num);
        let end_cidx = start_cidx + line.len_chars();

        for (range, &style) in styles.style.iter_range(start_cidx..end_cidx).unwrap() {
            let range = range.start - start_cidx..range.end - start_cidx;
            let space_metrics = text_font.space_metrics(self.text_size, style);
            let sp_awidth = space_metrics.advance.width.to_f32();
            split_text(
                &line.slice(range),
                tab_width,
                |n| {
                    if (gidx.get()..gidx.get() + n).contains(&line_gidx) {
                        cursor_x.set(cursor_x.get() + sp_awidth * (line_gidx - gidx.get()) as f32);
                        SplitCbRes::Stop
                    } else {
                        cursor_x.set(cursor_x.get() + sp_awidth * n as f32);
                        gidx.set(gidx.get() + n);
                        SplitCbRes::Continue
                    }
                },
                |text| {
                    let shaped = text_font.shape(text, self.text_size, style);
                    let mut gis = shaped.glyph_infos.iter().peekable();
                    for j in text.grapheme_idxs() {
                        while let Some(cluster) = gis.peek().map(|gi| gi.cluster) {
                            if cluster >= j as u32 {
                                break;
                            }
                            let gi = gis.next().unwrap();
                            cursor_x.set(cursor_x.get() + gi.advance.width.to_f32());
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
        }

        let cursor_width = if self.cursor.style == CursorStyle::Line {
            CURSOR_WIDTH
        } else {
            cursor_block_width.get()
        };
        let cursor_max_x = (cursor_x.get() + cursor_width).ceil() as i32;
        let cursor_min_x = cursor_x.get().floor() as i32;
        let rect_width = self.rect.width() as i32 - self.gutter_width(data) as i32;
        if self.off.x > cursor_min_x {
            self.off.x = cursor_min_x;
        } else if self.off.x + rect_width < cursor_max_x {
            self.off.x = cursor_max_x - rect_width;
        }

        self.bed_handle.request_redraw();
    }

    fn snap_to_line(&mut self, linum: usize, data: &Rope, tab_width: usize, styles: &StyleRanges) {
        self.sanity_check(data);
        if linum <= self.start_line {
            self.start_line = linum;
            self.off.y = 0;
        } else {
            let mut start_line = linum + 1;
            let mut height = 0;
            while start_line > 0 {
                start_line -= 1;
                let metrics = self.line_metrics(start_line, data, tab_width, styles);
                height += metrics.height;
                if height >= self.rect.height() {
                    self.start_line = start_line;
                    self.off.y = height as i32 - self.rect.height() as i32;
                    return;
                }
                if start_line == self.start_line {
                    return;
                }
            }
            self.start_line = 0;
            self.off.y = 0;
        }
    }

    pub(super) fn scroll(
        &mut self,
        scroll: Vector2D<i32, PixelSize>,
        data: &Rope,
        tab_width: usize,
        styles: &StyleRanges,
    ) {
        self.sanity_check(data);
        self.off += scroll;

        // Scroll y
        while self.off.y < 0 && self.start_line > 0 {
            self.start_line -= 1;
            let metrics = self.line_metrics(self.start_line, data, tab_width, styles);
            self.off.y += metrics.height as i32;
        }
        if self.off.y < 0 {
            self.off.y = 0;
        }
        while self.off.y > 0 {
            let metrics = self.line_metrics(self.start_line, data, tab_width, styles);
            if metrics.height as i32 > self.off.y {
                break;
            }
            if self.start_line == data.len_lines() - 1 {
                self.off.y = 0;
                break;
            }
            self.off.y -= metrics.height as i32;
            self.start_line += 1;
        }

        // Scroll X
        if self.off.x <= 0 {
            self.off.x = 0;
        } else {
            let mut height = -self.off.y;
            let mut max_xoff = 0i32;
            for linum in self.start_line..data.len_lines() {
                let metrics = self.line_metrics(linum, data, tab_width, styles);
                if metrics.width as i32 > max_xoff {
                    max_xoff = metrics.width as i32;
                }
                height += metrics.height as i32;
                if height >= self.rect.height() as i32 {
                    break;
                }
            }
            max_xoff -= self.rect.width() as i32 - self.gutter_width(data) as i32;
            if max_xoff < 0 {
                max_xoff = 0;
            }
            if self.off.x > max_xoff {
                self.off.x = max_xoff;
            }
        }

        self.bed_handle.request_redraw();
    }

    pub(super) fn draw(
        &mut self,
        painter: &mut Painter,
        data: &Rope,
        tab_width: usize,
        styles: &StyleRanges,
    ) {
        self.sanity_check(data);
        let gutter_width = self.gutter_width(data) as f32;
        let mut textview_rect = self.rect.cast();
        textview_rect.origin.x += gutter_width;
        textview_rect.size.width -= gutter_width;

        let theme = self.bed_handle.theme();
        let mut gutter_baselines = Vec::new();

        {
            let mut paint_ctx = painter.widget_ctx(textview_rect, theme.textview.background, false);
            let mut text_font = self.bed_handle.text_font();
            let mut text_ctx = text_font.render_ctx(&mut paint_ctx);

            let mut origin = point2(0.0f32, 0.0f32) - self.off.cast();
            let mut linum = self.start_line;
            let line_pad = f26_6::from(self.bed_handle.text_line_pad() as f32);

            let mut start_cidx = data.line_to_char(self.start_line);

            for rope_line in data.lines_at(self.start_line) {
                let end_cidx = start_cidx + rope_line.len_chars();
                let style_type = if start_cidx == end_cidx {
                    StyleType::Const(0..0, TextStyle::default(), theme.textview.foreground, false)
                } else {
                    StyleType::Range(styles.sub_range(start_cidx..end_cidx))
                };
                start_cidx = end_cidx;

                origin.y += line_pad.to_f32();
                if origin.y >= self.rect.height() as f32 {
                    break;
                }
                let text_cursor = if linum == self.cursor.line_num {
                    Some(TextCursor {
                        gidx: self.cursor.line_gidx,
                        style: self.cursor.style,
                        color: theme.textview.cursor,
                    })
                } else {
                    None
                };
                let (ascender, descender) = text_ctx.draw_line(
                    &rope_line,
                    style_type,
                    tab_width,
                    origin,
                    textview_rect.width(),
                    text_cursor,
                    self.text_size,
                    TextAlign::Left,
                );
                if self.show_gutter {
                    gutter_baselines.push(origin.y + ascender);
                }
                origin.y += ascender - descender + line_pad.to_f32();
                linum += 1;
            }
        }

        if self.show_gutter {
            let mut linum = self.start_line;
            let mut gutter_rect = textview_rect;
            gutter_rect.origin.x = self.rect.min_x() as f32;
            gutter_rect.size.width = gutter_width;
            let gutter_padding = self.bed_handle.gutter_padding() as f32;

            let mut paint_ctx = painter.widget_ctx(gutter_rect, theme.gutter.background, false);
            let mut text_font = self.bed_handle.gutter_font();
            let text_size = self.text_size.scale(self.bed_handle.gutter_font_scale());
            let metrics = text_font.metrics(text_size);
            let ascender = metrics.ascender.to_f32();
            let mut text_ctx = text_font.render_ctx(&mut paint_ctx);

            let mut buf = String::new();

            for base in gutter_baselines {
                linum += 1;
                buf.clear();
                write!(&mut buf, "{}", linum).unwrap();
                let origin = point2(gutter_padding, base - ascender);
                let fgcol = theme.gutter.foreground;
                text_ctx.draw_line(
                    &buf.as_str(),
                    StyleType::Const(0..buf.len(), TextStyle::default(), fgcol, false),
                    tab_width,
                    origin,
                    gutter_rect.width() - gutter_padding,
                    None,
                    text_size,
                    TextAlign::Right,
                );
            }
        }
    }

    pub(super) fn scroll_to_top(&mut self) {
        self.start_line = 0;
        self.off = vec2(0, 0);
        self.bed_handle.request_redraw();
    }

    fn line_metrics(
        &self,
        linum: usize,
        data: &Rope,
        tab_width: usize,
        styles: &StyleRanges,
    ) -> LineMetrics {
        let mut text_font = self.bed_handle.text_font();
        let line = data.line(linum);
        let start_cidx = data.line_to_char(linum);
        let end_cidx = start_cidx + line.len_chars();
        let space_metrics = text_font.space_metrics(self.text_size, TextStyle::default());
        let state = RefCell::new((space_metrics.ascender, space_metrics.descender, 0.0));
        let line_pad = f26_6::from(self.bed_handle.text_line_pad() as f32);
        for (range, &style) in styles.style.iter_range(start_cidx..end_cidx).unwrap() {
            let range = range.start - start_cidx..range.end - start_cidx;
            let space_metrics = text_font.space_metrics(self.text_size, style);
            split_text(
                &line.slice(range),
                tab_width,
                |n| {
                    state.borrow_mut().2 += space_metrics.advance.width.to_f32() * n as f32;
                    SplitCbRes::Continue
                },
                |text| {
                    let mut inner = state.borrow_mut();
                    let shaped = text_font.shape(text, self.text_size, style);
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
        }
        let state = &*state.borrow();
        LineMetrics {
            height: (state.0 - state.1 + line_pad * 2.0).to_f32().ceil() as u32,
            width: state.2.ceil() as u32,
        }
    }

    fn gutter_width(&self, data: &Rope) -> u32 {
        if !self.show_gutter {
            0
        } else {
            let last_line = data.len_lines() - 1;
            let line_str = last_line.to_string();
            let mut font = self.bed_handle.gutter_font();
            let size = self.text_size.scale(self.bed_handle.gutter_font_scale());
            let shaped = font.shape(&line_str.as_str(), size, TextStyle::default());
            shaped.width.to_f32().ceil() as u32 + self.bed_handle.gutter_padding() * 2
        }
    }

    fn sanity_check(&mut self, data: &Rope) {
        if self.start_line >= data.len_lines() {
            self.start_line = data.len_lines() - 1;
            self.off = vec2(0, 0);
            self.bed_handle.request_redraw();
        }
    }
}

struct LineMetrics {
    height: u32,
    width: u32,
}

#[derive(Clone, Default)]
pub(super) struct ViewCursor {
    pub(super) cidx: usize,
    pub(super) line_num: usize,
    pub(super) line_cidx: usize,
    pub(super) line_gidx: usize,
    pub(super) line_global_x: usize,
    pub(super) style: CursorStyle,
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

    pub(super) fn sync_gidx(&mut self, data: &Rope, tab_width: usize) {
        let trimmed = rope_trim_newlines(data.line(self.line_num));
        let (cidx, gidx) = cidx_gidx_from_gidx(&trimmed, self.line_gidx, tab_width);
        self.line_cidx = cidx;
        self.line_gidx = gidx;
        self.line_global_x = self.line_gidx;
        self.cidx = data.line_to_char(self.line_num) + self.line_cidx;
    }

    pub(super) fn sync_global_x(&mut self, data: &Rope, tab_width: usize) {
        let trimmed = rope_trim_newlines(data.line(self.line_num));
        let (cidx, gidx) = cidx_gidx_from_global_x(&trimmed, self.line_global_x, tab_width);
        self.line_cidx = cidx;
        self.line_gidx = gidx;
        self.cidx = data.line_to_char(self.line_num) + self.line_cidx;
    }

    pub(super) fn sync_line_cidx_gidx_left(&mut self, data: &Rope, tab_width: usize) {
        let trimmed = rope_trim_newlines(data.line(self.line_num));
        let len_chars = trimmed.len_chars();
        if self.line_cidx >= len_chars {
            self.line_cidx = len_chars;
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
        let (cidx, gidx) = cidx_gidx_from_cidx(&trimmed, self.line_cidx, tab_width);
        self.line_cidx = cidx;
        self.line_gidx = gidx;
        self.line_global_x = self.line_gidx;
        self.cidx = data.line_to_char(self.line_num) + self.line_cidx;
    }

    fn past_end(&self) -> bool {
        self.style == CursorStyle::Line
    }

    fn limit_for_style(&mut self, data: &Rope, tab_width: usize, update_global_x: bool) {
        let trimmed = rope_trim_newlines(data.line(self.line_num));
        let len_chars = trimmed.len_chars();
        if !self.past_end() && self.line_cidx == len_chars && self.line_cidx > 0 {
            let (cidx, gidx) = cidx_gidx_from_cidx(&trimmed, self.line_cidx - 1, tab_width);
            self.line_cidx = cidx;
            self.line_gidx = gidx;
            if update_global_x {
                self.line_global_x = self.line_gidx;
            }
            self.cidx = data.line_to_char(self.line_num) + self.line_cidx;
        }
    }
}

fn cidx_gidx_from_cidx(slice: &RopeSlice, cidx: usize, tab_width: usize) -> (usize, usize) {
    let (mut gidx, mut ccount) = (0, 0);
    for (_, g) in RopeGraphemes::new(slice) {
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
) -> (usize, usize) {
    let (mut gcount, mut cidx) = (0, 0);
    let len_chars = slice.len_chars();
    for (_, g) in RopeGraphemes::new(slice) {
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

fn cidx_gidx_from_global_x(slice: &RopeSlice, global_x: usize, tab_width: usize) -> (usize, usize) {
    let (mut gidx, mut ccount) = (0, 0);
    let len_chars = slice.len_chars();
    for (_, g) in RopeGraphemes::new(slice) {
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
