// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::Rect;

use crate::buffer::{BufferHandle, BufferViewId};
use crate::common::PixelSize;
use crate::painter::Painter;

use super::view::TextView;

pub(crate) struct TextTree {
    border_width: f32,
    root: Node,
}

impl TextTree {
    pub(crate) fn new(
        rect: Rect<f32, PixelSize>,
        border_width: f32,
        buffer: BufferHandle,
        view_id: BufferViewId,
    ) -> TextTree {
        TextTree {
            border_width,
            root: Node::new_leaf(rect, buffer, view_id),
        }
    }

    pub(crate) fn set_rect(&mut self, rect: Rect<f32, PixelSize>) {
        self.root.set_rect(rect)
    }

    pub(crate) fn draw(&mut self, painter: &mut Painter) {
        self.root.draw(painter);
    }

    pub(crate) fn active_mut(&mut self) -> &mut TextView {
        self.root.active_mut()
    }
}

struct Node {
    view: TextView,
}

impl Node {
    fn new_leaf(rect: Rect<f32, PixelSize>, buffer: BufferHandle, view_id: BufferViewId) -> Node {
        Node {
            view: TextView::new(rect, buffer, view_id),
        }
    }

    fn set_rect(&mut self, rect: Rect<f32, PixelSize>) {
        self.view.set_rect(rect)
    }

    fn draw(&mut self, painter: &mut Painter) {
        self.view.draw(painter)
    }

    fn active_mut(&mut self) -> &mut TextView {
        &mut self.view
    }
}
