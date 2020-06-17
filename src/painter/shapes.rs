// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::{Point2D, Rect};

use crate::common::{PixelSize, TextureSize};
use crate::opengl::Element;
use crate::style::Color;

pub(super) struct ColorTriangle([f32; 18]);

impl Element for ColorTriangle {
    fn num_vertices() -> usize {
        3
    }

    fn num_elements() -> usize {
        3
    }

    fn elements() -> &'static [u32] {
        &[0, 1, 2]
    }

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

impl ColorTriangle {
    #[rustfmt::skip]
    pub(crate) fn new(
        points: &[Point2D<f32, PixelSize>; 3],
        color: Color,
    ) -> ColorTriangle {
        let (r, g, b, a) = color.to_opengl_color();
        ColorTriangle([
            points[0].x, points[0].y, r, g, b, a,
            points[1].x, points[1].y, r, g, b, a,
            points[2].x, points[2].y, r, g, b, a,
        ])
    }
}

pub(crate) struct ColorQuad([f32; 24]);

impl Element for ColorQuad {
    fn num_vertices() -> usize {
        4
    }

    fn num_elements() -> usize {
        6
    }

    fn elements() -> &'static [u32] {
        &[0, 2, 1, 1, 2, 3]
    }

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

pub(crate) struct TexColorQuad {
    data: [f32; 32],
}

impl Element for TexColorQuad {
    fn num_vertices() -> usize {
        4
    }

    fn num_elements() -> usize {
        6
    }

    fn elements() -> &'static [u32] {
        &[0, 2, 1, 1, 2, 3]
    }

    fn num_points_per_vertex() -> usize {
        8
    }

    fn vertex_attributes() -> &'static [(i32, usize, usize)] {
        &[(4, 8, 0), (4, 8, 4)]
    }

    fn data(&self) -> &[f32] {
        &self.data
    }
}

impl TexColorQuad {
    #[rustfmt::skip]
    pub(crate) fn new(
        quad_rect: Rect<f32, PixelSize>,
        tex_rect: Rect<f32, TextureSize>,
        color: Color,
    ) -> TexColorQuad {
        let (r, g, b, a) = color.to_opengl_color();
        let qbox = quad_rect.to_box2d();
        let tbox = tex_rect.to_box2d();
        TexColorQuad {
            data: [
                qbox.min.x, qbox.min.y, tbox.min.x, tbox.min.y, r, g, b, a,
                qbox.min.x, qbox.max.y, tbox.min.x, tbox.max.y, r, g, b, a,
                qbox.max.x, qbox.min.y, tbox.max.x, tbox.min.y, r, g, b, a,
                qbox.max.x, qbox.max.y, tbox.max.x, tbox.max.y, r, g, b, a,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use euclid::{point2, size2};

    #[test]
    #[rustfmt::skip]
    fn test_colored_quad() {
        let quad = ColorQuad::new(
            Rect::new(point2(10.0, 10.0), size2(30.0, 30.0)),
            Color::new(0xff, 0xff, 0xff, 0xff),
        );
        assert_eq!(quad.data(), [
            10.0, 10.0, 1.0, 1.0, 1.0, 1.0,
            10.0, 40.0, 1.0, 1.0, 1.0, 1.0,
            40.0, 10.0, 1.0, 1.0, 1.0, 1.0,
            40.0, 40.0, 1.0, 1.0, 1.0, 1.0,
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_tex_color_quad() {
        let quad = TexColorQuad::new(
            Rect::new(point2(10.0, 10.0), size2(30.0, 30.0)),
            Rect::new(point2(0.25, 0.25), size2(0.5, 0.5)),
            Color::new(0xff, 0xff, 0xff, 0xff),
        );
        assert_eq!(quad.data(), [
            10.0, 10.0, 0.25, 0.25, 1.0, 1.0, 1.0, 1.0,
            10.0, 40.0, 0.25, 0.75, 1.0, 1.0, 1.0, 1.0,
            40.0, 10.0, 0.75, 0.25, 1.0, 1.0, 1.0, 1.0,
            40.0, 40.0, 0.75, 0.75, 1.0, 1.0, 1.0, 1.0,
        ]);
    }
}
