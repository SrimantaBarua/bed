// Second attempt at writing a text editor in Rust. This incorporates all my learning
// from the first attempt

// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::size2;
use glfw::WindowEvent;

mod common;
mod opengl;
mod painter;
mod style;
mod window;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("failed to initialize GLFW");
    let size = size2(800, 600);
    let (mut window, _, events) = window::Window::new(&mut glfw, size, "bed");
    let mut painter = painter::Painter::new(size, window.viewable_rect());
    while !window.should_close() {
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                WindowEvent::FramebufferSize(w, h) => {
                    painter.resize(size2(w, h).cast(), window.viewable_rect())
                }
                _ => {}
            }
        }
        painter.clear();
        window.swap_buffers();
        glfw.poll_events();
    }
}
