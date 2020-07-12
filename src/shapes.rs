// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::{point2, Angle, Point2D, Rect, Rotation2D, SideOffsets2D, Vector2D};

use crate::common::{PixelSize, TextureSize};
use crate::opengl::Element;
use crate::style::Color;

/*
 * For rounded rects, I'm fixing the radius to 2 pixels. That way, we need 3 triangles per corner
 * 2 triangles in the center, and 2 triangles per edge. So 22 triangles. We'll be sharing some
 * verts. So - 2 verts per corner, 2 verts per edge, 4 verts for center. So finally - 20 verts
 */
const ROUND_RADIUS: f32 = 2.0;
const ROUND_NUM_VERTS: usize = 20;

pub(crate) struct RoundColorRect([f32; ROUND_NUM_VERTS * 6]);

impl Element for RoundColorRect {
    fn num_vertices() -> usize {
        ROUND_NUM_VERTS
    }

    fn num_elements() -> usize {
        48
    }

    #[rustfmt::skip]
    fn elements() -> &'static [u32] {
        &[
            // horizontal quad
            6, 4, 7, 7, 4, 5,
            // vertical quad
            8, 9, 10, 10, 9, 11,
            // top-right triangles
            2, 12, 4, 2, 13, 12, 2, 9, 13,
            // top-left triangles
            0, 14, 8, 0, 15, 14, 0, 6, 15,
            // bottom-left triangles
            1, 16, 7, 1, 17, 16, 1, 10, 17,
            // bottom-right triangles
            3, 18, 11, 3, 19, 18, 3, 5, 19,
        ]
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

impl RoundColorRect {
    #[rustfmt::skip]
    pub(crate) fn new(
        bound: Rect<f32, PixelSize>,
        color: Color,
    ) -> RoundColorRect {
        assert!(bound.size.width >= ROUND_RADIUS * 2.0 && bound.size.height >= ROUND_RADIUS * 2.0);
        let (r, g, b, a) = color.to_opengl_color();
        let side_offsets = SideOffsets2D::new_all_same(ROUND_RADIUS);
        let irect = bound.inner_rect(side_offsets);
        let ibox2d = irect.to_box2d();

        // Inner points
        let i_tl: Point2D<f32, PixelSize> = point2(ibox2d.min.x, ibox2d.min.y);
        let i_bl: Point2D<f32, PixelSize> = point2(ibox2d.min.x, ibox2d.max.y);
        let i_tr: Point2D<f32, PixelSize> = point2(ibox2d.max.x, ibox2d.min.y);
        let i_br: Point2D<f32, PixelSize> = point2(ibox2d.max.x, ibox2d.max.y);

        // Edge points
        let re_t: Point2D<f32, PixelSize> = point2(ibox2d.max.x + ROUND_RADIUS, ibox2d.min.y);
        let re_b: Point2D<f32, PixelSize> = point2(ibox2d.max.x + ROUND_RADIUS, ibox2d.max.y);
        let le_t: Point2D<f32, PixelSize> = point2(ibox2d.min.x - ROUND_RADIUS, ibox2d.min.y);
        let le_b: Point2D<f32, PixelSize> = point2(ibox2d.min.x - ROUND_RADIUS, ibox2d.max.y);
        let te_l: Point2D<f32, PixelSize> = point2(ibox2d.min.x, ibox2d.min.y - ROUND_RADIUS);
        let te_r: Point2D<f32, PixelSize> = point2(ibox2d.max.x, ibox2d.min.y - ROUND_RADIUS);
        let be_l: Point2D<f32, PixelSize> = point2(ibox2d.min.x, ibox2d.max.y + ROUND_RADIUS);
        let be_r: Point2D<f32, PixelSize> = point2(ibox2d.max.x, ibox2d.max.y + ROUND_RADIUS);

        let angle = Angle::frac_pi_3() / 2.0;
        let rotation = Rotation2D::new(angle);
        let mut vec = Vector2D::new(ROUND_RADIUS, 0.0);
        vec = rotation.transform_vector(vec);

        // Corner triangle points
        // Bottom-right
        let tr_br0 = i_br + vec;
        vec = rotation.transform_vector(vec);
        let tr_br1 = i_br + vec;
        vec = rotation.transform_vector(vec);
        vec = rotation.transform_vector(vec);

        // Bottom-left
        let tr_bl0 = i_bl + vec;
        vec = rotation.transform_vector(vec);
        let tr_bl1 = i_bl + vec;
        vec = rotation.transform_vector(vec);
        vec = rotation.transform_vector(vec);

        // Top-left
        let tr_tl0 = i_tl + vec;
        vec = rotation.transform_vector(vec);
        let tr_tl1 = i_tl + vec;
        vec = rotation.transform_vector(vec);
        vec = rotation.transform_vector(vec);

        // Top-right
        let tr_tr0 = i_tr + vec;
        vec = rotation.transform_vector(vec);
        let tr_tr1 = i_tr + vec;

        RoundColorRect([
            // Inner
            i_tl.x, i_tl.y, r, g, b, a,
            i_bl.x, i_bl.y, r, g, b, a,
            i_tr.x, i_tr.y, r, g, b, a,
            i_br.x, i_br.y, r, g, b, a,
            // Edge
            re_t.x, re_t.y, r, g, b, a,
            re_b.x, re_b.y, r, g, b, a,
            le_t.x, le_t.y, r, g, b, a,
            le_b.x, le_b.y, r, g, b, a,
            te_l.x, te_l.y, r, g, b, a,
            te_r.x, te_r.y, r, g, b, a,
            be_l.x, be_l.y, r, g, b, a,
            be_r.x, be_r.y, r, g, b, a,
            // Corners
            tr_tr0.x, tr_tr0.y, r, g, b, a,
            tr_tr1.x, tr_tr1.y, r, g, b, a,
            tr_tl0.x, tr_tl0.y, r, g, b, a,
            tr_tl1.x, tr_tl1.y, r, g, b, a,
            tr_bl0.x, tr_bl0.y, r, g, b, a,
            tr_bl1.x, tr_bl1.y, r, g, b, a,
            tr_br0.x, tr_br0.y, r, g, b, a,
            tr_br1.x, tr_br1.y, r, g, b, a,
        ])
    }
}

pub(crate) struct ColorTriangle([f32; 18]);

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

#[derive(Debug)]
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
