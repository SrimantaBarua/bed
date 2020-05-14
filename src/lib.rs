// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;
use std::{thread, time};

use euclid::{size2, vec2, Size2D};
use glfw::{Action, Key, MouseButtonLeft, WindowEvent};
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

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
    _buffer_mgr: buffer::BufferMgr,
    window: window::Window,
    _syntax_set: Rc<SyntaxSet>,
    _theme_set: Rc<ThemeSet>,
}

impl Bed {
    pub fn run(args: clap::ArgMatches, size: Size2D<u32, PixelSize>) {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("failed to initialize GLFW");
        let (mut window, dpi, events) = window::Window::new(&mut glfw, size, "bed");
        window.set_cursor(Some(glfw::Cursor::standard(glfw::StandardCursor::IBeam)));
        let viewable_rect = window.viewable_rect();

        let painter = painter::Painter::new(size, viewable_rect, dpi);

        let mut font_core = font::FontCore::new().unwrap();
        let face_key = font_core.find("monospace").unwrap();
        let text_size = style::TextSize::from_f32(7.5);
        let text_shaper = Rc::new(RefCell::new(text::TextShaper::new(font_core)));

        let syntax_set = Rc::new(SyntaxSet::load_defaults_newlines());
        let theme_set = Rc::new(ThemeSet::load_defaults());

        let mut buffer_mgr =
            buffer::BufferMgr::new(syntax_set.clone(), theme_set.clone(), "base16-ocean.dark");
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
            _buffer_mgr: buffer_mgr,
            textview_tree: textview_tree,
            _syntax_set: syntax_set,
            _theme_set: theme_set,
        };

        let mut start = time::Instant::now();
        let target = time::Duration::from_nanos(1_000_000_000 / 60);

        bed.draw();

        while !bed.window.should_close() {
            let mut redraw = false;
            let mut scroll_amt = (0.0, 0.0);

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
                    WindowEvent::MouseButton(MouseButtonLeft, Action::Press, _) => {
                        bed.move_cursor_to_mouse()
                    }
                    WindowEvent::Scroll(xsc, ysc) => {
                        scroll_amt.0 += xsc;
                        scroll_amt.1 += ysc;
                    }
                    _ => {}
                }
            }
            redraw |= bed.scroll(scroll_amt, target);

            let diff = start.elapsed();
            start = time::Instant::now();
            if redraw {
                bed.draw();
            }

            if diff < target {
                thread::sleep(target - diff);
            }
            glfw.poll_events();
        }
    }

    fn draw(&mut self) {
        self.painter.clear(style::Color::new(0, 0, 0, 0xff));
        self.textview_tree.draw(&mut self.painter);
        self.window.swap_buffers();
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

    fn move_cursor_to_mouse(&mut self) {
        let pos = self.window.cursor_pos();
        if let Some(view) = self.textview_tree.set_active_and_get_from_pos(pos) {
            view.move_cursor_to_point(pos);
        }
    }

    fn scroll(&mut self, amt: (f64, f64), duration: time::Duration) -> bool {
        let pos = self.window.cursor_pos();
        let vec = vec2(amt.0, -amt.1).cast();
        let redraw = self.textview_tree.map(|pane| {
            if pane.rect().contains(pos) {
                pane.scroll(vec, duration)
            } else {
                pane.scroll(vec2(0, 0), duration)
            }
        });
        redraw
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}
