// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ffi::CStr;

use euclid::{size2, Rect, Size2D};

use crate::common::{PixelSize, TextureSize};
use crate::opengl::{
    gl_clear, gl_clear_color, gl_clear_stencil, gl_set_stencil_reading, gl_set_stencil_test,
    gl_set_stencil_writing, ElemArr, GlTexture, Mat4, ShaderProgram, TexRed, TexUnit,
};
use crate::shapes::{ColorQuad, RoundColorRect, TexColorQuad};
use crate::style::Color;

const ATLAS_SIZE: Size2D<u32, PixelSize> = size2(4096, 4096);

pub(crate) struct Painter {
    cq_shader: ShaderProgram,
    cq_arr: ElemArr<ColorQuad>,
    rcq_arr: ElemArr<RoundColorRect>,
    tcq_shader: ShaderProgram,
    tcq_arr: ElemArr<TexColorQuad>,
    text_atlas: GlTexture<TexRed>,
}

impl Painter {
    pub(crate) fn new(window_size: Size2D<f32, PixelSize>) -> Painter {
        let projection = Mat4::projection(window_size);
        let cq_vert = include_str!("shader_src/color_quad.vert");
        let cq_frag = include_str!("shader_src/color_quad.frag");
        let mut cq_shader = ShaderProgram::new(cq_vert, cq_frag).unwrap();
        let proj_str = CStr::from_bytes_with_nul(b"projection\0").unwrap();
        {
            let mut active = cq_shader.use_program();
            active.uniform_mat4f(proj_str, &projection);
        }
        let tcq_vert = include_str!("shader_src/tex_color_quad.vert");
        let tcq_frag = include_str!("shader_src/tex_color_quad.frag");
        let mut tcq_shader = ShaderProgram::new(tcq_vert, tcq_frag).unwrap();
        {
            let mut active = tcq_shader.use_program();
            active.uniform_mat4f(proj_str, &projection);
        }
        Painter {
            cq_shader,
            cq_arr: ElemArr::new(8),
            rcq_arr: ElemArr::new(8),
            tcq_shader,
            tcq_arr: ElemArr::new(4096),
            text_atlas: GlTexture::new(TexUnit::Texture0, ATLAS_SIZE),
        }
    }

    pub(crate) fn resize(&mut self, window_size: Size2D<f32, PixelSize>) {
        let projection = Mat4::projection(window_size);
        let proj_str = CStr::from_bytes_with_nul(b"projection\0").unwrap();
        {
            let mut active = self.cq_shader.use_program();
            active.uniform_mat4f(proj_str, &projection)
        }
        {
            let mut active = self.tcq_shader.use_program();
            active.uniform_mat4f(proj_str, &projection)
        }
    }

    pub(crate) fn widget_ctx(
        &mut self,
        rect: Rect<f32, PixelSize>,
        bg_col: Color,
        rounded: bool,
    ) -> WidgetCtx {
        let mut ret = WidgetCtx {
            painter: self,
            rect,
            background_color: bg_col,
        };
        ret.draw_bg_stencil(rounded);
        ret
    }

    pub(crate) fn clear(&mut self) {
        gl_clear_color(Color::new(0xff, 0xff, 0xff, 0xff));
        gl_clear();
    }

    fn flush(&mut self) {
        {
            let active = self.cq_shader.use_program();
            self.rcq_arr.flush(&active);
            self.cq_arr.flush(&active);
        }
        {
            // FIXME: Activate texture?
            let active = self.tcq_shader.use_program();
            self.tcq_arr.flush(&active);
        }
    }
}

pub(crate) struct WidgetCtx<'a> {
    painter: &'a mut Painter,
    rect: Rect<f32, PixelSize>,
    background_color: Color,
}

impl<'a> WidgetCtx<'a> {
    pub(crate) fn color_quad(&mut self, rect: Rect<f32, PixelSize>, color: Color, rounded: bool) {
        let tvec = self.rect.origin.to_vector();
        if rounded {
            let rcr = RoundColorRect::new(rect.translate(tvec), color);
            self.painter.rcq_arr.push(rcr);
        } else {
            let cq = ColorQuad::new(rect.translate(tvec), color);
            self.painter.cq_arr.push(cq);
        }
    }

    pub(crate) fn texture_color_quad(
        &mut self,
        rect: Rect<f32, PixelSize>,
        texture_rect: Rect<f32, TextureSize>,
        color: Color,
    ) {
        let tvec = self.rect.origin.to_vector();
        let tex_quad = TexColorQuad::new(rect.translate(tvec), texture_rect, color);
        self.painter.tcq_arr.push(tex_quad);
    }

    pub(crate) fn text_atlas(&mut self) -> &mut GlTexture<TexRed> {
        &mut self.painter.text_atlas
    }

    fn draw_bg_stencil(&mut self, rounded: bool) {
        // Activate stencil writing
        gl_set_stencil_test(true);
        gl_set_stencil_writing();
        // Draw background and write to stencil
        let ash = self.painter.cq_shader.use_program();
        if rounded {
            let rcr = RoundColorRect::new(self.rect.cast(), self.background_color);
            self.painter.rcq_arr.push(rcr);
            self.painter.rcq_arr.flush(&ash);
        } else {
            let cq = ColorQuad::new(self.rect.cast(), self.background_color);
            self.painter.cq_arr.push(cq);
            self.painter.cq_arr.flush(&ash);
        }
        // Set stencil reading
        gl_set_stencil_reading();
    }
}

impl<'a> Drop for WidgetCtx<'a> {
    fn drop(&mut self) {
        self.painter.flush();
        gl_clear_stencil();
    }
}
