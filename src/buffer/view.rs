// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::{Cell, RefCell};

use euclid::{point2, size2, vec2, Rect, Vector2D};
use ropey::{Rope, RopeSlice};
use unicode_segmentation::UnicodeSegmentation;

use crate::buffer::BufferBedHandle;
use crate::common::PixelSize;
use crate::painter::Painter;
use crate::style::{Color, TextStyle};
use crate::text::{split_text, ShapedSpan};

const CUSROR_WIDTH: f32 = 2.0;

enum SpanOrSpace {
    Span(ShapedSpan),
    Space(usize),
}

pub(super) struct View {
    cursor: ViewCursor,
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

    pub(super) fn set_rect(&mut self, rect: Rect<f32, PixelSize>) {
        self.rect = rect;
    }

    pub(super) fn scroll(&mut self, scroll: Vector2D<f32, PixelSize>, data: &Rope) {
        assert!(self.start_line < data.len_lines());

        self.off += scroll;

        // Scroll y
        while self.off.y < 0.0 && self.start_line > 0 {
            self.start_line -= 1;
            let metrics = self.line_metrics(&data.line(self.start_line));
            self.off.y += metrics.height;
        }
        if self.off.y < 0.0 {
            self.off.y = 0.0;
        }
        while self.off.y > 0.0 {
            let metrics = self.line_metrics(&data.line(self.start_line));
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

        // Scroll x
        if self.off.x <= 0.0 {
            self.off.x = 0.0;
        } else {
            let mut height = -self.off.y;
            let mut max_xoff = 0.0;
            for line in data.lines_at(self.start_line) {
                let metrics = self.line_metrics(&line);
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

    pub(super) fn draw(&mut self, data: &Rope, painter: &mut Painter) {
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

        for rope_line in data.lines_at(self.start_line) {
            if origin.y >= self.rect.origin.y + self.rect.size.height {
                break;
            }
            let mut ascender = space_metrics.ascender;
            let mut descender = space_metrics.descender;
            let gidx = Cell::new(0);

            let cursor = &self.cursor;
            let mut cursor_x = None;
            let current_x = Cell::new(origin.x);

            split_text(
                &rope_line,
                8,
                |n| {
                    gidx.set(gidx.get() + n);
                    current_x.set(current_x.get() + space_metrics.advance.width.to_f32());
                    let inner = &mut *spans.borrow_mut();
                    inner.push(SpanOrSpace::Space(n));
                },
                |text| {
                    let shaped = text_font.shape(text, text_size, TextStyle::default());
                    let mut gis = shaped.glyph_infos.iter().peekable();
                    for (j, _) in text.grapheme_indices(true) {
                        while gis.peek().unwrap().cluster < j as u32 {
                            let gi = gis.next().unwrap();
                            current_x.set(current_x.get() + gi.advance.width.to_f32());
                        }
                        if linum == cursor.line_num && gidx.get() == cursor.line_gidx {
                            cursor_x = Some(current_x.get());
                        }
                        gidx.set(gidx.get() + 1);
                    }
                    if shaped.ascender > ascender {
                        ascender = shaped.ascender;
                    }
                    if shaped.descender > descender {
                        descender = shaped.descender;
                    }
                    let inner = &mut *spans.borrow_mut();
                    inner.push(SpanOrSpace::Span(shaped));
                },
            );
            let height = ascender - descender;

            if let Some(x) = cursor_x {
                let rect: Rect<f32, PixelSize> =
                    Rect::new(point2(x, origin.y), size2(CUSROR_WIDTH, height.to_f32()));
                paint_ctx.color_quad(rect, Color::new(0x88, 0x44, 0x22, 0x88), false);
                //eprintln!("Rect: {:?}", rect);
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

    fn line_metrics(&self, line: &RopeSlice) -> LineMetrics {
        let mut text_font = self.bed_handle.text_font();
        let text_size = self.bed_handle.text_size();
        let text_style = TextStyle::default();
        let space_metrics = text_font.space_metrics(text_size, text_style);
        let state = RefCell::new((space_metrics.ascender, space_metrics.descender, 0.0));
        split_text(
            &line,
            8,
            |n| {
                let inner = &mut *state.borrow_mut();
                inner.2 += space_metrics.advance.width.to_f32() * n as f32;
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

#[derive(Default)]
struct ViewCursor {
    cidx: usize,
    line_num: usize,
    line_cidx: usize,
    line_gidx: usize,
    line_global_x: usize,
}
