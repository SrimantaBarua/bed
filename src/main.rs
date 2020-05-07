// Second attempt at writing a text editor in Rust. This incorporates all my learning
// from the first attempt

// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::time;

use euclid::size2;

mod common;
mod window;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("failed to initialize GLFW");
    let (mut window, _, events) = window::Window::new(&mut glfw, size2(800, 600), "bed");
    let mut last_time = time::Instant::now();
    while !window.should_close() {
        for _event in glfw::flush_messages(&events) {
            // TODO
        }
        let cur_time = time::Instant::now();
        window.refresh(cur_time - last_time);
        last_time = cur_time;
        glfw.poll_events();
    }
}
