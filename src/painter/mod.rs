// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ffi::CStr;
use std::ops::Drop;

use euclid::{point2, size2, Point2D, Rect, Size2D};

use crate::buffer::CursorStyle;
use crate::common::{PixelSize, DPI};
use crate::font::{FaceKey, RasterFace};
use crate::opengl::{
    gl_clear, gl_clear_color, gl_clear_stencil, gl_set_stencil_reading, gl_set_stencil_test,
    gl_set_stencil_writing, gl_viewport, ElemArr, Mat4, ShaderProgram,
};
use crate::style::{Color, TextSize, TextStyle};
use crate::text::{ShapedClusterIter, ShapedText, ShapedTextMetrics, TextAlignment, TextShaper};
use crate::{CURSOR_BLOCK_WIDTH, CURSOR_LINE_WIDTH};

mod glyphrender;
mod shapes;

use glyphrender::GlyphRenderer;
use shapes::{ColorQuad, ColorTriangle, TexColorQuad};

// Struct which handles drawing UI elements
pub(crate) struct Painter {
    projection: Mat4,
    glyph_render: GlyphRenderer,
    // Vertex buffers
    ct_arr: ElemArr<ColorTriangle>,
    cq_arr: ElemArr<ColorQuad>,
    tcq_arr: ElemArr<TexColorQuad>,
    // Shaders
    cq_shader: ShaderProgram,
    tcq_shader: ShaderProgram,
}

impl Painter {
    pub(crate) fn new(
        winsz: Size2D<u32, PixelSize>,
        view_rect: Rect<u32, PixelSize>,
        dpi: Size2D<u32, DPI>,
    ) -> Painter {
        let projection = Mat4::projection(winsz);
        let ct_arr = ElemArr::new(8);
        let cq_arr = ElemArr::new(8);
        let tcq_arr = ElemArr::new(128);
        let glyph_render = GlyphRenderer::new(dpi);
        // Compile shader(s)
        let cqvsrc = include_str!("shader_src/colored_quad.vert");
        let cqfsrc = include_str!("shader_src/colored_quad.frag");
        let cq_shader = ShaderProgram::new(cqvsrc, cqfsrc).unwrap();
        let tcqvsrc = include_str!("shader_src/tex_color_quad.vert");
        let tcqfsrc = include_str!("shader_src/tex_color_quad.frag");
        let tcq_shader = ShaderProgram::new(tcqvsrc, tcqfsrc).unwrap();
        // Return painter
        let mut ret = Painter {
            projection,
            glyph_render,
            ct_arr,
            cq_arr,
            tcq_arr,
            cq_shader,
            tcq_shader,
        };
        ret.resize(winsz, view_rect);
        ret
    }

    pub(crate) fn resize(&mut self, winsz: Size2D<u32, PixelSize>, view: Rect<u32, PixelSize>) {
        gl_viewport(view.cast());
        let projection = CStr::from_bytes_with_nul(b"projection\0").unwrap();
        self.projection = Mat4::projection(winsz);
        {
            let mut ash = self.cq_shader.use_program();
            ash.uniform_mat4f(projection, &self.projection);
        }
        {
            let mut ash = self.tcq_shader.use_program();
            ash.uniform_mat4f(projection, &self.projection);
        }
    }

    pub(crate) fn clear(&mut self, color: Color) {
        gl_clear_color(color);
        gl_clear();
    }

    pub(crate) fn widget_ctx(&mut self, rect: Rect<i32, PixelSize>, bgcol: Color) -> WidgetPainter {
        let mut ret = WidgetPainter {
            painter: self,
            rect,
            background_color: bgcol,
        };
        ret.draw_bg_stencil();
        ret
    }

    fn flush(&mut self) {
        {
            let ash = self.cq_shader.use_program();
            self.cq_arr.flush(&ash);
            self.ct_arr.flush(&ash);
        }
        {
            let ash = self.tcq_shader.use_program();
            self.tcq_arr.flush(&ash);
        }
    }
}

pub(crate) struct WidgetPainter<'a> {
    painter: &'a mut Painter,
    rect: Rect<i32, PixelSize>,
    background_color: Color,
}

impl<'a> WidgetPainter<'a> {
    pub(crate) fn color_quad(&mut self, rect: Rect<i32, PixelSize>, color: Color) {
        let tvec = self.rect.origin.to_vector();
        self.painter
            .cq_arr
            .push(ColorQuad::new(rect.translate(tvec).cast(), color));
    }

    pub(crate) fn color_triangle(&mut self, points: &[Point2D<i32, PixelSize>; 3], color: Color) {
        let tvec = self.rect.origin.to_vector();
        let points = [
            (points[0] + tvec).cast(),
            (points[1] + tvec).cast(),
            (points[2] + tvec).cast(),
        ];
        self.painter.ct_arr.push(ColorTriangle::new(&points, color));
    }

    pub(crate) fn glyph(
        &mut self,
        pos: Point2D<i32, PixelSize>,
        face: FaceKey,
        gid: u32,
        size: TextSize,
        color: Color,
        style: TextStyle,
        raster: &mut RasterFace,
    ) {
        let tvec = self.rect.origin.to_vector();
        self.painter.glyph_render.render_glyph(
            pos + tvec,
            face,
            gid,
            size,
            color,
            style,
            raster,
            &mut self.painter.tcq_arr,
        );
    }

    pub(crate) fn draw_shaped_text(
        &mut self,
        shaper: &mut TextShaper,
        pos: Point2D<i32, PixelSize>,
        line: &ShapedText,
        cursor: Option<(usize, Color, CursorStyle)>,
        width: u32,
    ) {
        let collected = line.styled_iter().collect::<Vec<_>>();
        let mut i = 0;

        // Draw left-aligned text
        while i < collected.len() && collected[i].6 == TextAlignment::Left {
            i += 1;
        }
        let (mut pos, mut gidx) = self.draw_shaped_text_inner(
            shaper,
            pos,
            &collected[..i],
            line.metrics,
            0,
            cursor,
            width,
        );

        // Draw right-aligned text
        if pos.x <= width as i32 && i < collected.len() {
            let space_remaining = width as i32 - pos.x;
            let mut rem_width = 0;
            for (clusters, _, _, _, _, _, _) in &collected[i..] {
                for clus in *clusters {
                    rem_width += clus.glyph_infos.iter().fold(0, |a, x| a + x.advance.width);
                }
            }
            if rem_width <= space_remaining {
                pos.x += space_remaining - rem_width;
            }
            let (pos_here, gidx_here) = self.draw_shaped_text_inner(
                shaper,
                pos,
                &collected[i..],
                line.metrics,
                gidx,
                cursor,
                width,
            );
            pos = pos_here;
            gidx = gidx_here;
        }

        if let Some((cgidx, ccolor, cstyle)) = cursor {
            if gidx == cgidx {
                let cwidth = match cstyle {
                    CursorStyle::Line => CURSOR_LINE_WIDTH,
                    _ => CURSOR_BLOCK_WIDTH,
                };
                let (cy, cheight) = match cstyle {
                    CursorStyle::Underline => (
                        pos.y - line.metrics.underline_position,
                        line.metrics.underline_thickness,
                    ),
                    _ => (
                        pos.y - line.metrics.ascender,
                        line.metrics.ascender - line.metrics.descender,
                    ),
                };
                self.color_quad(Rect::new(point2(pos.x, cy), size2(cwidth, cheight)), ccolor);
            }
        }
    }

    fn draw_shaped_text_inner<'b>(
        &mut self,
        shaper: &mut TextShaper,
        mut pos: Point2D<i32, PixelSize>,
        line: &'b [(
            ShapedClusterIter<'b>,
            FaceKey,
            TextStyle,
            TextSize,
            Color,
            Option<Color>,
            TextAlignment,
        )],
        metrics: ShapedTextMetrics,
        mut gidx: usize,
        cursor: Option<(usize, Color, CursorStyle)>,
        width: u32,
    ) -> (Point2D<i32, PixelSize>, usize) {
        for (clusters, face, style, size, color, opt_under, _) in line {
            let (clusters, face, style, size, color, opt_under) =
                (*clusters, *face, *style, *size, *color, *opt_under);
            for cluster in clusters {
                if pos.x >= width as i32 {
                    break;
                }
                let raster = shaper.get_raster(face, style).unwrap();
                let start_x = pos.x;
                for gi in cluster.glyph_infos {
                    if pos.x + gi.offset.width + gi.advance.width <= 0 {
                        pos.x += gi.advance.width;
                        continue;
                    }
                    self.glyph(pos + gi.offset, face, gi.gid, size, color, style, raster);
                    pos.x += gi.advance.width;
                }
                if pos.x <= 0 {
                    gidx += cluster.num_graphemes;
                    continue;
                }
                let width = pos.x - start_x;
                if let Some((cgidx, ccolor, cstyle)) = cursor {
                    if gidx <= cgidx && gidx + cluster.num_graphemes > cgidx {
                        let mut cx = (width * (cgidx - gidx) as i32) / cluster.num_graphemes as i32;
                        cx += start_x;
                        let cwidth = match cstyle {
                            CursorStyle::Line => CURSOR_LINE_WIDTH,
                            _ => width / cluster.num_graphemes as i32,
                        };
                        let (cy, cheight) = match cstyle {
                            CursorStyle::Underline => (
                                pos.y - metrics.underline_position,
                                metrics.underline_thickness,
                            ),
                            _ => (
                                pos.y - metrics.ascender,
                                metrics.ascender - metrics.descender,
                            ),
                        };
                        self.color_quad(Rect::new(point2(cx, cy), size2(cwidth, cheight)), ccolor);
                    }
                }
                if let Some(under) = opt_under {
                    self.color_quad(
                        Rect::new(
                            point2(start_x, pos.y - metrics.underline_position),
                            size2(width, metrics.underline_thickness),
                        ),
                        under,
                    );
                }
                gidx += cluster.num_graphemes;
            }
        }
        (pos, gidx)
    }

    fn draw_bg_stencil(&mut self) {
        // Activate stencil writing
        gl_set_stencil_test(true);
        gl_set_stencil_writing();
        // Draw background and write to stencil
        {
            let ash = self.painter.cq_shader.use_program();
            self.painter
                .cq_arr
                .push(ColorQuad::new(self.rect.cast(), self.background_color));
            self.painter.cq_arr.flush(&ash);
        }
        // Set stencil reading
        gl_set_stencil_reading();
    }
}

impl<'a> Drop for WidgetPainter<'a> {
    fn drop(&mut self) {
        self.painter.flush();
        gl_clear_stencil();
    }
}
