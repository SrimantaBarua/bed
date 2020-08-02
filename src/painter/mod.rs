// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ffi::CStr;

use euclid::{Rect, Size2D};

use crate::common::PixelSize;
use crate::opengl::{
    gl_clear, gl_clear_color, gl_clear_stencil, gl_set_stencil_reading, gl_set_stencil_test,
    gl_set_stencil_writing, ElemArr, Mat4, ShaderProgram,
};
use crate::shapes::{ColorQuad, RoundColorRect};
use crate::style::Color;

pub(crate) struct Painter {
    cq_shader: ShaderProgram,
    cq_arr: ElemArr<ColorQuad>,
    rcq_arr: ElemArr<RoundColorRect>,
}

impl Painter {
    pub(crate) fn new(window_size: Size2D<f32, PixelSize>) -> Painter {
        let projection = Mat4::projection(window_size);
        let cq_vert = include_str!("shader_src/color_quad.vert");
        let cq_frag = include_str!("shader_src/color_quad.frag");
        let mut cq_shader = ShaderProgram::new(cq_vert, cq_frag).unwrap();
        {
            let mut active = cq_shader.use_program();
            active.uniform_mat4f(
                CStr::from_bytes_with_nul(b"projection\0").unwrap(),
                &projection,
            );
        }
        Painter {
            cq_shader,
            cq_arr: ElemArr::new(8),
            rcq_arr: ElemArr::new(8),
        }
    }

    pub(crate) fn resize(&mut self, window_size: Size2D<f32, PixelSize>) {
        let projection = Mat4::projection(window_size);
        let mut active = self.cq_shader.use_program();
        active.uniform_mat4f(
            CStr::from_bytes_with_nul(b"projection\0").unwrap(),
            &projection,
        )
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
        let active = self.cq_shader.use_program();
        self.cq_arr.flush(&active);
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
