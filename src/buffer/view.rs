// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::collections::VecDeque;
use std::ops::Range;
use std::rc::Rc;

use euclid::{point2, size2, Point2D, Rect, Size2D, Vector2D};
use ropey::Rope;

use crate::common::{rope_trim_newlines, PixelSize, DPI};
use crate::font::FaceKey;
use crate::painter::Painter;
use crate::style::{Color, TextSize, TextStyle};
use crate::text::{ShapedText, TextShaper};
use crate::theme::Theme;

use super::cursor::{Cursor, CursorStyle};

const CURSOR_LINE_WIDTH: i32 = 2;
const CURSOR_BLOCK_WIDTH: i32 = 10;

// All indices here are codepoint indices
#[derive(Debug)]
pub(super) struct StyledText {
    styles: Vec<(usize, TextStyle)>,
    colors: Vec<(usize, Color)>,
    unders: Vec<(usize, Option<Color>)>,
}

impl StyledText {
    pub(super) fn empty() -> StyledText {
        StyledText {
            styles: Vec::new(),
            colors: Vec::new(),
            unders: Vec::new(),
        }
    }

    pub(super) fn new(
        len: usize,
        style: TextStyle,
        color: Color,
        under: Option<Color>,
    ) -> StyledText {
        StyledText {
            styles: vec![(len, style)],
            colors: vec![(len, color)],
            unders: vec![(len, under)],
        }
    }

    pub(super) fn push(
        &mut self,
        len: usize,
        style: TextStyle,
        color: Color,
        under: Option<Color>,
    ) {
        let style_len = self.styles.len();
        let color_len = self.colors.len();
        let under_len = self.unders.len();
        if style_len == 0 {
            self.styles.push((len, style));
        } else if self.styles[style_len - 1].1 != style {
            self.styles
                .push((self.styles[style_len - 1].0 + len, style));
        } else {
            self.styles[style_len - 1].0 += len;
        }
        if color_len == 0 {
            self.colors.push((len, color));
        } else if self.colors[color_len - 1].1 != color {
            self.colors
                .push((self.colors[color_len - 1].0 + len, color));
        } else {
            self.colors[color_len - 1].0 += len;
        }
        if under_len == 0 {
            self.unders.push((len, under));
        } else if self.unders[under_len - 1].1 != under {
            self.unders
                .push((self.unders[under_len - 1].0 + len, under));
        } else {
            self.unders[under_len - 1].0 += len;
        }
    }

    // TODO: Cut down on code duplication
    pub(super) fn set(
        &mut self,
        range: Range<usize>,
        style: TextStyle,
        color: Color,
        under: Option<Color>,
    ) {
        self.set_style(range.clone(), style);
        self.set_color(range.clone(), color);
        self.set_under(range, under);
    }

    // TODO: Merge adjacent if required
    fn set_style(&mut self, range: Range<usize>, style: TextStyle) {
        let mut i = match self.styles.binary_search_by_key(&range.start, |x| x.0) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        assert!(i < self.styles.len());
        if range.start == 0 || (i > 0 && range.start == self.styles[i - 1].0) {
            self.styles.insert(i, (range.end, style));
            i += 1;
        } else {
            self.styles.insert(i, (range.start, self.styles[i].1));
            self.styles.insert(i + 1, (range.end, style));
            i += 2;
        }
        // Remove everything after this that is completely covered
        while i < self.styles.len() && self.styles[i].0 <= range.end {
            self.styles.remove(i);
        }
    }

    fn set_color(&mut self, range: Range<usize>, color: Color) {
        assert!(self.colors.len() > 0 && range.start < self.colors[self.colors.len() - 1].0);
        let mut i = match self.colors.binary_search_by_key(&range.start, |x| x.0) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        assert!(i < self.colors.len());
        if range.start == 0 || (i > 0 && range.start == self.colors[i - 1].0) {
            self.colors.insert(i, (range.end, color));
            i += 1;
        } else {
            self.colors.insert(i, (range.start, self.colors[i].1));
            self.colors.insert(i + 1, (range.end, color));
            i += 2;
        }
        // Remove everything after this that is completely covered
        while i < self.colors.len() && self.colors[i].0 <= range.end {
            self.colors.remove(i);
        }
    }

    fn set_under(&mut self, range: Range<usize>, under: Option<Color>) {
        let mut i = match self.unders.binary_search_by_key(&range.start, |x| x.0) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        assert!(i < self.unders.len());
        if range.start == 0 || (i > 0 && range.start == self.unders[i - 1].0) {
            self.unders.insert(i, (range.end, under));
            i += 1;
        } else {
            self.unders.insert(i, (range.start, self.unders[i].1));
            self.unders.insert(i + 1, (range.end, under));
            i += 2;
        }
        // Remove everything after this that is completely covered
        while i < self.unders.len() && self.unders[i].0 <= range.end {
            self.unders.remove(i);
        }
    }
}

#[derive(Clone)]
pub(crate) struct BufferViewCreateParams {
    pub(crate) face_key: FaceKey,
    pub(crate) text_size: TextSize,
    pub(crate) dpi: Size2D<u32, DPI>,
    pub(crate) text_shaper: Rc<RefCell<TextShaper>>,
    pub(crate) rect: Rect<u32, PixelSize>,
}

pub(super) struct BufferView {
    pub(super) cursor: Cursor,
    pub(super) rect: Rect<u32, PixelSize>,
    pub(super) hl_valid_upto: usize,
    pub(super) needs_redraw: bool,
    tab_width: usize,
    // Text shaping
    face_key: FaceKey,
    text_size: TextSize,
    dpi: Size2D<u32, DPI>,
    text_shaper: Rc<RefCell<TextShaper>>,
    // Shaped lines, and the first shaped line
    shaped_lines: VecDeque<ShapedText>,
    ascender: i32,
    descender: i32,
    height: u32,
    // View start line, and offsets
    start_line: usize,
    yoff: u32,
    xoff: u32,
    // Misc.
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
        let (ascender, descender) = {
            let shaper = &mut *params.text_shaper.borrow_mut();
            let raster = shaper
                .get_raster(params.face_key, TextStyle::default())
                .unwrap();
            let metrics = raster.get_metrics(params.text_size, params.dpi);
            (metrics.ascender, metrics.descender)
        };
        let mut view = BufferView {
            cursor: Cursor::default(),
            rect: params.rect,
            hl_valid_upto: 0,
            needs_redraw: true,
            tab_width,
            face_key: params.face_key,
            text_size: params.text_size,
            dpi: params.dpi,
            text_shaper: params.text_shaper,
            ascender,
            descender,
            height: (ascender - descender) as u32,
            shaped_lines: VecDeque::new(),
            start_line: 0,
            yoff: 0,
            xoff: 0,
            theme: theme,
        };
        view.fill_or_truncate_view(data, styled_lines);
        view
    }

    pub(super) fn max_line_visible(&self) -> usize {
        let len = self.rect.size.height / self.height;
        if self.rect.size.height % self.height != 0 {
            self.start_line + len as usize + 1
        } else {
            self.start_line + len as usize
        }
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
        self.fill_or_truncate_view(data, styled_lines);
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
        self.fill_or_truncate_view(data, styled_lines);
        self.needs_redraw = true;
    }

    pub(super) fn insert_line(&mut self, data: &Rope, styled_lines: &[StyledText], linum: usize) {
        if linum < self.start_line {
            self.start_line += 1;
            return;
        }
        if linum >= self.max_line_visible() {
            return;
        }
        let shaper = &mut *self.text_shaper.borrow_mut();
        let line = data.line(linum);
        let styled = &styled_lines[linum];
        let trimmed = rope_trim_newlines(line);
        let len_chars = trimmed.len_chars();
        let shaped = shaper.shape_line_rope(
            trimmed,
            self.dpi,
            self.tab_width,
            &[(len_chars, self.face_key)],
            &styled.styles,
            &[(len_chars, self.text_size)],
            &styled.colors,
            &styled.unders,
        );
        let height = self.shaped_lines.len() as u32 * self.height;
        if linum == self.start_line + self.shaped_lines.len() {
            if height >= self.rect.size.height {
                self.shaped_lines.pop_front();
                self.start_line += 1;
            }
        } else {
            if height >= self.rect.size.height {
                self.shaped_lines.pop_back();
            }
        }
        self.shaped_lines.insert(linum - self.start_line, shaped);
        self.needs_redraw = true;
    }

    pub(super) fn delete_line(&mut self, data: &Rope, styled_lines: &[StyledText], linum: usize) {
        if linum < self.start_line {
            self.start_line -= 1;
            return;
        }
        if linum >= self.start_line + self.shaped_lines.len() {
            return;
        }
        self.shaped_lines.remove(linum - self.start_line);
        let shaper = &mut *self.text_shaper.borrow_mut();
        let last_line = self.start_line + self.shaped_lines.len();
        if last_line >= data.len_lines() {
            return;
        }
        let line = data.line(self.start_line + self.shaped_lines.len());
        let styled = &styled_lines[self.start_line + self.shaped_lines.len()];
        let trimmed = rope_trim_newlines(line);
        let len_chars = trimmed.len_chars();
        let shaped = shaper.shape_line_rope(
            trimmed,
            self.dpi,
            self.tab_width,
            &[(len_chars, self.face_key)],
            &styled.styles,
            &[(len_chars, self.text_size)],
            &styled.colors,
            &styled.unders,
        );
        self.shaped_lines.push_back(shaped);
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
        } else if cursor_x + cursor_width >= self.xoff + self.rect.size.width {
            self.xoff = cursor_x + cursor_width - self.rect.size.width;
        }
        self.needs_redraw = true;
    }

    pub(super) fn draw(&mut self, painter: &mut Painter) {
        let mut painter = painter.widget_ctx(self.rect.cast(), self.theme.textview.background);
        self.needs_redraw = false;
        let shaper = &mut *self.text_shaper.borrow_mut();
        let mut pos = point2(-(self.xoff as i32), -(self.yoff as i32));
        let mut linum = 0;
        let cursor = if self.cursor.line_num < self.start_line {
            None
        } else {
            Some((
                self.cursor.line_num - self.start_line,
                self.cursor.line_gidx,
            ))
        };
        let mut ccolor = self.theme.textview.cursor;
        if self.cursor.style == CursorStyle::Block {
            ccolor = ccolor.opacity(50);
        }
        for line in &self.shaped_lines {
            pos.y += self.ascender;
            let mut gidx = 0;

            for (clusters, face, style, size, color, opt_under) in line.styled_iter() {
                for cluster in clusters {
                    if pos.x >= self.rect.size.width as i32 {
                        break;
                    }
                    let raster = shaper.get_raster(face, style).unwrap();
                    let start_x = pos.x;
                    for gi in cluster.glyph_infos {
                        if pos.x + gi.offset.width + gi.advance.width <= 0 {
                            pos.x += gi.advance.width;
                            continue;
                        }
                        painter.glyph(pos + gi.offset, face, gi.gid, size, color, style, raster);
                        pos.x += gi.advance.width;
                    }
                    if pos.x <= 0 {
                        gidx += cluster.num_graphemes;
                        continue;
                    }
                    let width = pos.x - start_x;
                    if let Some((cline, cgidx)) = cursor {
                        if linum == cline && gidx <= cgidx && gidx + cluster.num_graphemes > cgidx {
                            let mut cx =
                                (width * (cgidx - gidx) as i32) / cluster.num_graphemes as i32;
                            cx += start_x;
                            let cwidth = match self.cursor.style {
                                CursorStyle::Line => CURSOR_LINE_WIDTH,
                                _ => width / cluster.num_graphemes as i32,
                            };
                            let (cy, cheight) = match self.cursor.style {
                                CursorStyle::Underline => (
                                    pos.y - line.metrics.underline_position,
                                    line.metrics.underline_thickness,
                                ),
                                _ => (
                                    pos.y - line.metrics.ascender,
                                    line.metrics.ascender - line.metrics.descender,
                                ),
                            };
                            painter.color_quad(
                                Rect::new(point2(cx, cy), size2(cwidth, cheight)),
                                ccolor,
                            );
                        }
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
            if let Some((cline, cgidx)) = cursor {
                if linum == cline && gidx == cgidx {
                    let cwidth = match self.cursor.style {
                        CursorStyle::Line => CURSOR_LINE_WIDTH,
                        _ => CURSOR_BLOCK_WIDTH,
                    };
                    let (cy, cheight) = match self.cursor.style {
                        CursorStyle::Underline => (
                            pos.y - line.metrics.underline_position,
                            line.metrics.underline_thickness,
                        ),
                        _ => (
                            pos.y - line.metrics.ascender,
                            line.metrics.ascender - line.metrics.descender,
                        ),
                    };
                    painter
                        .color_quad(Rect::new(point2(pos.x, cy), size2(cwidth, cheight)), ccolor);
                }
            }
            pos.y -= self.descender;
            pos.x = -(self.xoff as i32);
            linum += 1;
        }
    }

    fn fill_or_truncate_view(&mut self, data: &Rope, styled_lines: &[StyledText]) {
        let shaper = &mut *self.text_shaper.borrow_mut();
        let mut height = self.shaped_lines.len() as u32 * self.height;
        if height >= self.rect.size.height {
            let mut num_lines = self.rect.size.height / self.height;
            if self.rect.size.height % self.height > 0 {
                num_lines += 1;
            }
            self.shaped_lines.truncate(num_lines as usize);
            return;
        }
        let start = self.start_line + self.shaped_lines.len();
        for (line, styled) in data.lines_at(start).zip(&styled_lines[start..]) {
            // println!("styled: {:?}", styled);
            let trimmed = rope_trim_newlines(line);
            let len_chars = trimmed.len_chars();
            let shaped = shaper.shape_line_rope(
                trimmed,
                self.dpi,
                self.tab_width,
                &[(len_chars, self.face_key)],
                &styled.styles,
                &[(len_chars, self.text_size)],
                &styled.colors,
                &styled.unders,
            );
            height += self.height;
            self.shaped_lines.push_back(shaped);
            if height >= self.rect.size.height + self.yoff {
                break;
            }
        }
    }

    fn move_view_up_to_cursor(&mut self, data: &Rope, styled_lines: &[StyledText]) {
        self.yoff = 0;
        let shaper = &mut *self.text_shaper.borrow_mut();
        let mut cur_height = self.shaped_lines.len() as u32 * self.height;
        if self.start_line == self.cursor.line_num {
            if cur_height > self.rect.size.height + self.height {
                self.shaped_lines.pop_back();
            }
            return;
        }
        let mut new_height = 0;
        let mut new_shaped_lines = Vec::new();
        let cline = self.cursor.line_num;
        for (line, styled) in data.lines_at(cline).zip(&styled_lines[cline..]) {
            let trimmed = rope_trim_newlines(line);
            let len_chars = trimmed.len_chars();
            let shaped = shaper.shape_line_rope(
                trimmed,
                self.dpi,
                self.tab_width,
                &[(len_chars, self.face_key)],
                &styled.styles,
                &[(len_chars, self.text_size)],
                &styled.colors,
                &styled.unders,
            );
            new_height += self.height;
            new_shaped_lines.push(shaped);
            if new_height >= self.rect.size.height
                || self.cursor.line_num + new_shaped_lines.len() == self.start_line
            {
                break;
            }
        }
        self.start_line = self.cursor.line_num;
        if new_height >= self.rect.size.height {
            self.shaped_lines.clear();
            for line in new_shaped_lines {
                self.shaped_lines.push_back(line);
            }
            return;
        }
        while let Some(line) = new_shaped_lines.pop() {
            self.shaped_lines.push_front(line);
        }
        cur_height += new_height;
        if cur_height > self.rect.size.height {
            let num_lines = (self.rect.size.height + self.height - 1) / self.height;
            self.shaped_lines.truncate(num_lines as usize);
        }
    }

    fn move_view_down_to_cursor(&mut self, data: &Rope, styled_lines: &[StyledText]) {
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
        let mut new_shaped_lines = Vec::new();
        let mut lines = data.lines_at(self.cursor.line_num + 1);
        let mut linum = self.cursor.line_num + 1;

        // TODO
        while let Some(line) = lines.prev() {
            linum -= 1;
            let styled = &styled_lines[linum];
            let trimmed = rope_trim_newlines(line);
            let len_chars = trimmed.len_chars();
            let shaped = shaper.shape_line_rope(
                trimmed,
                self.dpi,
                self.tab_width,
                &[(len_chars, self.face_key)],
                &styled.styles,
                &[(len_chars, self.text_size)],
                &styled.colors,
                &styled.unders,
            );
            height += self.height;
            new_shaped_lines.push(shaped);
            if height >= self.rect.size.height
                || self.cursor.line_num < shaped_end + new_shaped_lines.len()
            {
                break;
            }
        }
        if height >= self.rect.size.height {
            self.shaped_lines.clear();
            let mut i = self.cursor.line_num + 1;
            while let Some(line) = new_shaped_lines.pop() {
                self.shaped_lines.push_back(line);
                i -= 1;
            }
            self.start_line = i;
            self.yoff = height - self.rect.size.height;
            return;
        }
        while let Some(line) = new_shaped_lines.pop() {
            self.shaped_lines.push_back(line);
        }
        height = self.shaped_lines.len() as u32 * self.height;
        if height > self.rect.size.height {
            while let Some(line) = self.shaped_lines.pop_front() {
                if height < self.rect.size.height + self.height {
                    self.shaped_lines.push_front(line);
                    break;
                }
                self.start_line += 1;
                height -= self.height;
            }
        }
    }
}
