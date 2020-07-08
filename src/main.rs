// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::{point2, size2, Rect};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;

mod common;
mod opengl;
mod style;
mod text;
mod window;

struct Bed {
    font_core: text::FontCore,
    window: window::Window,
    scale_factor: f64,
}

impl Bed {
    fn new() -> (Bed, glutin::event_loop::EventLoop<()>) {
        let font_core = text::FontCore::new();
        let (window, event_loop) = window::Window::new("bed", size2(1024, 768));
        let scale_factor = window.scale_factor();
        opengl::gl_init();
        (
            Bed {
                font_core,
                window,
                scale_factor,
            },
            event_loop,
        )
    }
}

fn main() {
    let (mut bed, event_loop) = Bed::new();

    let font = bed.font_core.find("monospace");

    event_loop.run(move |event, _, control_flow| {
        println!("event: {:?}", event);
        *control_flow = ControlFlow::Wait;
        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    bed.window.resize(physical_size);
                    opengl::gl_viewport(Rect::new(
                        point2(0, 0),
                        size2(physical_size.width, physical_size.height).cast(),
                    ));
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    bed.scale_factor = scale_factor;
                }
                _ => {}
            },
            Event::MainEventsCleared => {}
            Event::RedrawRequested(_) => {
                opengl::gl_clear_color(style::Color::new(0xff, 0xff, 0xff, 0xff));
                opengl::gl_clear();
                bed.window.swap_buffers();
            }
            _ => {}
        }
    });
}
