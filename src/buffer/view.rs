// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;

use euclid::{vec2, Rect, Vector2D};
use ropey::{Rope, RopeSlice};

use crate::buffer::BufferBedHandle;
use crate::common::PixelSize;
use crate::style::{Color, TextStyle};
use crate::text::{f26_6, split_text, ShapedSpan};

enum SpanOrSpace {
    Span(ShapedSpan),
    Space(usize),
}

pub(super) struct View {
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

    pub(super) fn draw(&mut self, data: &Rope) {
        assert!(self.start_line < data.len_lines());

        let mut text_font = self.bed_handle.text_font();
        let text_size = self.bed_handle.text_size();
        let text_style = TextStyle::default();
        let space_metrics = text_font.space_metrics(text_size, text_style);
        let mut origin = self.rect.origin - self.off;
        let spans = RefCell::new(Vec::new());

        for rope_line in data.lines_at(self.start_line) {
            if origin.y >= self.rect.origin.y + self.rect.size.height {
                break;
            }
            let mut ascender = space_metrics.ascender;
            let mut descender = space_metrics.descender;
            split_text(
                &rope_line,
                8,
                |n| {
                    let inner = &mut *spans.borrow_mut();
                    inner.push(SpanOrSpace::Space(n));
                },
                |text| {
                    let shaped = text_font.shape(text, text_size, TextStyle::default());
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
