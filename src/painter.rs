// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ffi::CStr;

use euclid::{Rect, Size2D};

use crate::common::PixelSize;
use crate::opengl::{gl_clear, gl_clear_color, gl_viewport, ElemArr, Mat4, ShaderProgram};
use crate::quad::ColorQuad;
use crate::style::Color;

// Struct which handles drawing UI elements
pub(crate) struct Painter {
    projection: Mat4,
    color_quad_arr: ElemArr<ColorQuad>,
    color_quad_shader: ShaderProgram,
}

impl Painter {
    pub(crate) fn new(winsz: Size2D<u32, PixelSize>, view_rect: Rect<u32, PixelSize>) -> Painter {
        gl_viewport(view_rect.cast());
        let projection = Mat4::projection(winsz);
        let color_quad_arr = ElemArr::new(8);
        // Compile shader(s)
        let cqvsrc = include_str!("../shader_src/colored_quad.vert");
        let cqfsrc = include_str!("../shader_src/colored_quad.frag");
        let color_quad_shader = ShaderProgram::new(cqvsrc, cqfsrc).unwrap();
        // Return painter
        Painter {
            projection,
            color_quad_arr,
            color_quad_shader,
        }
    }

    pub(crate) fn resize(&mut self, winsz: Size2D<u32, PixelSize>, view: Rect<u32, PixelSize>) {
        gl_viewport(view.cast());
        let projection = CStr::from_bytes_with_nul(b"projection\0").unwrap();
        self.projection = Mat4::projection(winsz);
        {
            let mut ash = self.color_quad_shader.use_program();
            ash.uniform_mat4f(projection, &self.projection);
        }
    }

    pub(crate) fn clear(&mut self) {
        gl_clear_color(Color::new(0xff, 0xff, 0xff, 0xff));
        gl_clear();
    }

    pub(crate) fn rect(&mut self, rect: Rect<u32, PixelSize>, color: Color) {
        self.color_quad_arr.push(ColorQuad::new(rect.cast(), color));
    }

    pub(crate) fn flush(&mut self) {
        {
            let ash = self.color_quad_shader.use_program();
            self.color_quad_arr.flush(&ash);
        }
    }
}
