// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::ops::Drop;
use std::rc::Rc;

use euclid::{point2, size2, Rect};

use crate::buffer::{Buffer, BufferViewID};
use crate::common::PixelSize;
use crate::painter::Painter;
use crate::style::Color;

pub(crate) struct TextView {
    buffer: Rc<RefCell<Buffer>>,
    id: BufferViewID,
}

// TODO: Optimization: Track "active" views. i.e, the view that can actually be seen on a text
// pane

impl TextView {
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
        painter.rect(self.rect, Color::new(0xff, 0xff, 0xff, 0xff));
    }
}
