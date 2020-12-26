// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefMut;
use std::time::Duration;

use euclid::{vec2, Point2D, Rect, Vector2D};

use crate::buffer::{Buffer, BufferHandle, BufferViewId, CursorStyle};
use crate::common::PixelSize;
use crate::painter::Painter;
use crate::TARGET_DELTA;

struct ViewInner {
    buf_handle: BufferHandle,
    view_id: BufferViewId,
}

pub(crate) struct TextViewEditCtx<'a> {
    buffer: RefMut<'a, Buffer>,
    view_id: &'a BufferViewId,
}

impl<'a> TextViewEditCtx<'a> {
    pub(crate) fn move_cursor_up(&mut self, n: usize) {
        self.buffer.move_view_cursor_up(self.view_id, n);
    }

    pub(crate) fn move_cursor_down(&mut self, n: usize) {
        self.buffer.move_view_cursor_down(self.view_id, n);
    }

    pub(crate) fn move_cursor_left(&mut self, n: usize) {
        self.buffer.move_view_cursor_left(self.view_id, n);
    }

    pub(crate) fn move_cursor_right(&mut self, n: usize) {
        self.buffer.move_view_cursor_right(self.view_id, n);
    }

    pub(crate) fn move_cursor_to_line_start(&mut self, n: usize) {
        self.buffer.move_view_cursor_to_line_start(self.view_id, n);
    }

    pub(crate) fn move_cursor_to_line_end(&mut self, n: usize) {
        self.buffer.move_view_cursor_to_line_end(self.view_id, n);
    }

    pub(crate) fn move_cursor_to_line(&mut self, linum: usize) {
        self.buffer.move_view_cursor_to_line(self.view_id, linum);
    }

    pub(crate) fn move_cursor_to_last_line(&mut self) {
        self.buffer.move_view_cursor_to_last_line(self.view_id);
    }

    pub(crate) fn move_cursor_word(&mut self, n: usize) {
        self.buffer.move_view_cursor_word(self.view_id, n);
    }

    pub(crate) fn move_cursor_word_extended(&mut self, n: usize) {
        self.buffer.move_view_cursor_word_extended(self.view_id, n);
    }

    pub(crate) fn move_cursor_word_end(&mut self, n: usize) {
        self.buffer.move_view_cursor_word_end(self.view_id, n);
    }

    pub(crate) fn move_cursor_word_end_extended(&mut self, n: usize) {
        self.buffer
            .move_view_cursor_word_end_extended(self.view_id, n);
    }

    pub(crate) fn move_cursor_back(&mut self, n: usize) {
        self.buffer.move_view_cursor_back(self.view_id, n);
    }

    pub(crate) fn move_cursor_back_extended(&mut self, n: usize) {
        self.buffer.move_view_cursor_back_extended(self.view_id, n);
    }

    pub(crate) fn set_cursor_style(&mut self, style: CursorStyle) {
        self.buffer.set_view_cursor_style(self.view_id, style);
    }

    pub(crate) fn insert_char(&mut self, c: char) {
        self.buffer.insert_char(self.view_id, c);
    }

    pub(crate) fn delete_left(&mut self, n: usize) {
        self.buffer.delete_left(self.view_id, n);
    }

    pub(crate) fn delete_right(&mut self, n: usize) {
        self.buffer.delete_right(self.view_id, n);
    }

    pub(crate) fn snap_to_cursor(&mut self) {
        self.buffer.snap_to_cursor(self.view_id);
    }
}

pub(crate) struct TextView {
    scroll_vel: Vector2D<f32, PixelSize>,
    views: Vec<ViewInner>,
    active: usize,
}

impl TextView {
    pub(crate) fn scroll(&mut self, acc: Vector2D<f32, PixelSize>, duration: Duration) {
        // Make scrolling feel better
        let t = duration.as_secs_f32() / TARGET_DELTA.as_secs_f32();
        self.scroll_vel += acc * t;
        let dist = self.scroll_vel * 5.0;
        self.scroll_vel /= 4.0;
        // Round of scroll distance so that we do eventually stop
        let dist = dist.round();
        if dist.x != 0.0 || dist.y != 0.0 {
            let view = &mut self.views[self.active];
            view.buf_handle.scroll_view(&view.view_id, dist.cast());
        } else {
            self.scroll_vel = vec2(0.0, 0.0);
        }
    }

    pub(crate) fn edit_ctx(&mut self) -> TextViewEditCtx {
        let view = &mut self.views[self.active];
        let buffer = view.buf_handle.buffer();
        let view_id = &view.view_id;
        TextViewEditCtx { buffer, view_id }
    }

    pub(super) fn new(
        rect: Rect<u32, PixelSize>,
        mut buf_handle: BufferHandle,
        view_id: BufferViewId,
    ) -> TextView {
        buf_handle.new_view(&view_id, rect);
        let views = vec![ViewInner {
            buf_handle,
            view_id,
        }];
        TextView {
            scroll_vel: vec2(0.0, 0.0),
            views,
            active: 0,
        }
    }

    pub(super) fn set_rect(&mut self, rect: Rect<u32, PixelSize>) {
        let view = &mut self.views[self.active];
        view.buf_handle.set_view_rect(&view.view_id, rect);
    }

    pub(super) fn draw(&mut self, painter: &mut Painter) {
        let view = &mut self.views[self.active];
        view.buf_handle.draw_view(&view.view_id, painter);
    }

    pub(super) fn move_cursor_to_point(&mut self, point: Point2D<i32, PixelSize>) {
        let view = &mut self.views[self.active];
        view.buf_handle
            .move_view_cursor_to_point(&view.view_id, point);
    }
}
