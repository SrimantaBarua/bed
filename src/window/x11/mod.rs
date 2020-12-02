use std::rc::Rc;
use std::{thread, time};

use super::{ElemState, Event, Modifiers, MouseButton};
use crate::geom::{point2, vec2, Size2D};

mod wrapper;
pub(crate) use wrapper::Window;

// Wrapper around X server connection
pub(crate) struct WindowManager {
    display: Rc<wrapper::Display>,
}

impl WindowManager {
    pub(crate) fn connect() -> WindowManager {
        let display = Rc::new(wrapper::Display::open().expect("failed to open X display"));
        WindowManager { display }
    }

    pub(crate) fn new_window(&mut self, size: Size2D<u32>, name: &str) -> Window {
        Window::create(self.display.clone(), size, name)
    }

    pub(crate) fn run<F>(self, target_delta: time::Duration, mut callback: F)
    where
        F: FnMut(Event),
    {
        let mut modifiers = Modifiers::empty();
        let mut last_frame = time::Instant::now();

        loop {
            while self.display.has_events() {
                let xev = self.display.next_event();
                match xev {
                    wrapper::Event::ButtonPress(bev) => match bev.button {
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
                        6 => callback(Event::Scroll(vec2(-1.0, 0.0))),
                        7 => callback(Event::Scroll(vec2(1.0, 0.0))),
                        _ => unimplemented!(),
                    },
                    wrapper::Event::ButtonRelease(bev) => match bev.button {
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
                    wrapper::Event::Expose(eev) => {
                        let window = unsafe { Window::from_raw(self.display.clone(), eev.window) };
                        callback(Event::Resized(window.size()));
                    }
                    wrapper::Event::KeyPress(mut kev) | wrapper::Event::KeyRelease(mut kev) => {
                        wrapper::handle_key_event(&mut kev, &mut modifiers, &mut callback);
                    }
                    wrapper::Event::MotionNotify(mev) => {
                        callback(Event::MouseMotion(point2(mev.x, mev.y).cast()));
                    }
                }
            }

            let elapsed = last_frame.elapsed();
            if elapsed <= target_delta {
                thread::sleep(target_delta - elapsed);
            }
            let now = time::Instant::now();
            callback(Event::Refresh(now - last_frame));
            last_frame = now;
        }
    }
}
