// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::fs::File;
use std::io::Result as IOResult;
use std::rc::Rc;

use euclid::{point2, size2, Rect, Size2D};
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

    pub(crate) fn draw_view(&self, id: &BufferViewID, painter: &mut WidgetPainter) {
        let shaper = &mut *self.text_shaper.borrow_mut();
        let view = self.views.get(id).unwrap();
        let mut pos = point2(0, 0);
        for line in &self.shaped_lines {
            pos.y += line.metrics.ascender;
            for (gis, face, style, size, color, opt_under) in line.styled_iter() {
                if pos.x as u32 >= view.rect.size.width {
                    break;
                }
                let raster = shaper.get_raster(face, style).unwrap();
                let start_x = pos.x;
                for gi in gis {
                    painter.glyph(pos + gi.offset, face, gi.gid, size, color, style, raster);
                    pos.x += gi.advance.width;
                    if pos.x as u32 >= view.rect.size.width {
                        break;
                    }
                }
                if let Some(under) = opt_under {
                    painter.color_quad(
                        Rect::new(
                            point2(start_x, pos.y - line.metrics.underline_position),
                            size2(pos.x - start_x, line.metrics.underline_thickness),
                        )
                        .cast(),
                        under,
                    );
                }
            }
            pos.y -= line.metrics.descender;
            pos.x = 0;
            if pos.y as u32 >= view.rect.size.height {
                break;
            }
        }
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
