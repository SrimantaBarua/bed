// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::fs::File;
use std::io::Result as IOResult;
use std::rc::Rc;

use euclid::{point2, size2, Rect, Size2D};
use fnv::FnvHashMap;
use ropey::Rope;

use crate::common::{PixelSize, DPI};
use crate::font::FaceKey;
use crate::painter::Painter;
use crate::style::{TextSize, TextStyle};
use crate::text::{ShapedTextLine, TextShaper};

use super::view::BufferView;
use super::BufferViewID;

pub(crate) struct Buffer {
    data: Rope,
    views: FnvHashMap<BufferViewID, BufferView>,
    // Text rendering
    text_shaper: Rc<RefCell<TextShaper>>,
    text_size: TextSize,
    face_key: FaceKey,
    shaped_lines: Vec<ShapedTextLine>,
}

impl Buffer {
    pub(crate) fn new_view(&mut self, id: &BufferViewID, rect: Rect<u32, PixelSize>) {
        self.views.insert(id.clone(), BufferView::new(rect));
    }

    pub(crate) fn set_view_rect(&mut self, id: &BufferViewID, rect: Rect<u32, PixelSize>) {
        self.views.get_mut(id).unwrap().rect = rect;
    }

    pub(crate) fn draw_view(&self, id: &BufferViewID, painter: &mut Painter) {
        let shaper = &mut *self.text_shaper.borrow_mut();
        let view = self.views.get(id).unwrap();
        let mut pos = view.rect.origin.cast();
        for line in &self.shaped_lines {
            pos.y += line.metrics.ascender;
            'outer: for span in &line.spans {
                let raster = shaper.get_raster(span.face, span.style).unwrap();
                for (gis, color, opt_under) in span.styled_iter() {
                    let start_x = pos.x;
                    for gi in gis {
                        painter.glyph(
                            pos + gi.offset,
                            span.face,
                            gi.gid,
                            self.text_size,
                            color,
                            span.style,
                            raster,
                        );
                        pos.x += gi.advance.width;
                        if (pos.x as u32) - view.rect.origin.x >= view.rect.size.width {
                            break;
                        }
                    }
                    if let Some(under) = opt_under {
                        painter.rect(
                            Rect::new(
                                point2(start_x, pos.y - line.metrics.underline_position),
                                size2(pos.x - start_x, line.metrics.underline_thickness),
                            )
                            .cast(),
                            under,
                        );
                    }
                    if (pos.x as u32) - view.rect.origin.x >= view.rect.size.width {
                        break 'outer;
                    }
                }
            }
            pos.y -= line.metrics.descender;
            pos.x = view.rect.origin.x as i32;
            if (pos.y as u32) - view.rect.origin.y >= view.rect.size.height {
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
            .map(|rope| Buffer {
                text_size: text_size,
                face_key: face_key,
                text_shaper: text_shaper,
                data: rope,
                shaped_lines: Vec::new(),
                views: FnvHashMap::default(),
            })
    }

    pub(super) fn reload_from_file(&mut self, path: &str) -> IOResult<()> {
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| self.data = rope)
    }
}
