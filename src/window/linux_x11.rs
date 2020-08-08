// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::mem::MaybeUninit;
use std::ptr::null;

use x11::xlib::{
    Display, Screen, XBlackPixel, XClearWindow, XCloseDisplay, XCreateSimpleWindow, XDefaultScreen,
    XDefaultScreenOfDisplay, XDestroyWindow, XMapRaised, XNextEvent, XOpenDisplay,
    XRootWindowOfScreen, XWhitePixel,
};

use crate::geom::Size2D;

// Wrapper around X window
pub(crate) struct Window {
    display: *mut Display,
    window: u64,
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe { XDestroyWindow(self.display, self.window) };
    }
}

// Wrapper around X server connection
pub(crate) struct EventLoop {
    display: *mut Display,
    screen: *mut Screen,
    screen_id: i32,
}

impl EventLoop {
    pub(crate) fn new() -> EventLoop {
        unsafe {
            // Open the X display
            let display = XOpenDisplay(null());
            assert!(!display.is_null(), "ERROR: Failed to open X display");
            let screen = XDefaultScreenOfDisplay(display);
            let screen_id = XDefaultScreen(display);
            EventLoop {
                display,
                screen,
                screen_id,
            }
        }
    }

    pub(crate) fn new_window(&mut self, size: Size2D<u32>) -> Window {
        unsafe {
            // Open the window
            let window = XCreateSimpleWindow(
                self.display,
                XRootWindowOfScreen(self.screen),
                0,
                0,
                size.width,
                size.height,
                1,
                XBlackPixel(self.display, self.screen_id),
                XWhitePixel(self.display, self.screen_id),
            );

            // Show the window
            XClearWindow(self.display, window);
            XMapRaised(self.display, window);
            Window {
                display: self.display,
                window,
            }
        }
    }

    pub(crate) fn run<F>(self, f: F)
    where
        F: FnMut(),
    {
        unsafe {
            let mut ev = MaybeUninit::uninit();
            loop {
                XNextEvent(self.display, ev.as_mut_ptr());
            }
        }
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        unsafe { XCloseDisplay(self.display) };
    }
}
