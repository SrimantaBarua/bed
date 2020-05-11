// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::ops::Drop;
use std::rc::Rc;

use euclid::Rect;

use crate::buffer::{Buffer, BufferViewID};
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

    fn new(rect: Rect<u32, PixelSize>, buffer: Rc<RefCell<Buffer>>, id: BufferViewID) -> TextView {
        {
            let buffer = &mut *buffer.borrow_mut();
            buffer.new_view(&id, rect);
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
    rect: Rect<u32, PixelSize>,
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

    pub(super) fn new(
        rect: Rect<u32, PixelSize>,
        buffer: Rc<RefCell<Buffer>>,
        view_id: BufferViewID,
    ) -> TextPane {
        let views = vec![TextView::new(rect, buffer, view_id)];
        TextPane {
            views,
            active: 0,
            rect,
        }
    }

    pub(super) fn clone(&self, view_id: BufferViewID) -> TextPane {
        let views = vec![TextView::new(
            self.rect,
            self.views[self.active].buffer.clone(),
            view_id,
        )];
        let rect = self.rect;
        TextPane {
            views,
            active: 0,
            rect,
        }
    }

    pub(super) fn set_rect(&mut self, rect: Rect<u32, PixelSize>) {
        self.rect = rect;
        for v in &mut self.views {
            v.set_rect(rect);
        }
    }

    pub(super) fn draw(&self, painter: &mut Painter) {
        let mut widget = painter.widget_ctx(self.rect.cast(), Color::new(0xff, 0xff, 0xff, 0xff));
        self.views[self.active].draw(&mut widget);
    }
}
