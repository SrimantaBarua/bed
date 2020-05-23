// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ffi::CStr;
use std::ops::Drop;

use euclid::{Point2D, Rect, Size2D};

use crate::common::{PixelSize, DPI};
use crate::font::{FaceKey, RasterFace};
use crate::opengl::{
    gl_clear, gl_clear_color, gl_clear_stencil, gl_set_stencil_reading, gl_set_stencil_test,
    gl_set_stencil_writing, gl_viewport, ElemArr, Mat4, ShaderProgram,
};
use crate::style::{Color, TextSize, TextStyle};

mod glyphrender;
mod quad;

use glyphrender::GlyphRenderer;
use quad::{ColorQuad, TexColorQuad};

// Struct which handles drawing UI elements
pub(crate) struct Painter {
    projection: Mat4,
    glyph_render: GlyphRenderer,
    // Vertex buffers
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
            rect: rect,
            background_color: bgcol,
        };
        ret.draw_bg_stencil();
        ret
    }

    fn flush(&mut self) {
        {
            let ash = self.cq_shader.use_program();
            self.cq_arr.flush(&ash);
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
