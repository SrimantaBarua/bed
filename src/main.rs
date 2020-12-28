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
mod input;
mod language;
mod opengl;
mod painter;
mod shapes;
mod style;
mod text;
mod textview;
mod theme;
mod window;

use common::PixelSize;

static TARGET_DELTA: time::Duration = time::Duration::from_nanos(1_000_000_000 / 60);
static DEFAULT_THEME: &str = "default";

struct Bed {
    buffer_state: buffer::BufferBedHandle,
    config: Rc<RefCell<config::Config>>,
    theme_set: Rc<theme::ThemeSet>,
    buffer_mgr: buffer::BufferMgr,
    text_tree: textview::TextTree,
    painter: painter::Painter,
    scale_factor: f64,
    window: window::Window,
    font_core: text::FontCore,
}

impl Bed {
    fn new() -> (Bed, glutin::event_loop::EventLoop<()>) {
        let window_size = size2(1024, 768);
        let (window, event_loop) = window::Window::new("bed", window_size);
        opengl::gl_init();
        let theme_set = Rc::new(theme::ThemeSet::load());
        let scale_factor = window.scale_factor();
        let mut font_core = text::FontCore::new();
        let config = Rc::new(RefCell::new(config::Config::load(
            &mut font_core,
            scale_factor,
        )));
        let painter = painter::Painter::new(window_size.cast());

        let buffer_state = buffer::BufferBedHandle::new(config.clone(), theme_set.clone());
        let mut buffer_mgr = buffer::BufferMgr::new(buffer_state.clone());

        let first_buffer = buffer_mgr.read_file("src/buffer/view.rs").unwrap();
        let first_view_id = buffer_mgr.next_view_id();
        let text_tree = textview::TextTree::new(
            Rect::new(point2(0, 0), window.size()),
            1,
            first_buffer,
            first_view_id,
        );

        (
            Bed {
                painter,
                buffer_state,
                config,
                theme_set,
                buffer_mgr,
                text_tree,
                window,
                scale_factor,
                font_core,
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
        inner
            .text_tree
            .set_rect(Rect::new(point2(0, 0), window_size));
        inner.window.request_redraw();
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        let mut inner = self.0.borrow_mut();
        let scale_amt = scale_factor / inner.scale_factor;
        {
            let mut config = inner.config.borrow_mut();
            config.completion_font_size = config.completion_font_size.scale(scale_amt);
            config.gutter_font_size = config.gutter_font_size.scale(scale_amt);
            config.hover_font_size = config.hover_font_size.scale(scale_amt);
            config.prompt_font_size = config.prompt_font_size.scale(scale_amt);
            config.textview_font_size = config.textview_font_size.scale(scale_amt);
        }
        inner.buffer_mgr.scale_text(scale_amt);
        inner.scale_factor = scale_factor;
        inner.window.request_redraw();
    }

    fn check_redraw_required(&mut self) {
        let mut required = false;
        let mut inner = self.0.borrow_mut();
        required |= inner.buffer_state.collect_redraw_state();
        if required {
            inner.window.request_redraw();
        }
    }

    fn draw(&mut self) {
        let inner = &mut *self.0.borrow_mut();
        inner.painter.clear();
        inner.text_tree.draw(&mut inner.painter);
        inner.window.swap_buffers();
    }
}

fn main() {
    let (bed, event_loop) = Bed::new();
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
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    // FIXME: Maybe track window size change?
                    bed.set_scale_factor(scale_factor)
                }
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
        }
    });
}
