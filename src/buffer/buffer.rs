// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::fs::File;
use std::io::Result as IOResult;
use std::rc::Rc;

use euclid::{size2, Rect, Size2D};
use fnv::FnvHashMap;
use ropey::Rope;

use crate::common::{PixelSize, DPI};
use crate::font::{FaceKey, FontCore, ScaledFaceMetrics};
use crate::painter::Painter;
use crate::style::{TextSize, TextStyle};
use crate::text::ShapedTextLine;

use super::view::BufferView;
use super::BufferViewID;

fn get_key_and_metrics(
    font_core: &Rc<RefCell<FontCore>>,
    size: TextSize,
    dpi: Size2D<u32, DPI>,
) -> (FaceKey, ScaledFaceMetrics) {
    let core = &mut *font_core.borrow_mut();
    let key = core.find("monospace").unwrap();
    let (_, font) = core.get(key, TextStyle::default()).unwrap();
    let metrics = font.raster.get_metrics(size, dpi);
    (key, metrics)
}

pub(crate) struct Buffer {
    data: Rope,
    views: FnvHashMap<BufferViewID, BufferView>,
    // Text rendering
    font_core: Rc<RefCell<FontCore>>,
    text_size: TextSize,
    face_key: FaceKey,
    face_metrics: ScaledFaceMetrics,
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
        let core = &mut *self.font_core.borrow_mut();
        let view = self.views.get(id).unwrap();
        let mut pos = view.rect.origin.cast();
        for line in &self.shaped_lines {
            pos.y += self.face_metrics.ascender;
            'outer: for span in &line.spans {
                let (_, font) = core.get(span.face, span.style).unwrap();
                for (gis, color, opt_under) in span.styled_iter() {
                    let start = pos;
                    for gi in gis {
                        painter.glyph(
                            pos + gi.offset,
                            span.face,
                            gi.gid,
                            self.text_size,
                            color,
                            span.style,
                            &mut font.raster,
                        );
                        pos.x += gi.advance.width;
                        if (pos.x as u32) - view.rect.origin.x >= view.rect.size.width {
                            break;
                        }
                    }
                    if let Some(under) = opt_under {
                        painter.rect(
                            Rect::new(
                                start,
                                size2(pos.x - start.x, self.face_metrics.underline_thickness),
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
            pos.y -= self.face_metrics.descender;
            pos.x = view.rect.origin.x as i32;
            if (pos.y as u32) - view.rect.origin.y >= view.rect.size.height {
                break;
            }
        }
    }

    pub(crate) fn remove_view(&mut self, id: &BufferViewID) {
        self.views.remove(id);
    }

    pub(super) fn empty(font_core: Rc<RefCell<FontCore>>, dpi: Size2D<u32, DPI>) -> Buffer {
        let size = TextSize::from_f32(10.0);
        let (key, metrics) = get_key_and_metrics(&font_core, size, dpi);
        Buffer {
            text_size: size,
            face_key: key,
            font_core: font_core,
            data: Rope::new(),
            shaped_lines: Vec::new(),
            face_metrics: metrics,
            views: FnvHashMap::default(),
        }
    }

    pub(super) fn from_file(
        path: &str,
        font_core: Rc<RefCell<FontCore>>,
        dpi: Size2D<u32, DPI>,
    ) -> IOResult<Buffer> {
        let size = TextSize::from_f32(10.0);
        let (key, metrics) = get_key_and_metrics(&font_core, size, dpi);
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| Buffer {
                text_size: size,
                face_key: key,
                font_core: font_core,
                data: rope,
                shaped_lines: Vec::new(),
                face_metrics: metrics,
                views: FnvHashMap::default(),
            })
    }

    pub(super) fn reload_from_file(&mut self, path: &str) -> IOResult<()> {
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| self.data = rope)
    }
}
