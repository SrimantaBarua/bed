// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefMut;
use std::io::Result as IOResult;
use std::time::Duration;

use euclid::{vec2, Point2D, Rect, Vector2D};

use crate::buffer::{Buffer, BufferHandle, BufferViewId, Mode};
use crate::common::PixelSize;
use crate::input::MoveObj;
use crate::painter::Painter;
use crate::text::CursorStyle;
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
    pub(crate) fn move_cursor(&mut self, move_obj: MoveObj) {
        self.buffer.move_cursor(self.view_id, move_obj);
    }

    pub(crate) fn delete(&mut self, move_obj: MoveObj) -> String {
        self.buffer.delete(self.view_id, move_obj)
    }

    pub(crate) fn insert(&mut self, s: &str) {
        self.buffer.insert(self.view_id, s);
    }

    pub(crate) fn half_page_down(&mut self) {
        self.buffer.half_page_down_view(self.view_id);
    }

    pub(crate) fn half_page_up(&mut self) {
        self.buffer.half_page_up_view(self.view_id);
    }

    pub(crate) fn page_down(&mut self) {
        self.buffer.page_down_view(self.view_id);
    }

    pub(crate) fn page_up(&mut self) {
        self.buffer.page_up_view(self.view_id);
    }

    pub(crate) fn set_cursor_style(&mut self, style: CursorStyle) {
        self.buffer.set_view_cursor_style(self.view_id, style);
    }

    pub(crate) fn replace_repeated(&mut self, c: char, n: usize) {
        self.buffer.replace_repeated(self.view_id, c, n);
    }

    pub(crate) fn snap_to_cursor(&mut self, update_global_x: bool) {
        self.buffer.snap_to_cursor(self.view_id, update_global_x);
    }

    pub(crate) fn update_text_size(&mut self, diff: i16) {
        self.buffer.update_text_size(self.view_id, diff);
    }

    pub(crate) fn set_buffer_mode(&mut self, mode: Mode) {
        self.buffer.set_mode(self.view_id, mode);
    }
}

pub(crate) struct TextView {
    scroll_vel: Vector2D<f32, PixelSize>,
    views: Vec<ViewInner>,
    active: usize,
    rect: Rect<u32, PixelSize>,
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

    pub(crate) fn buffer_handle(&self) -> BufferHandle {
        self.views[self.active].buf_handle.clone()
    }

    pub(crate) fn reload_buffer(&mut self) -> IOResult<()> {
        self.views[self.active].buf_handle.reload()
    }

    pub(crate) fn new_view(&mut self, mut buf_handle: BufferHandle, view_id: BufferViewId) {
        for i in 0..self.views.len() {
            if self.views[i].buf_handle == buf_handle {
                self.active = i;
                return;
            }
        }
        buf_handle.new_view(&view_id, self.rect);
        self.active = self.views.len();
        self.views.push(ViewInner {
            buf_handle,
            view_id,
        });
    }

    pub(crate) fn next_view(&mut self) {
        self.active = (self.active + 1) % self.views.len();
    }

    pub(crate) fn previous_view(&mut self) {
        self.active = (self.active + self.views.len() - 1) % self.views.len();
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
            rect,
        }
    }

    pub(super) fn set_rect(&mut self, rect: Rect<u32, PixelSize>) {
        for view in &mut self.views {
            view.buf_handle.set_view_rect(&view.view_id, rect);
        }
        self.rect = rect;
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
