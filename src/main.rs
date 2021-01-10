// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;
use std::time;

use euclid::{point2, size2, Rect, Size2D};
use glutin::event::{Event, StartCause, WindowEvent};
use glutin::event_loop::ControlFlow;

mod buffer;
mod common;
mod config;
mod ds;
mod input;
mod language;
mod opengl;
mod painter;
mod prompt;
mod shapes;
mod style;
mod text;
mod textview;
mod theme;
mod ts;
mod window;

use common::PixelSize;

const TARGET_DELTA: time::Duration = time::Duration::from_nanos(1_000_000_000 / 60);
const DEFAULT_THEME: &str = "default";
const WINDOW_SIZE: Size2D<u32, PixelSize> = size2(1024, 768);

struct Bed {
    buffer_state: buffer::BufferBedHandle,
    config: Rc<RefCell<config::Config>>,
    theme_set: Rc<theme::ThemeSet>,
    buffer_mgr: buffer::BufferMgr,
    text_tree: textview::TextTree,
    prompt: prompt::Prompt,
    painter: painter::Painter,
    window: window::Window,
    font_core: text::FontCore,
    redraw_required: bool,
    ts_core: Rc<ts::TsCore>,
    quitting: bool,
}

impl Bed {
    fn new(args: &clap::ArgMatches) -> (Bed, glutin::event_loop::EventLoop<()>) {
        let window_size = WINDOW_SIZE;
        let (window, event_loop) = window::Window::new("bed", window_size);
        opengl::gl_init();
        let theme_set = Rc::new(theme::ThemeSet::load());
        let mut font_core = text::FontCore::new();
        let config = Rc::new(RefCell::new(config::Config::load(&mut font_core)));
        let painter = painter::Painter::new(window_size.cast());

        let ts_core = Rc::new(ts::TsCore::new());

        let window_rect = Rect::new(point2(0, 0), window.size());
        let prompt = prompt::Prompt::new(window_rect, config.clone(), theme_set.clone());

        let buffer_state = buffer::BufferBedHandle::new(config.clone(), theme_set.clone());
        let mut buffer_mgr = buffer::BufferMgr::new(buffer_state.clone(), ts_core.clone());
        let first_buffer = match args.value_of("FILE") {
            Some(path) => buffer_mgr.read_file(path).unwrap_or_else(|e| {
                eprintln!("failed to open file: {}: {}", path, e);
                buffer_mgr.empty_buffer()
            }),
            _ => buffer_mgr.empty_buffer(),
        };
        let first_view_id = buffer_mgr.next_view_id();

        let mut textview_size = window.size();
        if let Some(rect) = prompt.rect {
            textview_size.height -= rect.size.height;
        }
        let text_tree = textview::TextTree::new(
            Rect::new(point2(0, 0), textview_size),
            first_buffer,
            first_view_id,
            config.clone(),
            theme_set.clone(),
        );

        (
            Bed {
                painter,
                buffer_state,
                config,
                theme_set,
                buffer_mgr,
                text_tree,
                prompt,
                window,
                font_core,
                redraw_required: true,
                ts_core,
                quitting: false,
            },
            event_loop,
        )
    }
}

#[derive(Clone)]
struct BedHandle(Rc<RefCell<Bed>>);

impl BedHandle {
    fn new(bed: Bed) -> BedHandle {
        BedHandle(Rc::new(RefCell::new(bed)))
    }

    fn resize_window(&mut self, physical_size: glutin::dpi::PhysicalSize<u32>) {
        let mut inner = self.0.borrow_mut();
        inner.window.resize(physical_size);
        let window_size = inner.window.size();
        opengl::gl_viewport(Rect::new(point2(0, 0), window_size.cast()));
        inner.painter.resize(window_size.cast());
        let mut rect = Rect::new(point2(0, 0), window_size);
        inner.prompt.resize(rect);
        if let Some(cmd_rect) = inner.prompt.rect {
            rect.size.height -= cmd_rect.height();
        }
        inner.text_tree.set_rect(rect);
        inner.window.request_redraw();
    }

    fn check_redraw_required(&mut self) {
        let mut inner = self.0.borrow_mut();
        let mut required = inner.redraw_required;
        required |= inner.buffer_state.collect_redraw_state();
        required |= inner.prompt.needs_redraw;
        if required {
            inner.window.request_redraw();
        }
    }

    fn draw(&mut self) {
        let inner = &mut *self.0.borrow_mut();
        inner.painter.clear();
        inner.text_tree.draw(&mut inner.painter);
        inner.prompt.draw(&mut inner.painter);
        inner.window.swap_buffers();
        inner.redraw_required = false;
    }

    fn quitting(&self) -> bool {
        self.0.borrow().quitting
    }
}

fn main() {
    let args = parse_args();
    let (bed, event_loop) = Bed::new(&args);
    let mut bed = BedHandle::new(bed);
    let mut input_state = input::InputState::new(bed.clone());

    // random_text_here = "Hindi:उनका एक समय में बड़ा नाम था"

    let mut is_fps_boundary = true;
    let mut last_duration = time::Duration::from_secs(1);

    event_loop.run(move |event, _, control_flow| {
        //println!("event: {:?}", event);
        match event {
            Event::NewEvents(cause) => match cause {
                StartCause::ResumeTimeReached { start, .. } => {
                    let now = time::Instant::now();
                    *control_flow = ControlFlow::WaitUntil(now + TARGET_DELTA);
                    is_fps_boundary = true;
                    last_duration = now - start;
                }
                StartCause::WaitCancelled {
                    requested_resume, ..
                } => {
                    let req_res = requested_resume.expect("I dont' remember asking you to wait");
                    *control_flow = ControlFlow::WaitUntil(req_res);
                    is_fps_boundary = false;
                }
                StartCause::Init => {
                    *control_flow = ControlFlow::WaitUntil(time::Instant::now() + TARGET_DELTA);
                    is_fps_boundary = true;
                }
                _ => unreachable!(),
            },
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => bed.resize_window(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::ModifiersChanged(m) => input_state.update_modifiers(m),
                WindowEvent::MouseWheel { delta, .. } => input_state.add_scroll_amount(delta),
                WindowEvent::CursorMoved { position, .. } => {
                    input_state.handle_cursor_moved(position)
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    input_state.handle_mouse_input(button, state)
                }
                WindowEvent::ReceivedCharacter(c) => input_state.handle_char(c),
                WindowEvent::KeyboardInput { input, .. } => input_state.handle_keypress(input),
                _ => {}
            },
            Event::MainEventsCleared => {
                if is_fps_boundary {
                    input_state.flush_events(last_duration);
                    bed.check_redraw_required();
                }
            }
            Event::RedrawRequested(_) => bed.draw(),
            _ => {}
        };
        if bed.quitting() {
            *control_flow = ControlFlow::Exit;
        }
    });
}

fn parse_args() -> clap::ArgMatches<'static> {
    use clap::{App, Arg};
    App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("FILE")
                .help("file to open")
                .required(false)
                .index(1),
        )
        .get_matches()
}
