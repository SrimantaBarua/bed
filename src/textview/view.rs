// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::ops::Drop;
use std::rc::Rc;

use euclid::{Point2D, Rect};

use crate::buffer::{Buffer, BufferViewCreateParams, BufferViewID};
use crate::common::PixelSize;
use crate::painter::{Painter, WidgetPainter};
use crate::style::Color;

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

    fn move_cursor_to_point(&mut self, point: Point2D<u32, PixelSize>) {
        {
            let buffer = &mut *self.buffer.borrow_mut();
            buffer.move_view_cursor_to_point(&self.id, point);
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

    fn draw(&self, painter: &mut WidgetPainter) {
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

    pub(crate) fn move_cursor_to_point(&mut self, point: Point2D<u32, PixelSize>) {
        self.views[self.active].move_cursor_to_point(point);
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
        let mut widget =
            painter.widget_ctx(self.params.rect.cast(), Color::new(0xff, 0xff, 0xff, 0xff));
        self.views[self.active].draw(&mut widget);
    }
}
