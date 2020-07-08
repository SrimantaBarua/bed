// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::Size2D;
use glutin::dpi::PhysicalSize;
use glutin::event_loop::EventLoop;
use glutin::{Api, GlProfile, GlRequest, PossiblyCurrent, WindowedContext};

use crate::common::PixelSize;

pub(crate) struct Window {
    context: WindowedContext<PossiblyCurrent>,
}

impl Window {
    pub(crate) fn new(title: &str, size: Size2D<u32, PixelSize>) -> (Window, EventLoop<()>) {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(PhysicalSize::new(size.width, size.height));
        let windowed_context = glutin::ContextBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .with_gl_profile(GlProfile::Core)
            .build_windowed(window_builder, &event_loop)
            .unwrap();
        let context = unsafe { windowed_context.make_current().unwrap() };
        // Initialize opengl
        {
            let context = context.context();
            gl::load_with(|s| context.get_proc_address(s));
        }
        (Window { context }, event_loop)
    }

    pub(crate) fn swap_buffers(&mut self) {
        self.context.swap_buffers().unwrap();
    }

    pub(crate) fn resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.context.resize(physical_size);
    }

    pub(crate) fn scale_factor(&self) -> f64 {
        self.context.window().scale_factor()
    }
}
