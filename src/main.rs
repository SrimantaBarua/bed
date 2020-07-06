// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::{point2, size2, Rect};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;

mod common;
mod opengl;
mod style;
mod window;

fn main() {
    let (mut window, event_loop) = window::Window::new("bed", size2(1024, 768));
    opengl::gl_init();

    event_loop.run(move |event, _, control_flow| {
        println!("event: {:?}", event);
        *control_flow = ControlFlow::Wait;
        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    window.resize(physical_size);
                    opengl::gl_viewport(Rect::new(
                        point2(0, 0),
                        size2(physical_size.width, physical_size.height).cast(),
                    ));
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    println!("scale_factor: {}", scale_factor);
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                opengl::gl_clear_color(style::Color::new(0xff, 0xff, 0xff, 0xff));
                opengl::gl_clear();
                window.swap_buffers();
            }
            _ => {}
        }
    });
}
