// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::{Rect, Vector2D};

use crate::buffer::{BufferHandle, BufferViewId};
use crate::common::PixelSize;
use crate::painter::Painter;

struct ViewInner {
    buf_handle: BufferHandle,
    view_id: BufferViewId,
}

pub(crate) struct TextView {
    views: Vec<ViewInner>,
    active: usize,
}

impl TextView {
    pub(crate) fn scroll(&mut self, scroll: Vector2D<f32, PixelSize>) {
        let view = &mut self.views[self.active];
        view.buf_handle.scroll_view(&view.view_id, scroll);
    }

    pub(super) fn new(
        rect: Rect<f32, PixelSize>,
        mut buf_handle: BufferHandle,
        view_id: BufferViewId,
    ) -> TextView {
        buf_handle.new_view(&view_id, rect);
        let views = vec![ViewInner {
            buf_handle,
            view_id,
        }];
        TextView { views, active: 0 }
    }

    pub(super) fn set_rect(&mut self, rect: Rect<f32, PixelSize>) {
        let view = &mut self.views[self.active];
        view.buf_handle.set_view_rect(&view.view_id, rect);
    }

    pub(super) fn draw(&mut self, painter: &mut Painter) {
        let view = &mut self.views[self.active];
        view.buf_handle.draw_view(&view.view_id, painter);
    }
}
