// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use std::{thread, time};

extern crate crossbeam_channel;

use euclid::{size2, vec2, Rect, Size2D};
use glfw::{Action, MouseButtonLeft, WindowEvent};

#[macro_use]
mod log;

mod buffer;
mod cmdprompt;
mod commands;
mod common;
mod completion_popup;
mod config;
mod font;
mod input;
mod opengl;
mod painter;
mod project;
mod style;
mod text;
mod textview;
mod theme;
mod ts;
mod window;

use buffer::{BufferViewCreateParams, CursorStyle};
use common::{abspath, PixelSize};
use input::{Action as BedAction, MotionOrObj as BedMotionOrObj};

static CURSOR_LINE_WIDTH: i32 = 2;
static CURSOR_BLOCK_WIDTH: i32 = 10;
static DEFAULT_THEME: &str = "default";

pub struct Bed {
    textview_tree: textview::TextTree,
    painter: painter::Painter,
    input_state: input::State,
    buffer_mgr: buffer::BufferMgr,
    cmd_prompt: cmdprompt::CmdPrompt,
    window: window::Window,
    in_cmd_mode: bool,
}

impl Bed {
    pub fn run(args: clap::ArgMatches, size: Size2D<u32, PixelSize>) {
        let mut font_core = font::FontCore::new().unwrap();

        let config = Rc::new(config::Config::load(&mut font_core));
        let ts_core = ts::TsCore::new();
        let theme_set = theme::ThemeSet::load();
        let projects = project::Projects::load();

        let input_state = input::State::new();
        let mut input_actions = Vec::new();

        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("failed to initialize GLFW");
        let (mut window, dpi, events) = window::Window::new(&mut glfw, size, "bed");
        window.set_cursor(Some(glfw::Cursor::standard(glfw::StandardCursor::IBeam)));
        let viewable_rect = window.viewable_rect();

        let painter = painter::Painter::new(size, viewable_rect, dpi);
        let text_shaper = Rc::new(RefCell::new(text::TextShaper::new(font_core)));

        let theme = theme_set
            .0
            .get(&config.theme)
            .unwrap_or_else(|| theme_set.0.get(DEFAULT_THEME).unwrap())
            .clone();

        let mut buffer_mgr =
            buffer::BufferMgr::new(ts_core, projects, config.clone(), theme.clone());
        let buf = match args.value_of("FILE") {
            Some(path) => buffer_mgr
                .from_file(&abspath(path))
                .expect("failed to open file"),
            _ => buffer_mgr.empty(),
        };

        let cmd_prompt = cmdprompt::CmdPrompt::new(
            config.clone(),
            dpi,
            text_shaper.clone(),
            viewable_rect,
            theme.clone(),
        );

        let textview_rect = Rect::new(
            viewable_rect.origin,
            size2(
                viewable_rect.size.width,
                viewable_rect.size.height - cmd_prompt.rect.size.height,
            ),
        );

        let view_id = buffer_mgr.next_view_id();
        let view_params = BufferViewCreateParams {
            config: config.clone(),
            dpi,
            text_shaper,
            rect: textview_rect,
        };
        let textview_tree = textview::TextTree::new(view_params, buf, view_id, theme);

        window.show();

        let mut bed = Bed {
            window,
            painter,
            input_state,
            buffer_mgr,
            cmd_prompt,
            textview_tree,
            in_cmd_mode: false,
        };

        let mut start_time = time::Instant::now();
        let mut last_scroll_time = time::Instant::now();
        let mut last_blink_time = time::Instant::now();
        let mut cursor_blink_visible = true;

        let target_duration = time::Duration::from_nanos(1_000_000_000 / 60);
        let blink_duration = time::Duration::from_millis(500);

        bed.draw();

        while !bed.window.should_close() {
            let mut scroll_amt = (0.0, 0.0);
            let mut redraw = false;

            for (_, event) in glfw::flush_messages(&events) {
                input_actions.clear();
                redraw = true;

                match event {
                    WindowEvent::FramebufferSize(w, h) => {
                        let viewable_rect = bed.window.viewable_rect();
                        bed.painter.resize(size2(w, h).cast(), viewable_rect);
                        let textview_rect = bed.cmd_prompt.resize(viewable_rect);
                        bed.textview_tree.set_rect(textview_rect);
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
                        bed.textview_tree.active_mut().stop_completion();
                        bed.move_cursor_to_mouse();
                        bed.set_cursor_style(CursorStyle::Block);
                    }
                    WindowEvent::Scroll(xsc, ysc) => {
                        scroll_amt.0 += xsc;
                        scroll_amt.1 += ysc;
                    }
                    _ => {}
                }
            }

            let cur_time = time::Instant::now();
            redraw |= bed.scroll(scroll_amt, cur_time - last_scroll_time);
            last_scroll_time = cur_time;

            redraw |= bed.check_redraw();
            if redraw {
                last_blink_time = time::Instant::now();
                cursor_blink_visible = !bed.in_cmd_mode;
                bed.set_cursor_visible(cursor_blink_visible);
            } else if last_blink_time.elapsed() >= blink_duration {
                last_blink_time = time::Instant::now();
                cursor_blink_visible = !cursor_blink_visible & !bed.in_cmd_mode;
                bed.set_cursor_visible(cursor_blink_visible);
                redraw = true;
            }
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
            if self.in_cmd_mode {
                match action {
                    BedAction::GetCmd => {
                        let command = self.cmd_prompt.get_command();
                        self.handle_command(&command)
                    }
                    BedAction::StopCmdPrompt => {
                        self.cmd_prompt.clear();
                        self.in_cmd_mode = false;
                    }
                    _ => self.cmd_prompt.handle_action(action),
                }
            } else {
                match action {
                    BedAction::Move(mo) => self.move_cursor(*mo),
                    BedAction::InsertChar(c) => self.insert_char(*c),
                    BedAction::Delete(mo) => self.delete(*mo),
                    BedAction::UpdateCursorStyle(style) => self.set_cursor_style(*style),
                    BedAction::StartCmdPrompt(s) => {
                        self.cmd_prompt.set_prompt(s);
                        self.in_cmd_mode = true;
                    }
                    BedAction::GetCmd => unreachable!(),
                    BedAction::StopCmdPrompt => unreachable!(),
                }
            }
        }
    }

    fn check_redraw(&mut self) -> bool {
        self.textview_tree.map(|pane| pane.check_redraw())
    }

    fn draw(&mut self) {
        self.painter.clear(style::Color::new(0, 0, 0, 0xff));

        self.textview_tree.draw(&mut self.painter);
        self.cmd_prompt.draw(&mut self.painter);

        self.window.swap_buffers();
    }

    fn insert_char(&mut self, c: char) {
        self.textview_tree.active_mut().insert_char(c);
    }

    fn delete(&mut self, mo: BedMotionOrObj) {
        self.textview_tree.active_mut().delete(mo);
    }

    fn move_cursor(&mut self, mo: BedMotionOrObj) {
        self.textview_tree.active_mut().move_cursor(mo);
    }

    fn set_cursor_visible(&mut self, visible: bool) {
        self.textview_tree.active_mut().set_cursor_visible(visible)
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

    fn write_buffer(&mut self, optpath: Option<&str>) {
        let optpath = optpath.map(|path| abspath(path));
        let bufid = self.textview_tree.active().buffer_id();
        match self.buffer_mgr.write_buffer(bufid, optpath) {
            Some(Ok(nbytes)) => debug!("wrote {} bytes", nbytes),
            Some(Err(e)) => error!("error writing buffer: {}", e),
            None => debug!("buffer does not have path"),
        }
    }

    fn load_buffer(&mut self, optpath: Option<&str>) {
        let optpath = optpath.map(|path| abspath(path));
        let bufid = self.textview_tree.active().buffer_id();
        match self.buffer_mgr.load_buffer(bufid, optpath) {
            Some(Ok(buf)) => {
                let bufmgr = &mut self.buffer_mgr;
                self.textview_tree
                    .active_mut()
                    .new_buffer(buf, || bufmgr.next_view_id());
                debug!("loaded buffer");
            }
            Some(Err(e)) => error!("error loading buffer: {}", e),
            None => error!("buffer does not have path"),
        }
    }

    fn change_directory(&mut self, optpath: Option<&str>) {
        let path = optpath.unwrap_or("~");
        let abspath = abspath(path);
        if let Err(e) = env::set_current_dir(&abspath) {
            error!("failed to change directory to '{}': {}", path, e);
        }
    }

    fn horizontal_split(&mut self, optpath: Option<&str>) {
        let view_id = self.buffer_mgr.next_view_id();
        if let Some(path) = optpath {
            let abspath = abspath(path);
            match self.buffer_mgr.from_file(&abspath) {
                Ok(buf) => self.textview_tree.split_h(Some(buf), view_id),
                Err(e) => error!("erorr loading buffer: {}", e),
            }
        } else {
            self.textview_tree.split_h(None, view_id);
        }
    }

    fn vertical_split(&mut self, optpath: Option<&str>) {
        let view_id = self.buffer_mgr.next_view_id();
        if let Some(path) = optpath {
            let abspath = abspath(path);
            match self.buffer_mgr.from_file(&abspath) {
                Ok(buf) => self.textview_tree.split_v(Some(buf), view_id),
                Err(e) => error!("erorr loading buffer: {}", e),
            }
        } else {
            self.textview_tree.split_v(None, view_id);
        }
    }
}
