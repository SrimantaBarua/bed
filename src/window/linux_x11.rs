// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::mem::MaybeUninit;
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::{str, thread, time};

use x11::keysym::{
    XK_Alt_L, XK_Alt_R, XK_BackSpace, XK_Control_L, XK_Control_R, XK_Delete, XK_Down, XK_End,
    XK_Escape, XK_Home, XK_Insert, XK_KP_Delete, XK_KP_Down, XK_KP_End, XK_KP_Enter, XK_KP_Home,
    XK_KP_Insert, XK_KP_Left, XK_KP_Page_Down, XK_KP_Page_Up, XK_KP_Right, XK_KP_Space, XK_KP_Tab,
    XK_KP_Up, XK_Left, XK_Page_Down, XK_Page_Up, XK_Return, XK_Right, XK_Shift_L, XK_Shift_R,
    XK_Tab, XK_Up, XK_a, XK_b, XK_c, XK_d, XK_e, XK_f, XK_g, XK_h, XK_i, XK_j, XK_k, XK_l, XK_m,
    XK_n, XK_o, XK_p, XK_q, XK_r, XK_s, XK_space, XK_t, XK_u, XK_v, XK_w, XK_x, XK_y, XK_z, XK_0,
    XK_1, XK_2, XK_3, XK_4, XK_5, XK_6, XK_7, XK_8, XK_9, XK_A, XK_B, XK_C, XK_D, XK_E, XK_F, XK_G,
    XK_H, XK_I, XK_J, XK_K, XK_KP_0, XK_KP_1, XK_KP_2, XK_KP_3, XK_KP_4, XK_KP_5, XK_KP_6, XK_KP_7,
    XK_KP_8, XK_KP_9, XK_L, XK_M, XK_N, XK_O, XK_P, XK_Q, XK_R, XK_S, XK_T, XK_U, XK_V, XK_W, XK_X,
    XK_Y, XK_Z,
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

use super::{ElemState, Event, Key, Modifers, MouseButton};

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
}

impl EventLoop {
    pub(crate) fn with_window(size: Size2D<u32>) -> (EventLoop, Window) {
        unsafe {
            // Open the X display
            let display = XOpenDisplay(null());
            assert!(!display.is_null(), "ERROR: Failed to open X display");
            let screen = XDefaultScreenOfDisplay(display);
            let screen_id = XDefaultScreen(display);
            let shared = Rc::new(SharedState { display });

            let event_loop = EventLoop {
                shared: shared.clone(),
            };

            // Open the window
            let window = XCreateSimpleWindow(
                shared.display,
                XRootWindowOfScreen(screen),
                0,
                0,
                size.width,
                size.height,
                1,
                XBlackPixel(shared.display, screen_id),
                XWhitePixel(shared.display, screen_id),
            );
            XSelectInput(
                shared.display,
                window,
                ButtonPressMask
                    | ButtonReleaseMask
                    | KeyPressMask
                    | KeyReleaseMask
                    | KeymapStateMask
                    | PointerMotionMask,
            );

            // Show the window
            XClearWindow(shared.display, window);
            XMapRaised(shared.display, window);
            let window = Window { shared, window };

            (event_loop, window)
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
        let mut modifers = Modifers::empty();

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
                            if let Some(key) = keysym_map(
                                keysym,
                                ev.type_ == KeyPress,
                                &mut modifers,
                                &mut callback,
                            ) {
                                callback(Event::Key { key, state })
                            }
                            if let Ok(s) = str::from_utf8(&str_buf[..len as usize]) {
                                for ch in s.chars().filter(|ch| !ch.is_ascii_control()) {
                                    callback(Event::Char { ch, state })
                                }
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

fn update_modifiers<F>(orig: &mut Modifers, new: Modifers, set: bool, f: &mut F)
where
    F: FnMut(Event),
{
    let old = *orig;
    orig.set(new, set);
    if old != *orig {
        f(Event::Modifiers(*orig));
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

#[allow(non_upper_case_globals)]
fn keysym_map<F>(keysym: u64, press: bool, modifers: &mut Modifers, callback: &mut F) -> Option<Key>
where
    F: FnMut(Event),
{
    match keysym as u32 {
        XK_a | XK_A => Some(Key::A),
        XK_b | XK_B => Some(Key::B),
        XK_c | XK_C => Some(Key::C),
        XK_d | XK_D => Some(Key::D),
        XK_e | XK_E => Some(Key::E),
        XK_f | XK_F => Some(Key::F),
        XK_g | XK_G => Some(Key::G),
        XK_h | XK_H => Some(Key::H),
        XK_i | XK_I => Some(Key::I),
        XK_j | XK_J => Some(Key::J),
        XK_k | XK_K => Some(Key::K),
        XK_l | XK_L => Some(Key::L),
        XK_m | XK_M => Some(Key::M),
        XK_n | XK_N => Some(Key::N),
        XK_o | XK_O => Some(Key::O),
        XK_p | XK_P => Some(Key::P),
        XK_q | XK_Q => Some(Key::Q),
        XK_r | XK_R => Some(Key::R),
        XK_s | XK_S => Some(Key::S),
        XK_t | XK_T => Some(Key::T),
        XK_u | XK_U => Some(Key::U),
        XK_v | XK_V => Some(Key::V),
        XK_w | XK_W => Some(Key::W),
        XK_x | XK_X => Some(Key::X),
        XK_y | XK_Y => Some(Key::Y),
        XK_z | XK_Z => Some(Key::Z),
        XK_0 => Some(Key::Num0),
        XK_1 => Some(Key::Num1),
        XK_2 => Some(Key::Num2),
        XK_3 => Some(Key::Num3),
        XK_4 => Some(Key::Num4),
        XK_5 => Some(Key::Num5),
        XK_6 => Some(Key::Num6),
        XK_7 => Some(Key::Num7),
        XK_8 => Some(Key::Num8),
        XK_9 => Some(Key::Num9),
        XK_KP_0 => Some(Key::Keypad0),
        XK_KP_1 => Some(Key::Keypad1),
        XK_KP_2 => Some(Key::Keypad2),
        XK_KP_3 => Some(Key::Keypad3),
        XK_KP_4 => Some(Key::Keypad4),
        XK_KP_5 => Some(Key::Keypad5),
        XK_KP_6 => Some(Key::Keypad6),
        XK_KP_7 => Some(Key::Keypad7),
        XK_KP_8 => Some(Key::Keypad8),
        XK_KP_9 => Some(Key::Keypad9),
        XK_space | XK_KP_Space => Some(Key::Space),
        XK_Up | XK_KP_Up => Some(Key::Up),
        XK_Down | XK_KP_Down => Some(Key::Down),
        XK_Left | XK_KP_Left => Some(Key::Left),
        XK_Right | XK_KP_Right => Some(Key::Right),
        XK_Escape => Some(Key::Escape),
        XK_Tab | XK_KP_Tab => Some(Key::Tab),
        XK_Return | XK_KP_Enter => Some(Key::Enter),
        XK_Insert | XK_KP_Insert => Some(Key::Insert),
        XK_Delete | XK_KP_Delete => Some(Key::Delete),
        XK_BackSpace => Some(Key::BackSpace),
        XK_Home | XK_KP_Home => Some(Key::Home),
        XK_End | XK_KP_End => Some(Key::End),
        XK_Page_Up | XK_KP_Page_Up => Some(Key::PageUp),
        XK_Page_Down | XK_KP_Page_Down => Some(Key::PageDown),
        XK_Shift_L | XK_Shift_R => {
            update_modifiers(modifers, Modifers::SHIFT, press, callback);
            None
        }
        XK_Alt_L | XK_Alt_R => {
            update_modifiers(modifers, Modifers::ALT, press, callback);
            None
        }
        XK_Control_L | XK_Control_R => {
            update_modifiers(modifers, Modifers::CTRL, press, callback);
            None
        }
        _ => None,
    }
}
