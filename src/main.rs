// Second attempt at writing a text editor in Rust. This incorporates all my learning
// from the first attempt

// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;

use euclid::size2;
use glfw::WindowEvent;

mod buffer;
mod common;
mod font;
mod opengl;
mod painter;
mod style;
mod text;
mod textview;
mod window;

fn abspath(spath: &str) -> String {
    use std::env;
    use std::path::Path;

    let path = Path::new(spath);
    if path.is_absolute() {
        spath.to_owned()
    } else {
        let mut wdir = env::current_dir().expect("failed to get current directory");
        wdir.push(spath);
        wdir.to_str()
            .expect("failed to convert path to string")
            .to_owned()
    }
}

fn main() {
    let args = parse_args();

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("failed to initialize GLFW");
    let size = size2(800, 600);
    let (mut window, dpi, events) = window::Window::new(&mut glfw, size, "bed");
    let viewable_rect = window.viewable_rect();
    let mut painter = painter::Painter::new(size, viewable_rect, dpi);
    let mut font_core = font::FontCore::new().unwrap();
    let face_key = font_core.find("monospace").unwrap();
    let text_size = style::TextSize::from_f32(8.0);
    let text_shaper = Rc::new(RefCell::new(text::TextShaper::new(font_core)));
    let mut buffer_mgr = buffer::BufferMgr::new(text_shaper, face_key, text_size, dpi);
    let buf = match args.value_of("FILE") {
        Some(path) => buffer_mgr
            .from_file(&abspath(path))
            .expect("failed to open file"),
        _ => buffer_mgr.empty(),
    };
    let view_id = buffer_mgr.next_view_id();

    let mut textview_tree = textview::TextTree::new(viewable_rect, buf, view_id);
    textview_tree.split_h(buffer_mgr.next_view_id());
    textview_tree.split_v(buffer_mgr.next_view_id());

    while !window.should_close() {
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                WindowEvent::FramebufferSize(w, h) => {
                    let viewable_rect = window.viewable_rect();
                    painter.resize(size2(w, h).cast(), viewable_rect);
                    textview_tree.set_rect(viewable_rect);
                }
                _ => {}
            }
        }
        painter.clear();
        textview_tree.draw(&mut painter);
        painter.flush();
        window.swap_buffers();
        glfw.poll_events();
    }
}

fn parse_args() -> clap::ArgMatches<'static> {
    use clap::{App, Arg};
    App::new("bed")
        .version("0.0.1")
        .author("Srimanta Barua <srimanta.barua1@gmail.com>")
        .about("Barua's editor")
        .arg(
            Arg::with_name("FILE")
                .help("file to open")
                .required(false)
                .index(1),
        )
        .get_matches()
}
