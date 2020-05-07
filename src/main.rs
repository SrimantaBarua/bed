// Second attempt at writing a text editor in Rust. This incorporates all my learning
// from the first attempt

// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::size2;

mod common;
mod opengl;
mod style;
mod window;

use opengl::{gl_clear, gl_clear_color, gl_viewport};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("failed to initialize GLFW");
    let (mut window, _, events) = window::Window::new(&mut glfw, size2(800, 600), "bed");
    gl_viewport(window.viewable_rect().cast());
    gl_clear_color(style::Color::parse("#ffffff").unwrap());
    while !window.should_close() {
        for _event in glfw::flush_messages(&events) {
            // TODO
        }
        gl_clear();
        window.swap_buffers();
        glfw.poll_events();
    }
}
