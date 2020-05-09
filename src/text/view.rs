// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
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

impl TextView {
    fn new(buffer: Rc<RefCell<Buffer>>, id: BufferViewID) -> TextView {
        TextView { buffer, id }
    }
}

pub(crate) struct TextPane {
    rect: Rect<u32, PixelSize>,
    views: Vec<TextView>,
    active: usize,
}

impl TextPane {
    pub(super) fn new(buffer: Rc<RefCell<Buffer>>, view_id: BufferViewID) -> TextPane {
        let views = vec![TextView::new(buffer, view_id)];
        let rect = Rect::new(point2(0, 0), size2(0, 0));
        TextPane {
            views,
            active: 0,
            rect,
        }
    }

    pub(super) fn clone(&self, view_id: BufferViewID) -> TextPane {
        let views = vec![TextView::new(
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
        // TODO Set view rectangles
    }

    pub(super) fn draw(&self, painter: &mut Painter) {
        painter.rect(self.rect, Color::new(0xff, 0xff, 0xff, 0xff));
    }
}
