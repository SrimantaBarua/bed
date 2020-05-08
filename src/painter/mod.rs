// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ffi::CStr;

use euclid::{Point2D, Rect, Size2D};

use crate::common::{PixelSize, DPI};
use crate::font::{FaceKey, RasterFace};
use crate::opengl::{gl_clear, gl_clear_color, gl_viewport, ElemArr, Mat4, ShaderProgram};
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
        gl_viewport(view_rect.cast());
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
        Painter {
            projection,
            glyph_render,
            cq_arr,
            tcq_arr,
            cq_shader,
            tcq_shader,
        }
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

    pub(crate) fn clear(&mut self) {
        gl_clear_color(Color::new(0, 0, 0, 0xff));
        gl_clear();
    }

    pub(crate) fn rect(&mut self, rect: Rect<u32, PixelSize>, color: Color) {
        self.cq_arr.push(ColorQuad::new(rect.cast(), color));
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
        self.glyph_render.render_glyph(
            pos,
            face,
            gid,
            size,
            color,
            style,
            raster,
            &mut self.tcq_arr,
        );
    }

    pub(crate) fn flush(&mut self) {
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
