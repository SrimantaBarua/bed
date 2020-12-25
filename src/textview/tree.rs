// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::time::Duration;

use euclid::{Point2D, Rect, Vector2D};

use crate::buffer::{BufferHandle, BufferViewId};
use crate::common::PixelSize;
use crate::painter::Painter;

use super::view::TextView;

pub(crate) struct TextTree {
    border_width: u32,
    root: Node,
}

impl TextTree {
    pub(crate) fn new(
        rect: Rect<u32, PixelSize>,
        border_width: u32,
        buffer: BufferHandle,
        view_id: BufferViewId,
    ) -> TextTree {
        TextTree {
            border_width,
            root: Node::new_leaf(rect, buffer, view_id),
        }
    }

    pub(crate) fn set_rect(&mut self, rect: Rect<u32, PixelSize>) {
        self.root.set_rect(rect)
    }

    pub(crate) fn draw(&mut self, painter: &mut Painter) {
        self.root.draw(painter);
    }

    pub(crate) fn active_mut(&mut self) -> &mut TextView {
        self.root.active_mut()
    }

    pub(crate) fn move_cursor_to_point(&mut self, point: Point2D<i32, PixelSize>) {
        self.root.move_cursor_to_point(point);
    }

    pub(crate) fn scroll_views_with_active_acc(
        &mut self,
        acc: Vector2D<f32, PixelSize>,
        duration: Duration,
    ) {
        self.root.scroll_with_active_acc(acc, duration);
    }
}

struct Node {
    rect: Rect<u32, PixelSize>,
    view: TextView,
}

impl Node {
    fn new_leaf(rect: Rect<u32, PixelSize>, buffer: BufferHandle, view_id: BufferViewId) -> Node {
        Node {
            rect,
            view: TextView::new(rect, buffer, view_id),
        }
    }

    fn set_rect(&mut self, rect: Rect<u32, PixelSize>) {
        self.rect = rect;
        self.view.set_rect(rect)
    }

    fn draw(&mut self, painter: &mut Painter) {
        self.view.draw(painter)
    }

    fn active_mut(&mut self) -> &mut TextView {
        &mut self.view
    }

    fn move_cursor_to_point(&mut self, point: Point2D<i32, PixelSize>) {
        if !self.rect.cast().contains(point) {
            return;
        }
        self.view.move_cursor_to_point(point);
    }

    fn scroll_with_active_acc(&mut self, acc: Vector2D<f32, PixelSize>, duration: Duration) {
        self.view.scroll(acc, duration);
    }
}
