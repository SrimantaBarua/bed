// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;

use euclid::Rect;
use ropey::Rope;

use crate::buffer::BufferBedHandle;
use crate::common::PixelSize;
use crate::style::{Color, TextStyle};
use crate::text::{split_text, ShapedSpan};

enum SpanOrSpace {
    Span(ShapedSpan),
    Space(usize),
}

pub(super) struct View {
    bed_handle: BufferBedHandle,
    rect: Rect<f32, PixelSize>,
    start_line: usize,
}

impl View {
    pub(super) fn new(bed_handle: BufferBedHandle, rect: Rect<f32, PixelSize>) -> View {
        View {
            bed_handle,
            rect,
            start_line: 0,
        }
    }

    pub(super) fn set_rect(&mut self, rect: Rect<f32, PixelSize>) {
        self.rect = rect;
    }

    pub(super) fn draw(&mut self, data: &Rope) {
        if self.start_line >= data.len_lines() {
            self.start_line = data.len_lines() - 1;
        }

        let mut text_font = self.bed_handle.text_font();
        let text_size = self.bed_handle.text_size();
        let text_style = TextStyle::default();
        let space_metrics = text_font.space_metrics(text_size, text_style);
        let mut origin = self.rect.origin;
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
}
