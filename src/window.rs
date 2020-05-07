// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::sync::mpsc::Receiver;
use std::time::Duration;

use euclid::{size2, Size2D};
use glfw::{
    Context, Glfw, OpenGlProfileHint, Window as GlfwWindow, WindowEvent, WindowHint, WindowMode,
};

use crate::common::{PixelSize, DPI};

// Wrapper around GLFW window
pub(crate) struct Window {
    window: GlfwWindow,
}

type WindowRet = (Window, Size2D<u32, DPI>, Receiver<(f64, WindowEvent)>);

impl Window {
    // Create a new window. Compute monitor DPI. Return Window, DPI, and handle to GLFW events
    pub(crate) fn new(glfw: &mut Glfw, size: Size2D<u32, PixelSize>, title: &str) -> WindowRet {
        // Initialize GLFW
        glfw.window_hint(WindowHint::ContextVersion(3, 3));
        glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        // Create GLFW window, and calculate DPI
        let (mut window, events, dpi) = glfw.with_primary_monitor(|glfw, m| {
            let (window, events) = glfw
                .create_window(size.width, size.height, title, WindowMode::Windowed)
                .expect("failed to create GLFW window");
            let dpi = m
                .and_then(|m| {
                    const MM_IN: f32 = 0.0393701;
                    let (width_mm, height_mm) = m.get_physical_size();
                    let (width_in, height_in) = (width_mm as f32 * MM_IN, height_mm as f32 * MM_IN);
                    m.get_video_mode().map(|vm| {
                        let (width_p, height_p) = (vm.width as f32, vm.height as f32);
                        size2((width_p / width_in) as u32, (height_p / height_in) as u32)
                    })
                })
                .unwrap_or(size2(96, 96));
            (window, events, dpi)
        });
        // Make window the current GL context and set polling options
        window.make_current();
        window.set_key_polling(true);
        window.set_char_polling(true);
        window.set_scroll_polling(true);
        window.set_refresh_polling(true);
        window.set_framebuffer_size_polling(true);
        window.set_mouse_button_polling(true);
        // Return wrapper
        (Window { window }, dpi, events)
    }

    // Check if window should be close
    pub(crate) fn should_close(&self) -> bool {
        self.window.should_close()
    }

    // Refresh window contents
    pub(crate) fn refresh(&mut self, _duration: Duration) {
        self.window.swap_buffers();
    }
}
