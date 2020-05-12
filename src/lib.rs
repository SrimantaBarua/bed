// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;
use std::{thread, time};

use euclid::{size2, Size2D};
use glfw::{Action, Key, WindowEvent};

mod buffer;
mod common;
mod font;
mod opengl;
mod painter;
mod style;
mod text;
mod textview;
mod window;

use buffer::BufferViewCreateParams;
use common::PixelSize;

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
    window: window::Window,
}

impl Bed {
    pub fn run(args: clap::ArgMatches, size: Size2D<u32, PixelSize>) {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("failed to initialize GLFW");
        let (mut window, dpi, events) = window::Window::new(&mut glfw, size, "bed");
        let viewable_rect = window.viewable_rect();

        let painter = painter::Painter::new(size, viewable_rect, dpi);

        let mut font_core = font::FontCore::new().unwrap();
        let face_key = font_core.find("monospace").unwrap();
        let text_size = style::TextSize::from_f32(7.5);
        let text_shaper = Rc::new(RefCell::new(text::TextShaper::new(font_core)));

        let mut buffer_mgr = buffer::BufferMgr::new();
        let buf = match args.value_of("FILE") {
            Some(path) => buffer_mgr
                .from_file(&abspath(path))
                .expect("failed to open file"),
            _ => buffer_mgr.empty(),
        };

        let view_id = buffer_mgr.next_view_id();
        let view_params = BufferViewCreateParams {
            face_key,
            text_size,
            dpi,
            text_shaper,
            rect: viewable_rect,
        };
        let textview_tree = textview::TextTree::new(view_params, buf, view_id);

        window.show();

        let mut bed = Bed {
            window: window,
            painter: painter,
            buffer_mgr: buffer_mgr,
            textview_tree: textview_tree,
        };

        let mut start = time::Instant::now();
        let target = time::Duration::from_nanos(1_000_000_000 / 120);

        while !bed.window.should_close() {
            let mut redraw = false;

            for (_, event) in glfw::flush_messages(&events) {
                redraw = true;
                match event {
                    WindowEvent::FramebufferSize(w, h) => {
                        let viewable_rect = bed.window.viewable_rect();
                        bed.painter.resize(size2(w, h).cast(), viewable_rect);
                        bed.textview_tree.set_rect(viewable_rect);
                    }
                    WindowEvent::Key(Key::Up, _, Action::Press, _)
                    | WindowEvent::Key(Key::Up, _, Action::Repeat, _) => {
                        bed.move_cursor(Direction::Up)
                    }
                    WindowEvent::Key(Key::Down, _, Action::Press, _)
                    | WindowEvent::Key(Key::Down, _, Action::Repeat, _) => {
                        bed.move_cursor(Direction::Down)
                    }
                    WindowEvent::Key(Key::Left, _, Action::Press, _)
                    | WindowEvent::Key(Key::Left, _, Action::Repeat, _) => {
                        bed.move_cursor(Direction::Left)
                    }
                    WindowEvent::Key(Key::Right, _, Action::Press, _)
                    | WindowEvent::Key(Key::Right, _, Action::Repeat, _) => {
                        bed.move_cursor(Direction::Right)
                    }
                    WindowEvent::Key(Key::Enter, _, Action::Press, _)
                    | WindowEvent::Key(Key::Enter, _, Action::Repeat, _) => bed.insert_char('\n'),
                    WindowEvent::Key(Key::Tab, _, Action::Press, _)
                    | WindowEvent::Key(Key::Tab, _, Action::Repeat, _) => bed.insert_char('\t'),
                    WindowEvent::Char(c) => bed.insert_char(c),
                    WindowEvent::Key(Key::Backspace, _, Action::Press, _)
                    | WindowEvent::Key(Key::Backspace, _, Action::Repeat, _) => bed.delete_left(),
                    WindowEvent::Key(Key::Delete, _, Action::Press, _)
                    | WindowEvent::Key(Key::Delete, _, Action::Repeat, _) => bed.delete_right(),
                    _ => {}
                }
            }

            let diff = start.elapsed();
            start = time::Instant::now();

            if redraw {
                bed.painter.clear(style::Color::new(0, 0, 0, 0xff));
                bed.textview_tree.draw(&mut bed.painter);
                bed.window.swap_buffers();
            }

            if diff < target {
                thread::sleep(target - diff);
            }
            glfw.poll_events();
        }
    }

    fn insert_char(&mut self, c: char) {
        let textpane = self.textview_tree.active_mut();
        textpane.insert_char(c);
    }

    fn delete_left(&mut self) {
        let textpane = self.textview_tree.active_mut();
        textpane.delete_left();
    }

    fn delete_right(&mut self) {
        let textpane = self.textview_tree.active_mut();
        textpane.delete_right();
    }

    fn move_cursor(&mut self, dirn: Direction) {
        let textpane = self.textview_tree.active_mut();
        match dirn {
            Direction::Up => textpane.move_cursor_up(1),
            Direction::Down => textpane.move_cursor_down(1),
            Direction::Left => textpane.move_cursor_left(1),
            Direction::Right => textpane.move_cursor_right(1),
        }
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}
