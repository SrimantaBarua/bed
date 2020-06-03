// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::ops::Drop;
use std::rc::Rc;
use std::time::Duration;

use euclid::{vec2, Point2D, Rect, Vector2D};

use crate::buffer::{Buffer, BufferID, BufferViewCreateParams, BufferViewID, CursorStyle};
use crate::common::PixelSize;
use crate::input::MotionOrObj;
use crate::painter::Painter;

struct TextView {
    buffer: Rc<RefCell<Buffer>>,
    id: BufferViewID,
}

impl TextView {
    fn new(
        view_params: BufferViewCreateParams,
        buffer: Rc<RefCell<Buffer>>,
        id: BufferViewID,
    ) -> TextView {
        {
            let buffer = &mut *buffer.borrow_mut();
            buffer.new_view(&id, view_params);
        }
        TextView { buffer, id }
    }

    fn move_cursor(&mut self, mo: MotionOrObj) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.move_view_cursor(&self.id, mo);
        }
    }

    fn insert_char(&mut self, c: char) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.view_insert_char(&self.id, c);
        }
    }

    fn delete(&mut self, mo: MotionOrObj) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.view_delete(&self.id, mo);
        }
    }

    fn move_cursor_to_point(&mut self, point: Point2D<u32, PixelSize>) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.move_view_cursor_to_point(&self.id, point);
        }
    }

    fn scroll(&mut self, vec: Vector2D<i32, PixelSize>) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.scroll_view(&self.id, vec);
        }
    }

    fn set_cursor_visible(&mut self, visible: bool) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.set_view_cursor_visible(&self.id, visible);
        }
    }

    fn set_cursor_style(&mut self, style: CursorStyle) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.set_view_cursor_style(&self.id, style);
        }
    }

    fn set_rect(&mut self, rect: Rect<u32, PixelSize>) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.set_view_rect(&self.id, rect);
        }
    }

    fn buffer_id(&self) -> BufferID {
        {
            let buffer = &*self.buffer.borrow();
            buffer.buffer_id()
        }
    }

    fn check_redraw(&mut self) -> bool {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.check_view_needs_redraw(&self.id)
        }
    }

    fn draw(&self, painter: &mut Painter) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.draw_view(&self.id, painter);
        }
    }
}

impl Drop for TextView {
    fn drop(&mut self) {
        let buffer = &mut *self.buffer.borrow_mut();
        buffer.remove_view(&self.id);
    }
}

pub(crate) struct TextPane {
    scroll_vel: Vector2D<f64, PixelSize>,
    params: BufferViewCreateParams,
    views: Vec<TextView>,
    active: usize,
}

impl TextPane {
    pub(crate) fn move_cursor(&mut self, mo: MotionOrObj) {
        self.views[self.active].move_cursor(mo);
    }

    pub(crate) fn move_cursor_to_point(&mut self, point: Point2D<u32, PixelSize>) {
        self.views[self.active].move_cursor_to_point(point);
    }

    pub(crate) fn insert_char(&mut self, c: char) {
        self.views[self.active].insert_char(c);
    }

    pub(crate) fn delete(&mut self, mo: MotionOrObj) {
        self.views[self.active].delete(mo);
    }

    pub(crate) fn check_redraw(&mut self) -> bool {
        self.views[self.active].check_redraw()
    }

    pub(crate) fn scroll(&mut self, mut acc: Vector2D<f64, PixelSize>, duration: Duration) -> bool {
        // Base is for 60fps, so calculate t as a scale on that
        let target = std::time::Duration::from_nanos(1_000_000_000 / 60);
        let t = duration.as_secs_f64() / target.as_secs_f64();
        // TODO: update logic if duration varies. Currently assuming a const 60fps
        acc.x *= acc.x.abs();
        acc.y *= acc.y.abs();
        let dist = ((self.scroll_vel * t) + (acc * 0.5 * t * t)) * 2.0;
        self.scroll_vel += acc * t;
        if acc.x == 0.0 {
            self.scroll_vel.x /= 2.0;
        }
        if acc.y == 0.0 {
            self.scroll_vel.y /= 2.0;
        }
        let dist = dist.cast();
        if dist.x != 0 || dist.y != 0 {
            self.views[self.active].scroll(dist);
            return true;
        }
        return false;
    }

    pub(crate) fn set_cursor_visible(&mut self, visible: bool) {
        self.views[self.active].set_cursor_visible(visible);
    }

    pub(crate) fn set_cursor_style(&mut self, style: CursorStyle) {
        self.views[self.active].set_cursor_style(style);
    }

    pub(crate) fn buffer_id(&self) -> BufferID {
        self.views[self.active].buffer_id()
    }

    pub(crate) fn rect(&self) -> Rect<u32, PixelSize> {
        self.params.rect
    }

    pub(crate) fn new_buffer<F>(&mut self, buf: Rc<RefCell<Buffer>>, mut f: F)
    where
        F: FnMut() -> BufferViewID,
    {
        for i in 0..self.views.len() {
            if Rc::ptr_eq(&self.views[i].buffer, &buf) {
                self.active = i;
                return;
            }
        }
        let view_id = f();
        let view = TextView::new(self.params.clone(), buf, view_id);
        self.active = self.views.len();
        self.views.push(view);
    }

    pub(crate) fn next_buffer(&mut self) {
        self.active = (self.active + 1) % self.views.len();
    }

    pub(crate) fn prev_buffer(&mut self) {
        if self.active == 0 {
            self.active = self.views.len() - 1;
        } else {
            self.active -= 1;
        }
    }

    pub(super) fn new(
        view_params: BufferViewCreateParams,
        buffer: Rc<RefCell<Buffer>>,
        view_id: BufferViewID,
    ) -> TextPane {
        let views = vec![TextView::new(view_params.clone(), buffer, view_id)];
        TextPane {
            views,
            scroll_vel: vec2(0.0, 0.0),
            active: 0,
            params: view_params,
        }
    }

    pub(super) fn clone(&self, view_id: BufferViewID) -> TextPane {
        let views = vec![TextView::new(
            self.params.clone(),
            self.views[self.active].buffer.clone(),
            view_id,
        )];
        TextPane {
            views,
            active: 0,
            scroll_vel: vec2(0.0, 0.0),
            params: self.params.clone(),
        }
    }

    pub(super) fn set_rect(&mut self, rect: Rect<u32, PixelSize>) {
        self.params.rect = rect;
        for v in &mut self.views {
            v.set_rect(rect);
        }
    }

    pub(super) fn draw(&self, painter: &mut Painter) {
        self.views[self.active].draw(painter);
    }
}
