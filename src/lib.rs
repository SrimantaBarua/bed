// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;
use std::{thread, time};

extern crate crossbeam_channel;

use euclid::{size2, vec2, Size2D};
use glfw::{Action, MouseButtonLeft, WindowEvent};

mod buffer;
mod common;
mod config;
mod font;
mod input;
mod opengl;
mod painter;
mod style;
mod text;
mod textview;
mod theme;
mod ts;
mod window;

use buffer::{BufferViewCreateParams, CursorStyle};
use common::PixelSize;
use input::{Action as BedAction, Motion as BedMotion};

#[cfg(target_os = "linux")]
static DEFAULT_FONT: &'static str = "monospace";
#[cfg(target_os = "windows")]
static DEFAULT_FONT: &'static str = "Consolas";

static DEFAULT_THEME: &str = "default";
static DEFAULT_FONT_SIZE: f32 = 8.0;

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
    input_state: input::State,
    _buffer_mgr: buffer::BufferMgr,
    window: window::Window,
}

impl Bed {
    pub fn run(args: clap::ArgMatches, size: Size2D<u32, PixelSize>) {
        let config = config::Config::load();
        let ts_core = ts::TsCore::new();
        let theme_set = theme::ThemeSet::load();

        let input_state = input::State::new();
        let mut input_actions = Vec::new();

        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("failed to initialize GLFW");
        let (mut window, dpi, events) = window::Window::new(&mut glfw, size, "bed");
        window.set_cursor(Some(glfw::Cursor::standard(glfw::StandardCursor::IBeam)));
        let viewable_rect = window.viewable_rect();

        let painter = painter::Painter::new(size, viewable_rect, dpi);

        let mut font_core = font::FontCore::new().unwrap();
        let face_key = font_core
            .find(&config.font_family)
            .unwrap_or_else(|| font_core.find(&DEFAULT_FONT).expect("failed to find font"));
        let text_size = style::TextSize::from_f32(config.font_size);
        let text_shaper = Rc::new(RefCell::new(text::TextShaper::new(font_core)));

        let theme = theme_set
            .0
            .get(&config.theme)
            .unwrap_or_else(|| theme_set.0.get(DEFAULT_THEME).unwrap())
            .clone();

        let mut buffer_mgr = buffer::BufferMgr::new(ts_core, theme);
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
            input_state,
            _buffer_mgr: buffer_mgr,
            textview_tree: textview_tree,
        };

        let mut start_time = time::Instant::now();
        let mut last_scroll_time = time::Instant::now();
        let target_duration = time::Duration::from_nanos(1_000_000_000 / 60);

        bed.draw();

        while !bed.window.should_close() {
            let mut scroll_amt = (0.0, 0.0);

            for (_, event) in glfw::flush_messages(&events) {
                input_actions.clear();

                match event {
                    WindowEvent::FramebufferSize(w, h) => {
                        let viewable_rect = bed.window.viewable_rect();
                        bed.painter.resize(size2(w, h).cast(), viewable_rect);
                        bed.textview_tree.set_rect(viewable_rect);
                    }
                    WindowEvent::Key(k, _, Action::Press, md)
                    | WindowEvent::Key(k, _, Action::Repeat, md) => {
                        bed.input_state.handle_key(k, md, &mut input_actions);
                        bed.process_input_actions(&input_actions);
                    }
                    WindowEvent::Char(c) => {
                        bed.input_state.handle_char(c, &mut input_actions);
                        bed.process_input_actions(&input_actions);
                    }
                    WindowEvent::MouseButton(MouseButtonLeft, Action::Press, _) => {
                        bed.input_state.set_normal_mode();
                        bed.move_cursor_to_mouse()
                    }
                    WindowEvent::Scroll(xsc, ysc) => {
                        scroll_amt.0 += xsc;
                        scroll_amt.1 += ysc;
                    }
                    _ => {}
                }
            }

            let cur_time = time::Instant::now();
            let mut redraw = bed.scroll(scroll_amt, cur_time - last_scroll_time);
            last_scroll_time = cur_time;

            redraw |= bed.check_redraw();
            if redraw {
                bed.draw();
            }

            let tdiff = start_time.elapsed();
            if tdiff < target_duration {
                thread::sleep(target_duration - tdiff);
            }
            start_time = time::Instant::now();
            glfw.poll_events();
        }
    }

    fn process_input_actions(&mut self, actions: &[BedAction]) {
        for action in actions {
            match action {
                BedAction::Move(mov) => self.move_cursor(*mov),
                BedAction::InsertChar(c) => self.insert_char(*c),
                BedAction::Delete(mov) => self.delete(*mov),
                BedAction::UpdateCursorStyle(style) => self.set_cursor_style(*style),
            }
        }
    }

    fn check_redraw(&mut self) -> bool {
        self.textview_tree.map(|pane| pane.check_redraw())
    }

    fn draw(&mut self) {
        self.painter.clear(style::Color::new(0, 0, 0, 0xff));
        self.textview_tree.draw(&mut self.painter);
        self.window.swap_buffers();
    }

    fn insert_char(&mut self, c: char) {
        self.textview_tree.active_mut().insert_char(c);
    }

    fn delete(&mut self, mov: BedMotion) {
        self.textview_tree.active_mut().delete(mov);
    }

    fn move_cursor(&mut self, mov: BedMotion) {
        self.textview_tree.active_mut().move_cursor(mov);
    }

    fn set_cursor_style(&mut self, style: CursorStyle) {
        self.textview_tree.active_mut().set_cursor_style(style)
    }

    fn move_cursor_to_mouse(&mut self) {
        let pos = self.window.cursor_pos();
        if let Some(view) = self.textview_tree.set_active_and_get_from_pos(pos) {
            view.move_cursor_to_point(pos);
        }
    }

    fn scroll(&mut self, amt: (f64, f64), duration: time::Duration) -> bool {
        let pos = self.window.cursor_pos();
        let vec = vec2(amt.0, -amt.1);
        let redraw = self.textview_tree.map(|pane| {
            if pane.rect().contains(pos) {
                pane.scroll(vec, duration)
            } else {
                pane.scroll(vec2(0.0, 0.0), duration)
            }
        });
        redraw
    }
}
