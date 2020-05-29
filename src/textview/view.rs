// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::ops::Drop;
use std::rc::Rc;
use std::time::Duration;

use euclid::{vec2, Point2D, Rect, Vector2D};

use crate::buffer::{Buffer, BufferViewCreateParams, BufferViewID, CursorStyle};
use crate::common::PixelSize;
use crate::painter::Painter;

struct TextView {
    buffer: Rc<RefCell<Buffer>>,
    id: BufferViewID,
}

impl TextView {
    fn move_cursor_up(&mut self, n: usize) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.move_view_cursor_up(&self.id, n);
        }
    }

    fn move_cursor_down(&mut self, n: usize) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.move_view_cursor_down(&self.id, n);
        }
    }

    fn move_cursor_left(&mut self, n: usize) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.move_view_cursor_left(&self.id, n);
        }
    }

    fn move_cursor_right(&mut self, n: usize) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.move_view_cursor_right(&self.id, n);
        }
    }

    fn move_cursor_start_of_line(&mut self) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.move_view_cursor_start_of_line(&self.id);
        }
    }

    fn move_cursor_end_of_line(&mut self) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.move_view_cursor_end_of_line(&self.id);
        }
    }

    fn move_cursor_to_line(&mut self, linum: usize) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.move_view_cursor_to_line(&self.id, linum);
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

    fn insert_char(&mut self, c: char) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.view_insert_char(&self.id, c);
        }
    }

    fn delete_left(&mut self) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.view_delete_left(&self.id);
        }
    }

    fn delete_right(&mut self) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.view_delete_right(&self.id);
        }
    }

    fn delete_lines_down(&mut self, n: usize) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.view_delete_lines_down(&self.id, n);
        }
    }

    fn set_cursor_style(&mut self, style: CursorStyle) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.set_view_cursor_style(&self.id, style);
        }
    }

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

    fn set_rect(&mut self, rect: Rect<u32, PixelSize>) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.set_view_rect(&self.id, rect);
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
    pub(crate) fn move_cursor_up(&mut self, n: usize) {
        self.views[self.active].move_cursor_up(n);
    }

    pub(crate) fn move_cursor_down(&mut self, n: usize) {
        self.views[self.active].move_cursor_down(n);
    }

    pub(crate) fn move_cursor_left(&mut self, n: usize) {
        self.views[self.active].move_cursor_left(n);
    }

    pub(crate) fn move_cursor_right(&mut self, n: usize) {
        self.views[self.active].move_cursor_right(n);
    }

    pub(crate) fn move_cursor_start_of_line(&mut self) {
        self.views[self.active].move_cursor_start_of_line();
    }

    pub(crate) fn move_cursor_end_of_line(&mut self) {
        self.views[self.active].move_cursor_end_of_line();
    }

    pub(crate) fn move_cursor_to_line(&mut self, linum: usize) {
        self.views[self.active].move_cursor_to_line(linum);
    }

    pub(crate) fn move_cursor_to_point(&mut self, point: Point2D<u32, PixelSize>) {
        self.views[self.active].move_cursor_to_point(point);
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

    pub(crate) fn insert_char(&mut self, c: char) {
        self.views[self.active].insert_char(c);
    }

    pub(crate) fn delete_left(&mut self) {
        self.views[self.active].delete_left();
    }

    pub(crate) fn delete_right(&mut self) {
        self.views[self.active].delete_right();
    }

    pub(crate) fn delete_lines_down(&mut self, n: usize) {
        self.views[self.active].delete_lines_down(n);
    }

    pub(crate) fn set_cursor_style(&mut self, style: CursorStyle) {
        self.views[self.active].set_cursor_style(style);
    }

    pub(crate) fn rect(&self) -> Rect<u32, PixelSize> {
        self.params.rect
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
