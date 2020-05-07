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

    let mut font_core = font::FontCore::new().unwrap();
    let key = font_core.find("monospace").unwrap();
    let (buf, font) = font_core.get(key, style::TextStyle::default()).unwrap();
    font.shaper.set_scale(style::TextSize::from_f32(10.0), dpi);

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

        buf.clear_contents();
        buf.add_utf8("Hello, world!");
        buf.guess_segment_properties();

        let mut pos = point2(20, 30);
        for gi in font::harfbuzz::shape(&font.shaper, buf) {
            painter.glyph(
                pos,
                key,
                gi.gid,
                style::TextSize::from_f32(10.0),
                Color::new(0xff, 0xff, 0xff, 0xff),
                style::TextStyle::default(),
                &mut font.raster,
            );
            pos.x += gi.advance.width;
        }

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
