// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::mem::MaybeUninit;
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::{str, thread, time};

use x11::keysym::{
    XK_BackSpace, XK_Delete, XK_Down, XK_End, XK_Escape, XK_Home, XK_Insert, XK_KP_Delete,
    XK_KP_Down, XK_KP_End, XK_KP_Enter, XK_KP_Home, XK_KP_Insert, XK_KP_Left, XK_KP_Page_Down,
    XK_KP_Page_Up, XK_KP_Right, XK_KP_Up, XK_Left, XK_Page_Down, XK_Page_Up, XK_Return, XK_Right,
    XK_Up,
};
use x11::xlib::{
    ButtonPress, ButtonPressMask, ButtonRelease, ButtonReleaseMask, Display, KeyPress,
    KeyPressMask, KeyRelease, KeyReleaseMask, KeymapNotify, KeymapStateMask, MotionNotify,
    PointerMotionMask, Screen, XBlackPixel, XClearWindow, XCloseDisplay, XCreateSimpleWindow,
    XDefaultScreen, XDefaultScreenOfDisplay, XDestroyWindow, XLookupString, XMapRaised, XNextEvent,
    XOpenDisplay, XPending, XRefreshKeyboardMapping, XRootWindowOfScreen, XSelectInput,
    XWhitePixel,
};

use crate::geom::{point2, vec2, Size2D};

use super::{ElemState, Event, Key, MouseButton};

// Wrapper around X window
pub(crate) struct Window {
    shared: Rc<SharedState>,
    window: u64,
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe { XDestroyWindow(self.shared.display, self.window) };
    }
}

// Wrapper around X server connection
pub(crate) struct EventLoop {
    shared: Rc<SharedState>,
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
            let shared = Rc::new(SharedState { display });
            EventLoop {
                shared,
                screen,
                screen_id,
            }
        }
    }

    pub(crate) fn new_window(&mut self, size: Size2D<u32>) -> Window {
        unsafe {
            // Open the window
            let window = XCreateSimpleWindow(
                self.shared.display,
                XRootWindowOfScreen(self.screen),
                0,
                0,
                size.width,
                size.height,
                1,
                XBlackPixel(self.shared.display, self.screen_id),
                XWhitePixel(self.shared.display, self.screen_id),
            );
            XSelectInput(
                self.shared.display,
                window,
                ButtonPressMask
                    | ButtonReleaseMask
                    | KeyPressMask
                    | KeyReleaseMask
                    | KeymapStateMask
                    | PointerMotionMask,
            );

            // Show the window
            XClearWindow(self.shared.display, window);
            XMapRaised(self.shared.display, window);
            Window {
                shared: self.shared.clone(),
                window,
            }
        }
    }

    #[allow(non_upper_case_globals)]
    pub(crate) fn run<F>(self, target_delta: time::Duration, mut callback: F)
    where
        F: FnMut(Event),
    {
        let mut keysym = 0;
        let mut str_buf = [0; 25];
        let mut last_frame = time::Instant::now();

        unsafe {
            let mut ev = MaybeUninit::uninit();
            loop {
                while XPending(self.shared.display) > 0 {
                    XNextEvent(self.shared.display, ev.as_mut_ptr());
                    let mut ev = ev.assume_init();
                    match ev.type_ {
                        KeymapNotify => {
                            XRefreshKeyboardMapping(&mut ev.mapping);
                        }
                        KeyPress | KeyRelease => {
                            let state = if ev.type_ == KeyPress {
                                ElemState::Pressed
                            } else {
                                ElemState::Released
                            };
                            let len = XLookupString(
                                &mut ev.key,
                                str_buf.as_mut_ptr() as _,
                                str_buf.len() as _,
                                &mut keysym,
                                null_mut(),
                            );
                            if let Some(key) = match keysym as u32 {
                                XK_Up | XK_KP_Up => Some(Key::Up),
                                XK_Down | XK_KP_Down => Some(Key::Down),
                                XK_Left | XK_KP_Left => Some(Key::Left),
                                XK_Right | XK_KP_Right => Some(Key::Right),
                                XK_Escape => Some(Key::Escape),
                                XK_Return | XK_KP_Enter => Some(Key::Enter),
                                XK_Insert | XK_KP_Insert => Some(Key::Insert),
                                XK_Delete | XK_KP_Delete => Some(Key::Delete),
                                XK_BackSpace => Some(Key::BackSpace),
                                XK_Home | XK_KP_Home => Some(Key::Home),
                                XK_End | XK_KP_End => Some(Key::End),
                                XK_Page_Up | XK_KP_Page_Up => Some(Key::PageUp),
                                XK_Page_Down | XK_KP_Page_Down => Some(Key::PageDown),
                                _ => None,
                            } {
                                callback(Event::Key { key, state })
                            } else if let Some(ch) = str::from_utf8(&str_buf[..len as usize])
                                .ok()
                                .and_then(|s| s.chars().next())
                            {
                                callback(Event::Char { ch, state })
                            }
                        }
                        ButtonPress => match ev.button.button {
                            1 => callback(Event::MouseButton {
                                button: MouseButton::Left,
                                state: ElemState::Pressed,
                            }),
                            2 => callback(Event::MouseButton {
                                button: MouseButton::Middle,
                                state: ElemState::Pressed,
                            }),
                            3 => callback(Event::MouseButton {
                                button: MouseButton::Right,
                                state: ElemState::Pressed,
                            }),
                            4 => callback(Event::Scroll(vec2(0.0, -1.0))),
                            5 => callback(Event::Scroll(vec2(0.0, 1.0))),
                            _ => {}
                        },
                        ButtonRelease => match ev.button.button {
                            1 => callback(Event::MouseButton {
                                button: MouseButton::Left,
                                state: ElemState::Released,
                            }),
                            2 => callback(Event::MouseButton {
                                button: MouseButton::Middle,
                                state: ElemState::Released,
                            }),
                            3 => callback(Event::MouseButton {
                                button: MouseButton::Right,
                                state: ElemState::Released,
                            }),
                            _ => {}
                        },
                        MotionNotify => callback(Event::MouseMotion(point2(
                            ev.motion.x as f64,
                            ev.motion.y as f64,
                        ))),
                        _ => {}
                    }
                }
                let elapsed = last_frame.elapsed();
                if elapsed <= target_delta {
                    thread::sleep(target_delta - elapsed);
                }
                let now = time::Instant::now();
                //callback(Event::Refresh(now - last_frame));
                last_frame = now;
            }
        }
    }
}

struct SharedState {
    display: *mut Display,
}

impl Drop for SharedState {
    fn drop(&mut self) {
        unsafe { XCloseDisplay(self.display) };
    }
}
