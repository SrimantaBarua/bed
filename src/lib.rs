// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;

use euclid::{size2, Size2D};
use glfw::WindowEvent;

use crate::common::PixelSize;

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

pub struct Bed {
    textview_tree: textview::TextTree,
    painter: painter::Painter,
    buffer_mgr: buffer::BufferMgr,
    events: std::sync::mpsc::Receiver<(f64, WindowEvent)>,
    window: window::Window,
    glfw: glfw::Glfw,
}

impl Bed {
    pub fn new(args: clap::ArgMatches, size: Size2D<u32, PixelSize>) -> Bed {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("failed to initialize GLFW");
        let (mut window, dpi, events) = window::Window::new(&mut glfw, size, "bed");
        let viewable_rect = window.viewable_rect();

        let painter = painter::Painter::new(size, viewable_rect, dpi);

        let mut font_core = font::FontCore::new().unwrap();
        let face_key = font_core.find("monospace").unwrap();
        let text_size = style::TextSize::from_f32(7.5);
        let text_shaper = Rc::new(RefCell::new(text::TextShaper::new(font_core)));

        let mut buffer_mgr = buffer::BufferMgr::new(text_shaper, face_key, text_size, dpi);
        let buf = match args.value_of("FILE") {
            Some(path) => buffer_mgr
                .from_file(&abspath(path))
                .expect("failed to open file"),
            _ => buffer_mgr.empty(),
        };

        let view_id = buffer_mgr.next_view_id();
        let textview_tree = textview::TextTree::new(viewable_rect, buf, view_id);

        Bed {
            glfw: glfw,
            events: events,
            window: window,
            painter: painter,
            buffer_mgr: buffer_mgr,
            textview_tree: textview_tree,
        }
    }

    pub fn run(mut self) {
        while !self.window.should_close() {
            for (_, event) in glfw::flush_messages(&self.events) {
                match event {
                    WindowEvent::FramebufferSize(w, h) => {
                        let viewable_rect = self.window.viewable_rect();
                        self.painter.resize(size2(w, h).cast(), viewable_rect);
                        self.textview_tree.set_rect(viewable_rect);
                    }
                    _ => {}
                }
            }
            self.painter.clear(style::Color::new(0, 0, 0, 0xff));
            self.textview_tree.draw(&mut self.painter);
            self.window.swap_buffers();
            self.glfw.poll_events();
        }
    }
}
