// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;

use euclid::{point2, size2, Rect, Size2D};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;

mod buffer;
mod common;
mod opengl;
mod shapes;
mod style;
mod text;
mod textview;
mod window;

use common::PixelSize;

struct Bed {
    font_core: text::FontCore,
    text_font: text::FontCollectionHandle,
    text_size: style::TextSize,
    window: window::Window,
    scale_factor: f64,
}

impl Bed {
    fn new() -> (Bed, glutin::event_loop::EventLoop<()>) {
        let window_size = size2(1024, 768);
        let (window, event_loop) = window::Window::new("bed", window_size);
        opengl::gl_init();
        let scale_factor = window.scale_factor();
        let mut font_core = text::FontCore::new(window_size.cast());

        let text_font = font_core.find("monospace").expect("Failed to find font");
        let text_size = style::TextSize(14);

        (
            Bed {
                font_core,
                window,
                scale_factor,
                // Config
                text_font,
                text_size,
            },
            event_loop,
        )
    }
}

struct BedHandle(Rc<RefCell<Bed>>);

impl BedHandle {
    fn new(bed: Bed) -> BedHandle {
        BedHandle(Rc::new(RefCell::new(bed)))
    }

    fn resize_window(&mut self, physical_size: glutin::dpi::PhysicalSize<u32>) {
        let inner = &mut *self.0.borrow_mut();
        inner.window.resize(physical_size);
        inner.font_core.set_window_size(inner.window.size());
        inner.window.request_redraw();
    }

    fn window_size(&self) -> Size2D<f32, PixelSize> {
        let inner = &*self.0.borrow();
        inner.window.size()
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        let inner = &mut *self.0.borrow_mut();
        inner.text_size = inner.text_size.scale(scale_factor / inner.scale_factor);
        inner.scale_factor = scale_factor;
        inner.window.request_redraw();
    }

    fn swap_buffers(&mut self) {
        let inner = &mut *self.0.borrow_mut();
        inner.window.swap_buffers();
    }
}

fn main() {
    let (bed, event_loop) = Bed::new();
    let mut bed = BedHandle::new(bed);
    let mut buffer_mgr = buffer::BufferMgr::new(buffer::BufferBedHandle::new(&bed));

    let buffer = buffer_mgr.read_file("src/main.rs").unwrap();
    let view_id = buffer_mgr.next_view_id();
    let mut text_tree = textview::TextTree::new(
        Rect::new(point2(0.0, 0.0), bed.window_size()),
        1.0,
        buffer,
        view_id,
    );

    event_loop.run(move |event, _, control_flow| {
        println!("event: {:?}", event);
        *control_flow = ControlFlow::Wait;
        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    bed.resize_window(physical_size);
                    let window_size = bed.window_size();
                    opengl::gl_viewport(Rect::new(point2(0, 0), window_size.cast()));
                    text_tree.set_rect(Rect::new(point2(0.0, 0.0), bed.window_size()));
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    bed.set_scale_factor(scale_factor);
                }
                _ => {}
            },
            Event::MainEventsCleared => {}
            Event::RedrawRequested(_) => {
                opengl::gl_clear_color(style::Color::new(0xff, 0xff, 0xff, 0xff));
                opengl::gl_clear();
                text_tree.draw();
                bed.swap_buffers();
            }
            _ => {}
        }
    });
}
