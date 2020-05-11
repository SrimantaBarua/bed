// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::fs::File;
use std::io::Result as IOResult;
use std::rc::Rc;

use euclid::{Rect, Size2D};
use fnv::FnvHashMap;
use ropey::{Rope, RopeSlice};

use crate::common::{PixelSize, DPI};
use crate::font::FaceKey;
use crate::painter::WidgetPainter;
use crate::style::{Color, TextSize, TextStyle};
use crate::text::{ShapedText, TextShaper};

use super::view::BufferView;
use super::BufferViewID;

pub(crate) struct Buffer {
    data: Rope,
    views: FnvHashMap<BufferViewID, BufferView>,
    tab_width: usize,
    // Text rendering
    text_shaper: Rc<RefCell<TextShaper>>,
    face_key: FaceKey,
    text_size: TextSize,
    dpi: Size2D<u32, DPI>,
    shaped_lines: Vec<ShapedText>,
}

impl Buffer {
    pub(crate) fn new_view(&mut self, id: &BufferViewID, rect: Rect<u32, PixelSize>) {
        self.views.insert(id.clone(), BufferView::new(rect));
    }

    pub(crate) fn set_view_rect(&mut self, id: &BufferViewID, rect: Rect<u32, PixelSize>) {
        self.views.get_mut(id).unwrap().rect = rect;
    }

    pub(crate) fn move_view_cursor(&mut self, id: &BufferViewID, dirn: crate::Direction) {
        let view = self.views.get_mut(id).unwrap();
        view.move_cursor(dirn, &self.data);
    }

    pub(crate) fn draw_view(&self, id: &BufferViewID, painter: &mut WidgetPainter) {
        let shaper = &mut *self.text_shaper.borrow_mut();
        let view = self.views.get(id).unwrap();
        view.draw(&self.shaped_lines, shaper, painter);
    }

    pub(crate) fn remove_view(&mut self, id: &BufferViewID) {
        self.views.remove(id);
    }

    pub(super) fn empty(
        text_shaper: Rc<RefCell<TextShaper>>,
        face_key: FaceKey,
        text_size: TextSize,
        dpi: Size2D<u32, DPI>,
    ) -> Buffer {
        Buffer {
            text_size: text_size,
            face_key: face_key,
            tab_width: 8,
            dpi: dpi,
            text_shaper: text_shaper,
            data: Rope::new(),
            shaped_lines: Vec::new(),
            views: FnvHashMap::default(),
        }
    }

    pub(super) fn from_file(
        path: &str,
        text_shaper: Rc<RefCell<TextShaper>>,
        face_key: FaceKey,
        text_size: TextSize,
        dpi: Size2D<u32, DPI>,
    ) -> IOResult<Buffer> {
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| {
                let mut ret = Buffer {
                    text_size: text_size,
                    face_key: face_key,
                    tab_width: 8,
                    dpi: dpi,
                    text_shaper: text_shaper,
                    data: rope,
                    shaped_lines: Vec::new(),
                    views: FnvHashMap::default(),
                };
                ret.shape_text();
                ret
            })
    }

    pub(super) fn reload_from_file(&mut self, path: &str) -> IOResult<()> {
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| self.data = rope)
    }

    fn shape_text(&mut self) {
        self.shaped_lines.clear();
        let shaper = &mut *self.text_shaper.borrow_mut();
        for line in self.data.lines() {
            let trimmed = trim_newlines(line);
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
            self.shaped_lines.push(shaped);
        }
    }
}

fn trim_newlines<'a>(line: RopeSlice<'a>) -> RopeSlice<'a> {
    let mut nchars = line.len_chars();
    let mut chars = line.chars_at(line.len_chars());
    while let Some(c) = chars.prev() {
        match c {
            '\n' | '\x0b' | '\x0c' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}' => nchars -= 1,
            _ => break,
        }
    }
    line.slice(..nchars)
}
