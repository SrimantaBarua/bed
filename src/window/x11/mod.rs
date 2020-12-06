use std::rc::Rc;
use std::{thread, time};

use geom::{point2, vec2, Point2D, Size2D};
use x11::xinput2::{XI_DeviceChanged, XI_Motion};
use x11::xlib::{XFreeEventData, XGetEventData};

use super::{ElemState, Event, Modifiers, MouseButton};

mod wrapper;
pub(crate) use wrapper::Window;

// Wrapper around X server connection
pub(crate) struct WindowManager {
    display: Rc<wrapper::Display>,
    xinput2: wrapper::Input,
    scroll_info: wrapper::ScrollDeviceInfo,
    root_cursor: Point2D<f64>,
}

impl WindowManager {
    pub(crate) fn connect() -> WindowManager {
        let display = Rc::new(wrapper::Display::open().expect("failed to open X display"));
        let root_cursor = display.get_root_cursor();
        let xinput2 = wrapper::Input::open(display.clone());
        let scroll_info = xinput2
            .find_scroll_devinfo()
            .expect("failed to find scroll device");
        eprintln!("Scroll Info: {:?}", scroll_info);
        WindowManager {
            display,
            xinput2,
            scroll_info,
            root_cursor,
        }
    }

    pub(crate) fn new_window(&mut self, size: Size2D<u32>, name: &str) -> Window {
        let window = Window::create(self.display.clone(), size, name);
        self.xinput2
            .register_scroll_events(&window, &self.scroll_info);
        window
    }

    #[allow(non_upper_case_globals)]
    pub(crate) fn run<F>(mut self, target_delta: time::Duration, mut callback: F)
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
                        _ => {}
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
                    wrapper::Event::Generic(mut gev) => {
                        unsafe { XGetEventData(self.display.raw, &mut gev) };

                        if gev.extension == self.xinput2.major_opcode {
                            let ev = wrapper::DeviceEvent::from_raw(gev.data as _);
                            match gev.evtype {
                                XI_Motion => ev.scroll_event(
                                    &mut self.scroll_info,
                                    &mut self.root_cursor,
                                    &mut callback,
                                ),
                                XI_DeviceChanged => eprintln!("XI_DeviceChanged"),
                                _ => unimplemented!(),
                            }
                        }

                        unsafe { XFreeEventData(self.display.raw, &mut gev) };
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
