// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::cmp::{max, min};
use std::collections::VecDeque;
use std::fmt::Write;
use std::rc::Rc;

use euclid::{point2, size2, Point2D, Rect, Size2D, Vector2D};
use ropey::Rope;

use crate::common::{rope_trim_newlines, PixelSize, DPI};
use crate::config::Config;
use crate::painter::Painter;
use crate::style::TextStyle;
use crate::text::{RopeOrStr, ShapedText, TextShaper};
use crate::theme::Theme;
use crate::{CURSOR_BLOCK_WIDTH, CURSOR_LINE_WIDTH};

use super::cursor::{Cursor, CursorStyle};
use super::styled::StyledText;

#[derive(Clone)]
pub(crate) struct BufferViewCreateParams {
    pub(crate) config: Rc<Config>,
    pub(crate) dpi: Size2D<u32, DPI>,
    pub(crate) text_shaper: Rc<RefCell<TextShaper>>,
    pub(crate) rect: Rect<u32, PixelSize>,
}

pub(super) struct BufferView {
    pub(super) cursor: Cursor,
    pub(super) rect: Rect<u32, PixelSize>,
    pub(super) needs_redraw: bool,
    pub(super) is_active: bool,
    tab_width: usize,
    // Text shaping
    dpi: Size2D<u32, DPI>,
    text_shaper: Rc<RefCell<TextShaper>>,
    // Shaped lines and gutter
    shaped_lines: VecDeque<ShapedText>,
    shaped_gutter: VecDeque<ShapedText>,
    ascender: i32,
    descender: i32,
    height: u32,
    // View start line, and offsets
    start_line: usize,
    yoff: u32,
    xoff: u32,
    // Gutter
    gutter_width: u32,
    // Completion popup
    completion_active: bool,
    completion_shaped: VecDeque<ShapedText>,
    completion_rect: Rect<u32, PixelSize>,
    completion_ascender: i32,
    completion_descender: i32,
    completion_height: u32,
    // Misc.
    config: Rc<Config>,
    theme: Rc<Theme>,
}

impl BufferView {
    pub(super) fn new(
        params: BufferViewCreateParams,
        theme: Rc<Theme>,
        data: &Rope,
        styled_lines: &[StyledText],
        tab_width: usize,
    ) -> BufferView {
        let config = params.config;
        let (ascender, descender) = {
            let shaper = &mut *params.text_shaper.borrow_mut();
            let raster = shaper
                .get_raster(config.textview_face, TextStyle::default())
                .unwrap();
            let metrics = raster.get_metrics(config.textview_font_size, params.dpi);
            (metrics.ascender, metrics.descender)
        };
        let (completion_ascender, completion_descender) = {
            let shaper = &mut *params.text_shaper.borrow_mut();
            let raster = shaper
                .get_raster(config.completion_face, TextStyle::default())
                .unwrap();
            let metrics = raster.get_metrics(config.completion_font_size, params.dpi);
            (metrics.ascender, metrics.descender)
        };
        let mut view = BufferView {
            cursor: Cursor::default(),
            rect: params.rect,
            needs_redraw: true,
            is_active: true,
            tab_width,
            dpi: params.dpi,
            text_shaper: params.text_shaper,
            ascender,
            descender,
            height: (ascender - descender) as u32,
            shaped_lines: VecDeque::new(),
            shaped_gutter: VecDeque::new(),
            start_line: 0,
            yoff: 0,
            xoff: 0,
            gutter_width: 0,
            completion_active: false,
            completion_shaped: VecDeque::new(),
            completion_rect: Rect::new(params.rect.origin, size2(0, 0)),
            completion_ascender,
            completion_descender,
            completion_height: (completion_ascender - completion_descender) as u32,
            config,
            theme,
        };
        view.fill_or_truncate_view(data, styled_lines);
        view.update_gutter_width(data);
        view
    }

    pub(crate) fn deactivate(&mut self) {
        self.shaped_lines.clear();
        self.shaped_gutter.clear();
        self.is_active = false;
    }

    pub(crate) fn activate(&mut self, data: &Rope, styled_lines: &[StyledText]) {
        self.fill_or_truncate_view(data, styled_lines);
        self.update_gutter_width(data);
        self.is_active = true;
        self.needs_redraw = true;
    }

    pub(crate) fn start_completion(&mut self, list: &[String]) {
        if list.len() == 0 {
            return;
        }
        let mut origin = match self.cursor_baseline_to_relative_point() {
            Some(point) => point,
            None => return,
        };
        let height_below = self.rect.size.height - (origin.y as i32 - self.descender) as u32;
        let height_above = origin.y - self.ascender as u32;
        let max_height = max(height_above, height_below);

        let (mut height, mut width) = (self.config.completion_padding_vertical * 2, 0);
        let shaper = &mut *self.text_shaper.borrow_mut();
        for item in list {
            if height + self.completion_height > max_height {
                break;
            }
            let rs = RopeOrStr::from(item.as_ref());
            let lc = rs.len_chars();
            let shaped = shaper.shape_line(
                rs,
                self.dpi,
                self.tab_width,
                &[(lc, self.config.completion_face)],
                &[(lc, TextStyle::default())],
                &[(lc, self.config.completion_font_size)],
                &[(lc, self.theme.completion.foreground)],
                &[(lc, None)],
            );
            height += self.completion_height;
            width = max(width, shaped.width() as u32);
            self.completion_shaped.push_back(shaped);
        }
        if self.completion_shaped.len() == 0 {
            return;
        }
        width += 2 * self.config.completion_padding_horizontal;

        if height > height_below {
            origin.y -= self.ascender as u32 + height;
        } else {
            origin.y = (origin.y as i32 - self.descender) as u32;
        }

        // TODO: Let completion popup extend beyond view, within limits of window
        width = min(width, self.rect.size.width - self.gutter_width);
        if origin.x + width > self.rect.size.width - self.gutter_width {
            origin.x = self.rect.size.width - self.gutter_width - width;
        }
        origin += self.rect.origin.to_vector();
        origin.x += self.gutter_width;
        self.completion_rect = Rect::new(origin, size2(width, height));

        self.completion_active = true;
        self.needs_redraw = true;
    }

    pub(crate) fn stop_completion(&mut self) {
        self.completion_shaped.clear();
        self.completion_active = false;
        self.needs_redraw = true;
    }

    pub(super) fn scroll(
        &mut self,
        vec: Vector2D<i32, PixelSize>,
        data: &Rope,
        styled_lines: &[StyledText],
    ) {
        // Scroll y
        if vec.y < 0 {
            let ysub = (-vec.y) as usize;
            let mut ycur = self.start_line * self.height as usize + self.yoff as usize;
            if ycur <= ysub {
                self.start_line = 0;
                self.yoff = 0;
            } else {
                ycur -= ysub;
                self.start_line = ycur / self.height as usize;
                self.yoff = (ycur % self.height as usize) as u32;
            }
        } else {
            let ycur = self.start_line * self.height as usize + self.yoff as usize + vec.y as usize;
            self.start_line = ycur / self.height as usize;
            self.yoff = (ycur % self.height as usize) as u32;
            if self.start_line >= data.len_lines() - 1 {
                self.start_line = data.len_lines() - 1;
                self.yoff = 0;
            }
        }
        self.shaped_lines.clear();
        self.shaped_gutter.clear();
        self.fill_or_truncate_view(data, styled_lines);
        self.update_gutter_width(data);

        // Scroll x
        if vec.x < 0 {
            self.xoff += (-vec.x) as u32;
        } else {
            if self.xoff < vec.x as u32 {
                self.xoff = 0;
            } else {
                self.xoff -= vec.x as u32;
            }
        }

        self.needs_redraw = true;
    }

    pub(super) fn move_cursor_to_point(
        &mut self,
        mut point: Point2D<u32, PixelSize>,
        data: &Rope,
        styled_lines: &[StyledText],
        tab_width: usize,
    ) {
        assert!(self.rect.contains(point));
        point.y -= self.rect.origin.y;
        point.x -= self.rect.origin.x;
        point.y += self.yoff;
        point.x += self.xoff;
        if point.x <= self.gutter_width {
            point.x = 0;
        } else {
            point.x -= self.gutter_width;
        }
        let linum = (point.y / self.height) as usize;
        self.cursor.line_num = self.start_line + linum;
        if self.cursor.line_num >= data.len_lines() {
            self.cursor.line_num = data.len_lines() - 1;
        }
        let line = &self.shaped_lines[self.cursor.line_num - self.start_line];
        let (mut gidx, mut x) = (0, 0);
        'outer: for (clusters, _, _, _, _, _) in line.styled_iter() {
            for clus in clusters {
                let width = clus.glyph_infos.iter().fold(0, |a, x| a + x.advance.width);
                if x + width < point.x as i32 {
                    x += width;
                    gidx += clus.num_graphemes;
                    continue;
                }
                let rem_width = point.x as i32 - x;
                gidx += ((rem_width * clus.num_graphemes as i32) / width) as usize;
                break 'outer;
            }
        }
        self.cursor.line_gidx = gidx;
        self.cursor.sync_gidx(data, tab_width);
        self.snap_to_cursor(data, styled_lines);
        self.needs_redraw = true;
    }

    pub(super) fn set_rect(
        &mut self,
        rect: Rect<u32, PixelSize>,
        data: &Rope,
        styled_lines: &[StyledText],
    ) {
        self.rect = rect;
        self.fill_or_truncate_view(data, styled_lines);
        self.snap_to_cursor(data, styled_lines);
        self.needs_redraw = true;
    }

    pub(super) fn reshape(&mut self, data: &Rope, styled_lines: &[StyledText]) {
        self.shaped_lines.clear();
        self.shaped_gutter.clear();
        self.fill_or_truncate_view(data, styled_lines);
        self.update_gutter_width(data);
        self.needs_redraw = true;
    }

    pub(super) fn snap_to_cursor(&mut self, data: &Rope, styled_lines: &[StyledText]) {
        // Sync Y
        if self.cursor.line_num <= self.start_line {
            self.move_view_up_to_cursor(data, styled_lines);
        } else {
            self.move_view_down_to_cursor(data, styled_lines);
        }
        // Sync X
        let line = &self.shaped_lines[self.cursor.line_num - self.start_line];
        let mut gidx = 0;
        let mut cursor_x = 0;
        let mut cursor_width = 0;
        let cgidx = self.cursor.line_gidx;
        for (clusters, _, _, _, _, _) in line.styled_iter() {
            for clus in clusters {
                let width = clus.glyph_infos.iter().fold(0, |a, x| a + x.advance.width);
                if gidx + clus.num_graphemes <= cgidx {
                    gidx += clus.num_graphemes;
                    cursor_x += width;
                } else {
                    cursor_x += width * ((cgidx - gidx) as i32) / clus.num_graphemes as i32;
                    cursor_width = width * clus.num_graphemes as i32;
                    break;
                }
            }
        }
        let cursor_x = if cursor_x < 0 { 0u32 } else { cursor_x as u32 };
        let cursor_width = if cursor_width <= 0 {
            match self.cursor.style {
                CursorStyle::Line => CURSOR_LINE_WIDTH as u32,
                _ => CURSOR_BLOCK_WIDTH as u32,
            }
        } else {
            cursor_width as u32
        };
        if cursor_x < self.xoff {
            self.xoff = cursor_x;
        } else if cursor_x + cursor_width + self.gutter_width >= self.xoff + self.rect.size.width {
            self.xoff = cursor_x + cursor_width + self.gutter_width - self.rect.size.width;
        }
        self.update_gutter_width(data);
        self.needs_redraw = true;
    }

    pub(super) fn draw(&mut self, painter: &mut Painter) {
        self.needs_redraw = false;
        let shaper = &mut *self.text_shaper.borrow_mut();

        let gutter_rect = Rect::new(
            self.rect.origin,
            size2(self.gutter_width, self.rect.size.height),
        );
        let text_rect = Rect::new(
            point2(self.rect.origin.x + self.gutter_width, self.rect.origin.y),
            size2(
                self.rect.size.width - self.gutter_width,
                self.rect.size.height,
            ),
        );

        // Draw gutter
        {
            let mut painter = painter.widget_ctx(gutter_rect.cast(), self.theme.gutter.background);
            let basex = (self.gutter_width - self.config.gutter_padding) as i32;
            let mut pos = point2(basex, -(self.yoff as i32));
            for line in &self.shaped_gutter {
                pos.x -= line.width();
                pos.y += self.ascender;
                painter.draw_shaped_text(shaper, pos, line, None, gutter_rect.size.width);
                pos.y -= self.descender;
                pos.x = basex;
            }
        }

        // Draw textview
        {
            let mut painter = painter.widget_ctx(text_rect.cast(), self.theme.textview.background);

            let mut ccolor = self.theme.textview.cursor;
            if self.cursor.style == CursorStyle::Block {
                ccolor = ccolor.opacity(50);
            }
            let cursor = if !self.cursor.visible || self.cursor.line_num < self.start_line {
                None
            } else {
                Some((self.cursor.line_gidx, ccolor, self.cursor.style))
            };

            let mut pos = point2(-(self.xoff as i32), -(self.yoff as i32));
            let mut linum = self.start_line;

            for line in &self.shaped_lines {
                let cursor = if linum == self.cursor.line_num {
                    painter.color_quad(
                        Rect::new(
                            point2(0, pos.y),
                            size2(self.rect.size.width, self.height).cast(),
                        ),
                        self.theme.textview.cursor_line,
                    );
                    cursor
                } else {
                    None
                };
                pos.y += self.ascender;
                painter.draw_shaped_text(shaper, pos, line, cursor, text_rect.size.width);
                pos.y -= self.descender;
                linum += 1;
            }
        }

        // Draw completion
        if self.completion_active {
            let mut painter = painter.widget_ctx(
                self.completion_rect.cast(),
                self.theme.completion.background,
            );
            let basex = self.config.completion_padding_horizontal as i32;
            let mut pos = point2(basex, self.config.completion_padding_vertical as i32);
            for line in &self.completion_shaped {
                pos.y += self.completion_ascender;
                painter.draw_shaped_text(
                    shaper,
                    pos,
                    line,
                    None,
                    self.completion_rect.size.width - (basex as u32) * 2,
                );
                pos.y -= self.completion_descender;
                pos.x = basex;
            }
        }
    }

    fn fill_or_truncate_view(&mut self, data: &Rope, styled_lines: &[StyledText]) {
        let mut buf = String::new();
        let shaper = &mut *self.text_shaper.borrow_mut();
        let mut height = self.shaped_lines.len() as u32 * self.height;
        if height >= self.rect.size.height {
            let mut num_lines = self.rect.size.height / self.height;
            if self.rect.size.height % self.height > 0 {
                num_lines += 1;
            }
            self.shaped_lines.truncate(num_lines as usize);
            self.shaped_gutter.truncate(num_lines as usize);
            return;
        }
        let mut linum = self.start_line + self.shaped_lines.len();
        for (line, styled) in data.lines_at(linum).zip(&styled_lines[linum..]) {
            // Shape textview
            let trimmed = rope_trim_newlines(line);
            let len_chars = trimmed.len_chars();
            let shaped = shaper.shape_line(
                trimmed.into(),
                self.dpi,
                self.tab_width,
                &[(len_chars, self.config.textview_face)],
                &styled.styles,
                &[(len_chars, self.config.textview_font_size)],
                &styled.colors,
                &styled.unders,
            );
            height += self.height;
            self.shaped_lines.push_back(shaped);

            // Shape gutter
            linum += 1;
            buf.clear();
            write!(&mut buf, "{}", linum).unwrap();
            let rs = RopeOrStr::from(buf.as_ref());
            let lc = rs.len_chars();
            let shaped = shaper.shape_line(
                rs,
                self.dpi,
                self.tab_width,
                &[(lc, self.config.gutter_face)],
                &[(lc, TextStyle::default())],
                &[(lc, self.config.gutter_font_size)],
                &[(lc, self.theme.gutter.foreground)],
                &[(lc, None)],
            );
            self.shaped_gutter.push_back(shaped);

            if height >= self.rect.size.height + self.yoff {
                break;
            }
        }
    }

    fn move_view_up_to_cursor(&mut self, data: &Rope, styled_lines: &[StyledText]) {
        let mut buf = String::new();
        self.yoff = 0;
        let shaper = &mut *self.text_shaper.borrow_mut();
        let mut cur_height = self.shaped_lines.len() as u32 * self.height;
        if self.start_line == self.cursor.line_num {
            if cur_height > self.rect.size.height + self.height {
                self.shaped_lines.pop_back();
                self.shaped_gutter.pop_back();
            }
            return;
        }
        let mut new_height = 0;
        let (mut new_shaped_lines, mut new_shaped_gutter) = (Vec::new(), Vec::new());
        let mut linum = self.cursor.line_num;

        for (line, styled) in data.lines_at(linum).zip(&styled_lines[linum..]) {
            // Shape textview
            let trimmed = rope_trim_newlines(line);
            let len_chars = trimmed.len_chars();
            let shaped = shaper.shape_line(
                trimmed.into(),
                self.dpi,
                self.tab_width,
                &[(len_chars, self.config.textview_face)],
                &styled.styles,
                &[(len_chars, self.config.textview_font_size)],
                &styled.colors,
                &styled.unders,
            );
            new_height += self.height;
            new_shaped_lines.push(shaped);

            // Shape gutter
            linum += 1;
            buf.clear();
            write!(&mut buf, "{}", linum).unwrap();
            let rs = RopeOrStr::from(buf.as_ref());
            let lc = rs.len_chars();
            let shaped = shaper.shape_line(
                rs,
                self.dpi,
                self.tab_width,
                &[(lc, self.config.gutter_face)],
                &[(lc, TextStyle::default())],
                &[(lc, self.config.gutter_font_size)],
                &[(lc, self.theme.gutter.foreground)],
                &[(lc, None)],
            );
            new_shaped_gutter.push(shaped);

            if new_height >= self.rect.size.height
                || self.cursor.line_num + new_shaped_lines.len() == self.start_line
            {
                break;
            }
        }
        self.start_line = self.cursor.line_num;
        if new_height >= self.rect.size.height {
            self.shaped_lines.clear();
            self.shaped_gutter.clear();
            for line in new_shaped_lines {
                self.shaped_lines.push_back(line);
            }
            for line in new_shaped_gutter {
                self.shaped_gutter.push_back(line);
            }
            return;
        }
        while let Some(line) = new_shaped_lines.pop() {
            self.shaped_lines.push_front(line);
        }
        while let Some(line) = new_shaped_gutter.pop() {
            self.shaped_gutter.push_front(line);
        }
        cur_height += new_height;
        if cur_height > self.rect.size.height {
            let num_lines = (self.rect.size.height + self.height - 1) / self.height;
            self.shaped_lines.truncate(num_lines as usize);
            self.shaped_gutter.truncate(num_lines as usize);
        }
    }

    fn move_view_down_to_cursor(&mut self, data: &Rope, styled_lines: &[StyledText]) {
        let mut buf = String::new();
        if self.cursor.line_num < self.start_line + self.shaped_lines.len() {
            let height = (self.cursor.line_num as u32 - self.start_line as u32 + 1) * self.height;
            if height > self.rect.size.height {
                self.yoff = height - self.rect.size.height;
            }
            return;
        }
        let shaped_end = self.start_line + self.shaped_lines.len();
        let shaper = &mut *self.text_shaper.borrow_mut();
        let mut height = 0;
        let (mut new_shaped_lines, mut new_shaped_gutter) = (Vec::new(), Vec::new());
        let mut lines = data.lines_at(self.cursor.line_num + 1);
        let mut linum = self.cursor.line_num + 1;

        while let Some(line) = lines.prev() {
            // Shape textview
            linum -= 1;
            let styled = &styled_lines[linum];
            let trimmed = rope_trim_newlines(line);
            let len_chars = trimmed.len_chars();
            let shaped = shaper.shape_line(
                trimmed.into(),
                self.dpi,
                self.tab_width,
                &[(len_chars, self.config.textview_face)],
                &styled.styles,
                &[(len_chars, self.config.textview_font_size)],
                &styled.colors,
                &styled.unders,
            );
            height += self.height;
            new_shaped_lines.push(shaped);

            // Shape gutter
            buf.clear();
            write!(&mut buf, "{}", linum + 1).unwrap();
            let rs = RopeOrStr::from(buf.as_ref());
            let lc = rs.len_chars();
            let shaped = shaper.shape_line(
                rs,
                self.dpi,
                self.tab_width,
                &[(lc, self.config.gutter_face)],
                &[(lc, TextStyle::default())],
                &[(lc, self.config.gutter_font_size)],
                &[(lc, self.theme.gutter.foreground)],
                &[(lc, None)],
            );
            new_shaped_gutter.push(shaped);

            if height >= self.rect.size.height
                || self.cursor.line_num < shaped_end + new_shaped_lines.len()
            {
                break;
            }
        }
        if height >= self.rect.size.height {
            self.shaped_lines.clear();
            self.shaped_gutter.clear();
            let mut i = self.cursor.line_num + 1;
            while let Some(line) = new_shaped_lines.pop() {
                self.shaped_lines.push_back(line);
                i -= 1;
            }
            while let Some(line) = new_shaped_gutter.pop() {
                self.shaped_gutter.push_back(line);
            }
            self.start_line = i;
            self.yoff = height - self.rect.size.height;
            return;
        }
        while let Some(line) = new_shaped_lines.pop() {
            self.shaped_lines.push_back(line);
        }
        while let Some(line) = new_shaped_gutter.pop() {
            self.shaped_gutter.push_back(line);
        }
        height = self.shaped_lines.len() as u32 * self.height;
        if height > self.rect.size.height {
            while let Some(line) = self.shaped_lines.pop_front() {
                if height < self.rect.size.height + self.height {
                    self.shaped_lines.push_front(line);
                    break;
                }
                self.shaped_gutter.pop_front();
                self.start_line += 1;
                height -= self.height;
            }
        }
    }

    fn update_gutter_width(&mut self, data: &Rope) {
        let shaper = &mut *self.text_shaper.borrow_mut();
        let buf = format!("{}", data.len_lines());
        let rs = RopeOrStr::from(buf.as_ref());
        let lc = rs.len_chars();
        let shaped = shaper.shape_line(
            rs,
            self.dpi,
            self.tab_width,
            &[(lc, self.config.gutter_face)],
            &[(lc, TextStyle::default())],
            &[(lc, self.config.gutter_font_size)],
            &[(lc, self.theme.gutter.foreground)],
            &[(lc, None)],
        );
        let width = shaped.width();
        let width = if width < 0 { 0u32 } else { width as u32 };
        self.gutter_width = self.config.gutter_padding * 2 + width;
    }

    fn cursor_baseline_to_relative_point(&self) -> Option<Point2D<u32, PixelSize>> {
        if !self.cursor.visible
            || self.cursor.line_num < self.start_line
            || self.cursor.line_num >= self.start_line + self.shaped_lines.len()
        {
            return None;
        }
        let y =
            (self.cursor.line_num - self.start_line) as u32 * self.height + self.ascender as u32;
        if y < self.yoff {
            return None;
        }
        let line = &self.shaped_lines[self.cursor.line_num - self.start_line];
        let (mut gidx, mut x) = (0, 0);
        'outer: for (clusters, _, _, _, _, _) in line.styled_iter() {
            for clus in clusters {
                let width = clus.glyph_infos.iter().fold(0, |a, x| a + x.advance.width);
                if gidx + clus.num_graphemes < self.cursor.line_gidx {
                    x += width;
                    gidx += clus.num_graphemes;
                    continue;
                }
                let rem_graphemes = self.cursor.line_gidx - gidx;
                x += ((rem_graphemes * width as usize) / clus.num_graphemes) as i32;
                break 'outer;
            }
        }
        if x >= self.xoff as i32 {
            Some(point2(x as u32 - self.xoff, y - self.yoff))
        } else {
            None
        }
    }
}
