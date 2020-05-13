// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use euclid::{point2, size2, Rect, Size2D};
use ropey::Rope;

use crate::common::{rope_trim_newlines, PixelSize, DPI};
use crate::font::FaceKey;
use crate::painter::WidgetPainter;
use crate::style::{Color, TextSize, TextStyle};
use crate::text::{ShapedText, TextShaper};

use super::cursor::Cursor;

#[derive(Clone)]
pub(crate) struct BufferViewCreateParams {
    pub(crate) face_key: FaceKey,
    pub(crate) text_size: TextSize,
    pub(crate) dpi: Size2D<u32, DPI>,
    pub(crate) text_shaper: Rc<RefCell<TextShaper>>,
    pub(crate) rect: Rect<u32, PixelSize>,
}

const CURSOR_WIDTH: i32 = 2;

pub(super) struct BufferView {
    pub(super) cursor: Cursor,
    pub(super) rect: Rect<u32, PixelSize>,
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
}

impl BufferView {
    pub(super) fn new(params: BufferViewCreateParams, data: &Rope, tab_width: usize) -> BufferView {
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
        };
        view.fill_or_truncate_view(data);
        view
    }

    pub(super) fn set_rect(&mut self, rect: Rect<u32, PixelSize>, data: &Rope) {
        self.rect = rect;
        self.fill_or_truncate_view(data);
        self.snap_to_cursor(data);
    }

    pub(super) fn reshape_line(&mut self, data: &Rope, linum: usize) {
        if linum < self.start_line || linum >= self.start_line + self.shaped_lines.len() {
            return;
        }
        let shaper = &mut *self.text_shaper.borrow_mut();
        let line = data.line(linum);
        let trimmed = rope_trim_newlines(line);
        let len_chars = trimmed.len_chars();
        let shaped = shaper.shape_line_rope(
            trimmed,
            self.dpi,
            self.tab_width,
            &[(len_chars, self.face_key)],
            &[(len_chars, TextStyle::default())],
            &[(len_chars, self.text_size)],
            &[(len_chars, Color::new(0, 0, 0, 0xff))],
            &[(len_chars, None)],
        );
        self.shaped_lines[linum - self.start_line] = shaped;
    }

    pub(super) fn insert_line(&mut self, data: &Rope, linum: usize) {
        if linum < self.start_line {
            self.start_line += 1;
            return;
        }
        if linum >= self.start_line + self.shaped_lines.len() {
            return;
        }
        let shaper = &mut *self.text_shaper.borrow_mut();
        let line = data.line(linum);
        let trimmed = rope_trim_newlines(line);
        let len_chars = trimmed.len_chars();
        let shaped = shaper.shape_line_rope(
            trimmed,
            self.dpi,
            self.tab_width,
            &[(len_chars, self.face_key)],
            &[(len_chars, TextStyle::default())],
            &[(len_chars, self.text_size)],
            &[(len_chars, Color::new(0, 0, 0, 0xff))],
            &[(len_chars, None)],
        );
        if self.shaped_lines.len() as u32 * self.height >= self.rect.size.height {
            self.shaped_lines.pop_back();
        }
        self.shaped_lines.insert(linum - self.start_line, shaped);
    }

    pub(super) fn delete_line(&mut self, data: &Rope, linum: usize) {
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
        let trimmed = rope_trim_newlines(line);
        let len_chars = trimmed.len_chars();
        let shaped = shaper.shape_line_rope(
            trimmed,
            self.dpi,
            self.tab_width,
            &[(len_chars, self.face_key)],
            &[(len_chars, TextStyle::default())],
            &[(len_chars, self.text_size)],
            &[(len_chars, Color::new(0, 0, 0, 0xff))],
            &[(len_chars, None)],
        );
        self.shaped_lines.push_back(shaped);
    }

    pub(super) fn snap_to_cursor(&mut self, data: &Rope) {
        // Sync Y
        if self.cursor.line_num <= self.start_line {
            self.move_view_up_to_cursor(data);
        } else {
            self.move_view_down_to_cursor(data);
        }
        // Sync X
        let line = &self.shaped_lines[self.cursor.line_num - self.start_line];
        let mut gidx = 0;
        let mut cursor_x = 0;
        let cgidx = self.cursor.line_gidx;
        for (clusters, _, _, _, _, _) in line.styled_iter() {
            for clus in clusters {
                let width = clus.glyph_infos.iter().fold(0, |a, x| a + x.advance.width);
                if gidx + clus.num_graphemes <= cgidx {
                    gidx += clus.num_graphemes;
                    cursor_x += width;
                } else {
                    cursor_x += width * ((cgidx - gidx) as i32) / clus.num_graphemes as i32;
                    break;
                }
            }
        }
        let cursor_x = if cursor_x < 0 { 0u32 } else { cursor_x as u32 };
        if cursor_x < self.xoff {
            self.xoff = cursor_x;
        } else if cursor_x + CURSOR_WIDTH as u32 >= self.xoff + self.rect.size.width {
            self.xoff = cursor_x + CURSOR_WIDTH as u32 - self.rect.size.width;
        }
    }

    pub(super) fn draw(&self, painter: &mut WidgetPainter) {
        let shaper = &mut *self.text_shaper.borrow_mut();
        let mut pos = point2(-(self.xoff as i32), -(self.yoff as i32));
        let mut linum = 0;
        let (cline, cgidx) = (
            self.cursor.line_num - self.start_line,
            self.cursor.line_gidx,
        );
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
                    if linum == cline && gidx <= cgidx && gidx + cluster.num_graphemes > cgidx {
                        let cwidth = CURSOR_WIDTH;
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
            pos.y -= self.descender;
            pos.x = -(self.xoff as i32);
            linum += 1;
        }
    }

    fn fill_or_truncate_view(&mut self, data: &Rope) {
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
        for line in data.lines_at(self.start_line + self.shaped_lines.len()) {
            let trimmed = rope_trim_newlines(line);
            let len_chars = trimmed.len_chars();
            let shaped = shaper.shape_line_rope(
                trimmed,
                self.dpi,
                self.tab_width,
                &[(len_chars, self.face_key)],
                &[(len_chars, TextStyle::default())],
                &[(len_chars, self.text_size)],
                &[(len_chars, Color::new(0, 0, 0, 0xff))],
                &[(len_chars, None)],
            );
            height += self.height;
            self.shaped_lines.push_back(shaped);
            if height >= self.rect.size.height + self.yoff {
                break;
            }
        }
    }

    fn move_view_up_to_cursor(&mut self, data: &Rope) {
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
        for line in data.lines_at(self.cursor.line_num) {
            let trimmed = rope_trim_newlines(line);
            let len_chars = trimmed.len_chars();
            let shaped = shaper.shape_line_rope(
                trimmed,
                self.dpi,
                self.tab_width,
                &[(len_chars, self.face_key)],
                &[(len_chars, TextStyle::default())],
                &[(len_chars, self.text_size)],
                &[(len_chars, Color::new(0, 0, 0, 0xff))],
                &[(len_chars, None)],
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
            let mut num_lines = self.rect.size.height / self.height;
            if self.rect.size.height % self.height > 0 {
                num_lines += 1;
            }
            self.shaped_lines.truncate(num_lines as usize);
        }
    }

    fn move_view_down_to_cursor(&mut self, data: &Rope) {
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
        while let Some(line) = lines.prev() {
            let trimmed = rope_trim_newlines(line);
            let len_chars = trimmed.len_chars();
            let shaped = shaper.shape_line_rope(
                trimmed,
                self.dpi,
                self.tab_width,
                &[(len_chars, self.face_key)],
                &[(len_chars, TextStyle::default())],
                &[(len_chars, self.text_size)],
                &[(len_chars, Color::new(0, 0, 0, 0xff))],
                &[(len_chars, None)],
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
