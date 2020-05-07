// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::Rect;
use glfw::Window;

use crate::common::PixelSize;
use crate::style::Color;

// Initialize OpenGL
pub(crate) fn gl_init(window: &mut Window) {
    gl::load_with(|s| window.get_proc_address(s));
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
