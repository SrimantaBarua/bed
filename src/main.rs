// Second attempt at writing a text editor in Rust. This incorporates all my learning
// from the first attempt

// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::{point2, size2, Rect};
use glfw::WindowEvent;

mod common;
mod font;
mod opengl;
mod painter;
mod quad;
mod style;
mod window;

use style::Color;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("failed to initialize GLFW");
    let size = size2(800, 600);
    let (mut window, dpi, events) = window::Window::new(&mut glfw, size, "bed");
    let mut painter = painter::Painter::new(size, window.viewable_rect(), dpi);
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
        painter.rect(
            Rect::new(point2(10, 10), size2(200, 200)),
            Color::parse("#2288aa").unwrap(),
        );
        painter.rect(
            Rect::new(point2(100, 100), size2(200, 200)),
            Color::parse("#ff332288").unwrap(),
        );
        painter.flush();
        window.swap_buffers();
        glfw.poll_events();
    }
}
