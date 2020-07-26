// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::Rect;

use crate::common::PixelSize;
use crate::style::Color;

mod mat4;
mod shader;
mod texture;
mod vert_array;

pub(crate) use mat4::Mat4;
pub(crate) use shader::{ActiveShaderProgram, ShaderProgram};
pub(crate) use texture::{GlTexture, TexRed, TexUnit};
pub(crate) use vert_array::{ElemArr, Element};

pub(crate) fn gl_init() {
    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
    }
}

// Set active viewport
pub(crate) fn gl_viewport(rect: Rect<i32, PixelSize>) {
    unsafe {
        gl::Viewport(
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
        )
    };
}

// Set clear color
pub(crate) fn gl_clear_color(color: Color) {
    let (r, g, b, a) = color.to_opengl_color();
    unsafe {
        gl::ClearColor(r, g, b, a);
    }
}

// Clear screen
pub(crate) fn gl_clear() {
    unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::STENCIL_BUFFER_BIT) };
}

// Enable/disable stencil tests
pub(super) fn gl_set_stencil_test(val: bool) {
    if val {
        unsafe {
            gl::Enable(gl::STENCIL_TEST);
            gl::StencilOp(gl::ZERO, gl::ZERO, gl::REPLACE);
        }
    } else {
        unsafe {
            gl::Disable(gl::STENCIL_TEST);
        }
    }
}

// Enable stencil modification
pub(super) fn gl_set_stencil_writing() {
    unsafe {
        gl::StencilFunc(gl::ALWAYS, 1, 0xff);
        gl::StencilMask(0xff);
    }
}

// Disable stencil modification, and read from stencil
pub(super) fn gl_set_stencil_reading() {
    unsafe {
        gl::StencilFunc(gl::EQUAL, 1, 0xff);
        gl::StencilMask(0x00);
    }
}

// Clear stencil buffer
pub(super) fn gl_clear_stencil() {
    unsafe {
        gl::StencilMask(0xff);
        gl::Clear(gl::STENCIL_BUFFER_BIT);
        gl::StencilMask(0x00);
    }
}
