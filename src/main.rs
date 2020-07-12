// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::{point2, size2, Rect};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;

mod common;
mod opengl;
mod shapes;
mod style;
mod text;
mod window;

use style::Color;

struct Bed {
    font_core: text::FontCore,
    window: window::Window,
    scale_factor: f64,
}

impl Bed {
    fn new() -> (Bed, glutin::event_loop::EventLoop<()>) {
        let window_size = size2(1024, 768);
        let (window, event_loop) = window::Window::new("bed", window_size);
        opengl::gl_init();
        let scale_factor = window.scale_factor();
        let font_core = text::FontCore::new(window_size);
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

    let mut font = bed.font_core.find("monospace").unwrap();

    event_loop.run(move |event, _, control_flow| {
        println!("event: {:?}", event);
        *control_flow = ControlFlow::Wait;
        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    bed.window.resize(physical_size);
                    let window_size = size2(physical_size.width, physical_size.height);
                    opengl::gl_viewport(Rect::new(point2(0, 0), window_size.cast()));
                    bed.font_core.set_window_size(window_size);
                    bed.window.request_redraw();
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    bed.scale_factor = scale_factor;
                    bed.window.request_redraw();
                }
                _ => {}
            },
            Event::MainEventsCleared => {}
            Event::RedrawRequested(_) => {
                println!("************* HERE *****************");
                opengl::gl_clear_color(style::Color::new(0xff, 0xff, 0xff, 0xff));
                opengl::gl_clear();

                let text = "Hello";
                let shaped = font.shape(text, style::TextSize(64), style::TextStyle::default());
                shaped.draw(point2(60.0, 60.0), Color::new(0, 0, 0, 0xff));
                font.flush_glyphs();

                bed.window.swap_buffers();
            }
            _ => {}
        }
    });
}
