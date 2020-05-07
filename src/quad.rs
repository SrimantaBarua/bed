// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::Rect;

use crate::common::PixelSize;
use crate::opengl::Element;
use crate::style::Color;

pub(crate) struct ColorQuad([f32; 24]);

impl Element for ColorQuad {
    fn num_points_per_vertex() -> usize {
        6
    }

    fn vertex_attributes() -> &'static [(i32, usize, usize)] {
        &[(2, 6, 0), (4, 6, 2)]
    }

    fn data(&self) -> &[f32] {
        &self.0
    }
}

impl ColorQuad {
    #[rustfmt::skip]
    pub(crate) fn new(
        rect: Rect<f32, PixelSize>,
        color: Color,
    ) -> ColorQuad {
        let (r, g, b, a) = color.to_opengl_color();
        let qbox = rect.to_box2d();
        ColorQuad([
            qbox.min.x, qbox.min.y, r, g, b, a,
            qbox.min.x, qbox.max.y, r, g, b, a,
            qbox.max.x, qbox.min.y, r, g, b, a,
            qbox.max.x, qbox.max.y, r, g, b, a,
        ])
    }
}
