// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::{Rect, Size2D};

use crate::common::PixelSize;
use crate::opengl::{gl_clear, gl_clear_color, gl_viewport, Mat4};
use crate::style::Color;

// Struct which handles drawing UI elements
pub(crate) struct Painter {
    projection: Mat4, // Projection matrix
}

impl Painter {
    pub(crate) fn new(winsz: Size2D<u32, PixelSize>, view_rect: Rect<u32, PixelSize>) -> Painter {
        gl_viewport(view_rect.cast());
        let projection = Mat4::projection(winsz);
        Painter { projection }
    }

    pub(crate) fn resize(&mut self, winsz: Size2D<u32, PixelSize>, view: Rect<u32, PixelSize>) {
        gl_viewport(view.cast());
        self.projection = Mat4::projection(winsz);
    }

    pub(crate) fn clear(&mut self) {
        gl_clear_color(Color::new(0xff, 0xff, 0xff, 0xff));
        gl_clear();
    }
}
