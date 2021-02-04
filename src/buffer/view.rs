// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::{Cell, RefCell};
use std::fmt::Write;

use euclid::{point2, vec2, Point2D, Rect, Vector2D};
use ropey::{Rope, RopeSlice};

use crate::common::{
    rope_is_grapheme_boundary, rope_next_grapheme_boundary, rope_trim_newlines, split_text,
    PixelSize, RopeGraphemes, RopeOrStr, SplitCbRes,
};
use crate::config::Config;
use crate::painter::Painter;
use crate::style::{TextSize, TextStyle};
use crate::text::{f26_6, CursorStyle, StyleType, TextAlign, TextCursor, CURSOR_WIDTH};

use super::{buffer::BufferSharedState, BufferBedHandle, Mode};

pub(super) struct View {
    pub(super) cursor: ViewCursor,
    text_size: TextSize,
    rect: Rect<u32, PixelSize>,
    off: Vector2D<i32, PixelSize>,
    start_line: usize,
    bed_handle: BufferBedHandle,
    show_gutter: bool,
    status_height: u32,
    shared: BufferSharedState,
    mode: Mode,
}

impl View {
    pub(super) fn new(
        bed_handle: BufferBedHandle,
        rect: Rect<u32, PixelSize>,
        shared: BufferSharedState,
    ) -> View {
        let status_height = status_required_height(&bed_handle.config);
        assert!(status_height < rect.height());
        View {
            rect,
            text_size: bed_handle.text_font_size(),
            off: vec2(0, 0),
            start_line: 0,
            bed_handle,
            cursor: ViewCursor::default(),
            show_gutter: true,
            status_height,
            shared,
            mode: Mode::Normal,
        }
    }

    pub(super) fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.bed_handle.request_redraw();
    }

    pub(super) fn update_text_size(&mut self, diff: i16) {
        self.text_size += diff;
        self.status_height = status_required_height(&self.bed_handle.config);
        assert!(self.status_height < self.rect.height());
    }

    pub(super) fn set_rect(&mut self, rect: Rect<u32, PixelSize>) {
        self.rect = rect;
        assert!(self.status_height < self.rect.height());
        self.snap_to_cursor(true);
    }

    pub(super) fn move_cursor_to_first_line(&mut self) {
        self.cursor.line_num = self.start_line;
        self.cursor.line_cidx = 0;
        self.off.y = 0;
        let shared = self.shared.borrow();
        self.cursor
            .sync_line_cidx_gidx_left(&shared.rope, shared.tab_width);
    }

    pub(super) fn move_cursor_to_middle_line(&mut self) {
        let middle_linum = self.line_idx_below(self.view_height() / 2);
        self.cursor.line_num = middle_linum;
        self.cursor.line_cidx = 0;
        let shared = self.shared.borrow();
        self.cursor
            .sync_line_cidx_gidx_left(&shared.rope, shared.tab_width);
    }

    pub(super) fn move_cursor_to_last_line(&mut self) {
        let last_linum = self.line_idx_below(self.view_height());
        self.cursor.line_num = last_linum;
        self.cursor.line_cidx = 0;
        let shared = self.shared.borrow();
        self.cursor
            .sync_line_cidx_gidx_left(&shared.rope, shared.tab_width);
    }

    fn pixels_down(&mut self, npix: u32) {
        let linum = self.line_idx_below(npix);
        let shared = self.shared.borrow();
        let len_lines = shared.rope.len_lines();
        if linum >= len_lines - 1 {
            self.cursor.line_num = len_lines - 1;
        } else {
            let diff = linum - self.start_line;
            self.cursor.line_num += diff;
            if self.cursor.line_num >= len_lines {
                self.cursor.line_num = len_lines - 1;
            }
            self.start_line += diff;
            self.off.y = 0;
        }
        self.cursor.line_cidx = 0;
        self.cursor
            .sync_line_cidx_gidx_left(&shared.rope, shared.tab_width);
    }

    pub(super) fn half_page_down(&mut self) {
        self.pixels_down(self.view_height() / 2);
    }

    pub(super) fn page_down(&mut self) {
        self.pixels_down(self.view_height());
    }

    fn pixels_up(&mut self, npix: u32) {
        if self.start_line == 0 {
            self.cursor.line_num = 0;
        } else {
            let linum = self.line_idx_above(npix);
            let diff = self.start_line - linum;
            self.start_line -= diff;
            self.cursor.line_num -= diff;
        }
        self.off.y = 0;
        self.cursor.line_cidx = 0;
        let shared = self.shared.borrow();
        self.cursor
            .sync_line_cidx_gidx_left(&shared.rope, shared.tab_width);
    }

    pub(super) fn half_page_up(&mut self) {
        self.pixels_up(self.view_height() / 2);
    }

    pub(super) fn page_up(&mut self) {
        self.pixels_up(self.view_height());
    }

    pub(super) fn move_cursor_to_point(&mut self, mut point: Point2D<i32, PixelSize>) {
        self.sanity_check();
        let view_height = self.view_height() as i32;
        {
            let shared = self.shared.borrow();
            let data = &shared.rope;
            let tab_width = shared.tab_width;
            let styles = &shared.styles;

            let rect = self.rect.cast();
            assert!(rect.contains(point));
            point -= rect.origin.to_vector();
            point.x -= self.gutter_width() as i32;
            if point.x < 0 {
                // TODO: Handle click in gutter
                return;
            }

            // Find line
            self.cursor.line_num = self.start_line;
            let mut height = -self.off.y;
            for linum in self.start_line..data.len_lines() {
                let metrics = self.line_metrics(linum);
                height += metrics.height as i32;
                if height >= point.y {
                    if height > view_height {
                        self.off.y += height - view_height;
                    }
                    break;
                }
                self.cursor.line_num += 1;
                assert!(height < view_height);
            }
            if self.cursor.line_num >= data.len_lines() {
                self.cursor.line_num = data.len_lines() - 1;
            }
            // Trim lines from the top if we need to
            for linum in self.start_line..data.len_lines() {
                let metrics = self.line_metrics(linum);
                if self.off.y < metrics.height as i32 {
                    break;
                }
                self.off.y -= metrics.height as i32;
                self.start_line += 1;
            }

            // Check cursor position on the line
            let mut text_font = self.bed_handle.text_font();
            let mut style = TextStyle::default();
            let mut scale = 1.0;
            let mut space_metrics = text_font.space_metrics(self.text_size, style);
            let cursor_x = Cell::new(-self.off.x as f32);
            let gidx = Cell::new(0);
            let point = point.cast::<f32>();

            let line = data.line(self.cursor.line_num);
            let start_cidx = data.line_to_char(self.cursor.line_num);
            let end_cidx = start_cidx + line.len_chars();

            if end_cidx > start_cidx {
                for (range, cur_style, _, _, cur_scale) in styles.sub_range(start_cidx..end_cidx) {
                    let text_size = self.text_size.scale(cur_scale);
                    if cur_style != style || cur_scale != scale {
                        space_metrics = text_font.space_metrics(text_size, style);
                        style = cur_style;
                        scale = cur_scale;
                    }
                    let sp_awidth = space_metrics.advance.width.to_f32();
                    split_text(
                        &line.slice_with(range),
                        tab_width,
                        |n| {
                            let start = cursor_x.get();
                            let end = start + sp_awidth * n as f32;
                            if point.x < end {
                                let frac =
                                    ((point.x - start) * (n as f32) / (end - start)) as usize;
                                gidx.set(gidx.get() + frac);
                                SplitCbRes::Stop
                            } else {
                                cursor_x.set(cursor_x.get() + sp_awidth * n as f32);
                                gidx.set(gidx.get() + n);
                                SplitCbRes::Continue
                            }
                        },
                        |text| {
                            let shaped = text_font.shape(text, text_size, style);
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
            }

            self.cursor.line_gidx = gidx.get();
            self.cursor.sync_gidx(data, tab_width);
        }
        self.snap_to_cursor(true);
    }

    pub(super) fn snap_to_cursor(&mut self, update_global_x: bool) {
        // Snap to Y
        self.snap_to_line(self.cursor.line_num);

        let shared = self.shared.borrow();
        let data = &shared.rope;
        let tab_width = shared.tab_width;
        let styles = &shared.styles;

        // Limit cursor based on cursor style
        self.cursor
            .limit_for_style(data, tab_width, update_global_x);

        // Snap to X
        let shared = self.shared.borrow();
        let data = &shared.rope;
        let tab_width = shared.tab_width;

        let mut text_font = self.bed_handle.text_font();
        let mut style = TextStyle::default();
        let mut scale = 1.0;
        let mut space_metrics = text_font.space_metrics(self.text_size, style);
        let cursor_x = Cell::new(0.0);
        let cursor_block_width = Cell::new(space_metrics.advance.width.to_f32());
        let line_gidx = self.cursor.line_gidx;
        let gidx = Cell::new(0);

        let line = data.line(self.cursor.line_num);
        let start_cidx = data.line_to_char(self.cursor.line_num);
        let end_cidx = start_cidx + line.len_chars();

        if end_cidx > start_cidx {
            for (range, cur_style, _, _, cur_scale) in styles.sub_range(start_cidx..end_cidx) {
                let text_size = self.text_size.scale(cur_scale);
                if cur_style != style || cur_scale != scale {
                    space_metrics = text_font.space_metrics(text_size, style);
                    style = cur_style;
                    scale = cur_scale;
                }
                let sp_awidth = space_metrics.advance.width.to_f32();
                split_text(
                    &line.slice_with(range),
                    tab_width,
                    |n| {
                        if (gidx.get()..gidx.get() + n).contains(&line_gidx) {
                            cursor_x
                                .set(cursor_x.get() + sp_awidth * (line_gidx - gidx.get()) as f32);
                            SplitCbRes::Stop
                        } else {
                            cursor_x.set(cursor_x.get() + sp_awidth * n as f32);
                            gidx.set(gidx.get() + n);
                            SplitCbRes::Continue
                        }
                    },
                    |text| {
                        let shaped = text_font.shape(text, text_size, style);
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
        }

        let cursor_width = if self.cursor.style == CursorStyle::Line {
            CURSOR_WIDTH
        } else {
            cursor_block_width.get()
        };
        let cursor_max_x = (cursor_x.get() + cursor_width).ceil() as i32;
        let cursor_min_x = cursor_x.get().floor() as i32;
        let rect_width = self.rect.width() as i32 - self.gutter_width() as i32;
        if self.off.x > cursor_min_x {
            self.off.x = cursor_min_x;
        } else if self.off.x + rect_width < cursor_max_x {
            self.off.x = cursor_max_x - rect_width;
        }

        self.bed_handle.request_redraw();
    }

    fn snap_to_line(&mut self, linum: usize) {
        self.sanity_check();
        let view_height = self.view_height();
        if linum <= self.start_line {
            self.start_line = linum;
            self.off.y = 0;
        } else {
            let mut start_line = linum + 1;
            let mut height = 0;
            while start_line > 0 {
                start_line -= 1;
                let metrics = self.line_metrics(start_line);
                height += metrics.height;
                if height >= view_height {
                    self.start_line = start_line;
                    self.off.y = height as i32 - view_height as i32;
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

    pub(super) fn scroll(&mut self, scroll: Vector2D<i32, PixelSize>) {
        self.sanity_check();
        let view_height = self.view_height();
        self.off += scroll;

        // Scroll y
        while self.off.y < 0 && self.start_line > 0 {
            self.start_line -= 1;
            let metrics = self.line_metrics(self.start_line);
            self.off.y += metrics.height as i32;
        }
        if self.off.y < 0 {
            self.off.y = 0;
        }
        while self.off.y > 0 {
            let metrics = self.line_metrics(self.start_line);
            if metrics.height as i32 > self.off.y {
                break;
            }
            if self.start_line == self.shared.borrow().rope.len_lines() - 1 {
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
            for linum in self.start_line..self.shared.borrow().rope.len_lines() {
                let metrics = self.line_metrics(linum);
                if metrics.width as i32 > max_xoff {
                    max_xoff = metrics.width as i32;
                }
                height += metrics.height as i32;
                if height >= view_height as i32 {
                    break;
                }
            }
            max_xoff -= self.rect.width() as i32 - self.gutter_width() as i32;
            if max_xoff < 0 {
                max_xoff = 0;
            }
            if self.off.x > max_xoff {
                self.off.x = max_xoff;
            }
        }

        self.bed_handle.request_redraw();
    }

    pub(super) fn draw(&mut self, painter: &mut Painter) {
        self.sanity_check();
        let view_height = self.view_height();
        let gutter_width = self.gutter_width() as f32;
        let (status_left, status_right) = self.build_statusline();

        let shared = self.shared.borrow();
        let data = &shared.rope;
        let styles = &shared.styles;
        let tab_width = shared.tab_width;

        let mut textview_rect = self.rect.cast();
        textview_rect.origin.x += gutter_width;
        textview_rect.size.width -= gutter_width;
        textview_rect.size.height = view_height as f32;

        let theme = self.bed_handle.theme();
        let mut gutter_baselines = Vec::new();

        // Draw textview
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
                    StyleType::Const(
                        0..0,
                        TextStyle::default(),
                        theme.textview.foreground,
                        false,
                        1.0,
                    )
                } else {
                    StyleType::Range(styles.sub_range(start_cidx..end_cidx))
                };
                start_cidx = end_cidx;

                origin.y += line_pad.to_f32();
                if origin.y >= view_height as f32 {
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

        // Draw gutter
        if self.show_gutter {
            let mut linum = self.start_line;
            let mut gutter_rect = textview_rect;
            gutter_rect.origin.x = self.rect.min_x() as f32;
            gutter_rect.size.width = gutter_width;
            let gutter_padding = self.bed_handle.gutter_padding() as f32;
            let fgcol = theme.gutter.foreground;

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
                text_ctx.draw_line(
                    &buf.as_str(),
                    StyleType::Const(0..buf.len(), TextStyle::default(), fgcol, false, 1.0),
                    tab_width,
                    origin,
                    gutter_rect.width() - gutter_padding,
                    None,
                    text_size,
                    TextAlign::Right,
                );
            }
        }

        // Draw status bar
        {
            let cfg = &mut *self.bed_handle.config.borrow_mut();
            let status_font = &mut cfg.status_font;
            let mut status_rect = self.rect.cast();
            let hor_padding = cfg.status_padding_horizontal as f32;
            let ver_padding = cfg.status_padding_vertical as f32;
            status_rect.origin.y += status_rect.height() - self.status_height as f32;
            status_rect.size.height = self.status_height as f32;
            let mut paint_ctx = painter.widget_ctx(status_rect, theme.status.background, false);
            let mut text_ctx = status_font.render_ctx(&mut paint_ctx);
            let origin = point2(hor_padding, ver_padding);
            let text_size = cfg.textview_font_size.scale(cfg.status_font_scale);
            let fgcol = theme.status.foreground;

            text_ctx.draw_line(
                &status_left.as_str(),
                StyleType::Const(
                    0..status_left.len(),
                    TextStyle::default(),
                    fgcol,
                    false,
                    1.0,
                ),
                tab_width,
                origin,
                status_rect.width() - hor_padding,
                None,
                text_size,
                TextAlign::Left,
            );
            text_ctx.draw_line(
                &status_right.as_str(),
                StyleType::Const(
                    0..status_right.len(),
                    TextStyle::default(),
                    fgcol,
                    false,
                    1.0,
                ),
                tab_width,
                origin,
                status_rect.width() - hor_padding,
                None,
                text_size,
                TextAlign::Right,
            );
        }
    }

    pub(super) fn scroll_to_top(&mut self) {
        self.start_line = 0;
        self.off = vec2(0, 0);
        self.bed_handle.request_redraw();
    }

    fn line_idx_above(&self, npix: u32) -> usize {
        let mut linum = self.start_line;
        let mut height = self.off.y;
        while linum > 0 {
            if height >= npix as i32 {
                break;
            }
            let metrics = self.line_metrics(linum);
            height += metrics.height as i32;
            linum -= 1;
        }
        linum
    }

    fn line_idx_below(&self, npix: u32) -> usize {
        let shared = self.shared.borrow();
        let data = &shared.rope;
        let len_lines = data.len_lines();
        let mut linum = self.start_line;
        let mut height = -self.off.y;
        while linum + 1 < len_lines {
            let metrics = self.line_metrics(linum);
            height += metrics.height as i32;
            if height >= npix as i32 {
                break;
            }
            linum += 1;
        }
        linum
    }

    fn line_metrics(&self, linum: usize) -> LineMetrics {
        let shared = self.shared.borrow();
        let data = &shared.rope;
        let styles = &shared.styles;
        let tab_width = shared.tab_width;

        let mut text_font = self.bed_handle.text_font();
        let mut style = TextStyle::default();
        let mut scale = 1.0;
        let mut space_metrics = text_font.space_metrics(self.text_size, style);
        let line = data.line(linum);
        let start_cidx = data.line_to_char(linum);
        let end_cidx = start_cidx + line.len_chars();
        let state = RefCell::new((space_metrics.ascender, space_metrics.descender, 0.0));
        let line_pad = f26_6::from(self.bed_handle.text_line_pad() as f32);
        if end_cidx > start_cidx {
            for (range, cur_style, _, _, cur_scale) in styles.sub_range(start_cidx..end_cidx) {
                let text_size = self.text_size.scale(cur_scale);
                if cur_style != style || cur_scale != scale {
                    space_metrics = text_font.space_metrics(text_size, style);
                    style = cur_style;
                    scale = cur_scale;
                }
                split_text(
                    &line.slice_with(range),
                    tab_width,
                    |n| {
                        state.borrow_mut().2 += space_metrics.advance.width.to_f32() * n as f32;
                        SplitCbRes::Continue
                    },
                    |text| {
                        let mut inner = state.borrow_mut();
                        let shaped = text_font.shape(text, text_size, style);
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
        }
        let state = &*state.borrow();
        LineMetrics {
            height: (state.0 - state.1 + line_pad * 2.0).to_f32().ceil() as u32,
            width: state.2.ceil() as u32,
        }
    }

    fn gutter_width(&self) -> u32 {
        let shared = self.shared.borrow();
        let data = &shared.rope;
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

    fn sanity_check(&mut self) {
        let shared = self.shared.borrow();
        let data = &shared.rope;
        if self.start_line >= data.len_lines() {
            self.start_line = data.len_lines() - 1;
            self.off = vec2(0, 0);
            self.bed_handle.request_redraw();
        }
    }

    fn view_height(&self) -> u32 {
        self.rect.height() - self.status_height
    }

    fn build_statusline(&self) -> (String, String) {
        let cfg = self.bed_handle.config();
        let theme = &self.bed_handle.theme().status;
        let shared = self.shared.borrow();
        let mut var = String::new();
        let mut left = String::new();
        let mut right = String::new();

        let mut expand = |buf: &mut String, mut iter: std::str::Chars, sep| {
            while let Some(c) = iter.next() {
                match c {
                    '{' => match iter.next() {
                        Some('{') => buf.push('{'),
                        Some(c) => {
                            var.push(c);
                        }
                        _ => break,
                    },
                    '}' => {
                        if var.len() > 0 {
                            match var.as_ref() {
                                "mode" => buf.push_str(self.mode.to_str()),
                                "buffer" => buf.push_str(
                                    shared
                                        .optname
                                        .as_ref()
                                        .map(|s| s.as_str())
                                        .unwrap_or("[No Name]"),
                                ),
                                "encoding" => buf.push_str("utf-8"),
                                "language" => buf.push_str(
                                    shared.optlanguage.map(|l| l.to_str()).unwrap_or("none"),
                                ),
                                "line" => {
                                    write!(buf, "{}", self.cursor.line_num).unwrap();
                                }
                                "col" => {
                                    write!(buf, "{}", self.cursor.line_cidx).unwrap();
                                }
                                "sep" => buf.push_str(sep),
                                _ => {}
                            }
                            var.clear();
                        } else {
                            buf.push('}');
                        }
                    }
                    c => {
                        if var.len() > 0 {
                            var.push(c);
                        } else {
                            buf.push(c);
                        }
                    }
                }
            }
        };

        expand(&mut left, cfg.status_fmt_left.chars(), &theme.left_sep);
        expand(&mut right, cfg.status_fmt_right.chars(), &theme.right_sep);

        (left, right)
    }
}

fn status_required_height(config: &RefCell<Config>) -> u32 {
    let mut config_ref = config.borrow_mut();
    let font_size = config_ref
        .textview_font_size
        .scale(config_ref.status_font_scale);
    let font_metrics = config_ref.status_font.metrics(font_size);
    let height = (font_metrics.ascender - font_metrics.descender).to_f32()
        + (config_ref.status_padding_vertical as f32) * 2.0;
    height.ceil() as u32
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
        self.cidx = 0;
        self.line_num = 0;
        self.line_cidx = 0;
        self.line_gidx = 0;
        self.line_global_x = 0;
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
