// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ffi::CString;
use std::mem::MaybeUninit;
use std::ops::Drop;
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::{slice, str};

use x11::xinput2::{
    XIAllMasterDevices, XIDeviceEvent, XIEventMask, XIFreeDeviceInfo, XIMaskIsSet, XIQueryDevice,
    XIQueryVersion, XIScrollClass, XIScrollClassInfo, XIScrollTypeHorizontal, XIScrollTypeVertical,
    XISelectEvents, XISetMask, XIValuatorClass, XIValuatorClassInfo, XI_DeviceChanged, XI_Motion,
};
use x11::xlib::{
    BadRequest, ButtonPress, ButtonPressMask, ButtonRelease, ButtonReleaseMask, Expose,
    ExposureMask, GenericEvent, KeyPress, KeyPressMask, KeyRelease, KeyReleaseMask, Screen,
    XBlackPixel, XButtonEvent, XClearWindow, XCloseDisplay, XCreateSimpleWindow, XDefaultScreen,
    XDefaultScreenOfDisplay, XDestroyWindow, XEvent, XExposeEvent, XFlush, XGenericEventCookie,
    XGetWindowAttributes, XKeyEvent, XLookupString, XMapRaised, XNextEvent, XOpenDisplay, XPending,
    XQueryExtension, XQueryPointer, XRootWindowOfScreen, XSelectInput, XStoreName, XWhitePixel,
};

use crate::geom::{point2, size2, vec2, Point2D, Size2D};
use crate::window::{ElemState, Event as BedEvent, Key, Modifiers};

// Wrapper around X display
pub(super) struct Display {
    pub(super) raw: *mut x11::xlib::Display,
}

impl Display {
    // Open X display
    pub(super) fn open() -> Option<Display> {
        let raw = unsafe { XOpenDisplay(null()) };
        if raw.is_null() {
            None
        } else {
            Some(Display { raw })
        }
    }

    // Check if there are more events in the queue
    pub(super) fn has_events(&self) -> bool {
        unsafe { XPending(self.raw) > 0 }
    }

    // Get next X event
    pub(super) fn next_event(&self) -> Event {
        unsafe {
            let mut raw_ev = MaybeUninit::uninit();
            XNextEvent(self.raw, raw_ev.as_mut_ptr());
            Event::from_raw(raw_ev.assume_init())
        }
    }

    // Get cursor position relative to root window
    pub(super) fn get_root_cursor(&self) -> Point2D<f64> {
        let (mut root_x, mut root_y, mut win_x, mut win_y, mut root, mut child, mut mask) =
            (0, 0, 0, 0, 0, 0, 0);
        unsafe {
            XQueryPointer(
                self.raw,
                self.root_window(),
                &mut root,
                &mut child,
                &mut root_x,
                &mut root_y,
                &mut win_x,
                &mut win_y,
                &mut mask,
            )
        };
        point2(root_x, root_y).cast()
    }

    // Get default screen for X display
    unsafe fn default_screen(&self) -> *mut Screen {
        XDefaultScreenOfDisplay(self.raw)
    }

    // Get default screen number of X display
    unsafe fn default_screen_number(&self) -> i32 {
        XDefaultScreen(self.raw)
    }

    // Get root window
    unsafe fn root_window(&self) -> u64 {
        XRootWindowOfScreen(self.default_screen())
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe { XCloseDisplay(self.raw) };
    }
}

// Wrapper around X window
pub(crate) struct Window {
    raw: u64,
    display: Rc<Display>,
    owns: bool,
}

impl Window {
    // Create window on display with given dimensions
    pub(super) fn create(display: Rc<Display>, size: Size2D<u32>, name: &str) -> Window {
        let mut window = Window {
            raw: unsafe { Window::create_window(&display, size) },
            display,
            owns: true,
        };
        unsafe {
            window.select_events();
            window.set_name(name);
            window.show();
        }
        window
    }

    // Wrap window around raw
    pub(super) unsafe fn from_raw(display: Rc<Display>, raw: u64) -> Window {
        Window {
            raw,
            display,
            owns: false,
        }
    }

    // Get window size
    pub(super) fn size(&self) -> Size2D<u32> {
        unsafe {
            let mut attribs = MaybeUninit::uninit();
            XGetWindowAttributes(self.display.raw, self.raw, attribs.as_mut_ptr());
            let attribs = &attribs.assume_init();
            size2(attribs.width, attribs.height).cast()
        }
    }

    unsafe fn create_window(display: &Display, size: Size2D<u32>) -> u64 {
        XCreateSimpleWindow(
            display.raw,
            display.root_window(),
            0,
            0,
            size.width,
            size.height,
            1,
            XBlackPixel(display.raw, display.default_screen_number()),
            XWhitePixel(display.raw, display.default_screen_number()),
        )
    }

    unsafe fn select_events(&mut self) {
        // Select events
        XSelectInput(
            self.display.raw,
            self.raw,
            ButtonPressMask | ButtonReleaseMask | ExposureMask | KeyPressMask | KeyReleaseMask,
        );
    }

    unsafe fn set_name(&mut self, name: &str) {
        let cstr = CString::new(name).unwrap();
        XStoreName(self.display.raw, self.raw, cstr.as_ptr());
    }

    unsafe fn show(&mut self) {
        XClearWindow(self.display.raw, self.raw);
        XMapRaised(self.display.raw, self.raw);
        XFlush(self.display.raw);
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if self.owns {
            unsafe { XDestroyWindow(self.display.raw, self.raw) };
        }
    }
}

// Information about scroll device
#[derive(Debug)]
pub(super) struct ScrollDeviceInfo {
    pub(super) device_id: i32,
    pub(super) vertical_valuator: i32,
    pub(super) vertical_resolution: i32,
    pub(super) vertical_increment: f64,
    pub(super) vertical_min: f64,
    pub(super) vertical_max: f64,
    pub(super) vertical_value: f64,
    pub(super) vertical_mode: i32,
    pub(super) horizontal_valuator: i32,
    pub(super) horizontal_resolution: i32,
    pub(super) horizontal_increment: f64,
    pub(super) horizontal_min: f64,
    pub(super) horizontal_max: f64,
    pub(super) horizontal_value: f64,
    pub(super) horizontal_mode: i32,
}

// Wrapper around XInput2 extension
pub(super) struct Input {
    display: Rc<Display>,
    pub(super) major_opcode: i32,
    //first_event: i32,
    //first_error: i32,
    //major: i32,
    //minor: i32,
}

impl Input {
    pub(super) fn open(display: Rc<Display>) -> Input {
        let (mut op, mut ev, mut err, mut major, mut minor) = (0, 0, 0, 2, 1);
        unsafe {
            if XQueryExtension(
                display.raw,
                b"XInputExtension\0".as_ptr() as _,
                &mut op,
                &mut ev,
                &mut err,
            ) == 0
            {
                panic!("XInput2 extension not supported");
            }
            if XIQueryVersion(display.raw, &mut major, &mut minor) == BadRequest as i32 {
                panic!(
                    "XInput2 not supported. Server only supports v{}.{}",
                    major, minor
                );
            }
        }
        Input {
            display,
            major_opcode: op,
            //first_event: ev,
            //first_error: err,
            //major,
            //minor,
        }
    }

    pub(super) fn register_scroll_events(
        &mut self,
        window: &Window,
        scroll_info: &ScrollDeviceInfo,
    ) {
        let mut mask = [0];
        unsafe {
            XISetMask(&mut mask, XI_Motion);
            XISetMask(&mut mask, XI_DeviceChanged);
            let mut event_mask = XIEventMask {
                deviceid: scroll_info.device_id,
                mask_len: 1,
                mask: mask.as_mut_ptr(),
            };
            XISelectEvents(self.display.raw, window.raw, &mut event_mask, 1);
        }
    }

    #[allow(non_upper_case_globals)]
    pub(super) fn find_scroll_devinfo(&self) -> Option<ScrollDeviceInfo> {
        let mut ndevs = 0;

        unsafe {
            let devinfo_ptr = XIQueryDevice(self.display.raw, XIAllMasterDevices, &mut ndevs);
            let devinfo = slice::from_raw_parts(devinfo_ptr, ndevs as usize);

            for dev in devinfo {
                let (mut vscroll, mut vval, mut hscroll, mut hval) = (None, None, None, None);

                for j in 0..dev.num_classes as isize {
                    let ptr = *dev.classes.offset(j);
                    let class = &*ptr;
                    if class._type != XIScrollClass {
                        continue;
                    }

                    let scroll = &*(ptr as *const XIScrollClassInfo);
                    let mut valuator = None;

                    // Find corresponding valuator
                    for k in 0..dev.num_classes as isize {
                        let ptr = *dev.classes.offset(k);
                        let class = &*ptr;
                        if class._type != XIValuatorClass {
                            continue;
                        }

                        valuator = Some(&*(ptr as *const XIValuatorClassInfo));
                        break;
                    }
                    if valuator.is_none() {
                        continue;
                    }

                    match scroll.scroll_type {
                        XIScrollTypeVertical => {
                            vscroll = Some(*scroll);
                            vval = Some(*valuator.unwrap());
                        }
                        XIScrollTypeHorizontal => {
                            hscroll = Some(*scroll);
                            hval = Some(*valuator.unwrap());
                        }
                        _ => unimplemented!(),
                    }

                    if vscroll.is_some() && vval.is_some() && hscroll.is_some() && hval.is_some() {
                        let dev_id = dev.deviceid;
                        let (vscroll, vval, hscroll, hval) = (
                            vscroll.unwrap(),
                            vval.unwrap(),
                            hscroll.unwrap(),
                            hval.unwrap(),
                        );
                        XIFreeDeviceInfo(devinfo_ptr);

                        return Some(ScrollDeviceInfo {
                            device_id: dev_id,
                            vertical_valuator: vscroll.number,
                            vertical_increment: vscroll.increment,
                            vertical_resolution: vval.resolution,
                            vertical_min: vval.min,
                            vertical_max: vval.max,
                            vertical_value: vval.value,
                            vertical_mode: vval.mode,
                            horizontal_valuator: hscroll.number,
                            horizontal_increment: hscroll.increment,
                            horizontal_resolution: hval.resolution,
                            horizontal_min: hval.min,
                            horizontal_max: hval.max,
                            horizontal_value: hval.value,
                            horizontal_mode: hval.mode,
                        });
                    }
                }
            }

            XIFreeDeviceInfo(devinfo_ptr);
        }
        None
    }
}

// Wrapper around X event
pub(super) enum Event {
    ButtonPress(XButtonEvent),
    ButtonRelease(XButtonEvent),
    Expose(XExposeEvent),
    KeyPress(XKeyEvent),
    KeyRelease(XKeyEvent),
    Generic(XGenericEventCookie),
}

impl Event {
    #[allow(non_upper_case_globals)]
    unsafe fn from_raw(raw: XEvent) -> Event {
        match raw.type_ {
            ButtonPress => Event::ButtonPress(raw.button),
            ButtonRelease => Event::ButtonRelease(raw.button),
            Expose => Event::Expose(raw.expose),
            KeyPress => Event::KeyPress(raw.key),
            KeyRelease => Event::KeyRelease(raw.key),
            GenericEvent => Event::Generic(raw.generic_event_cookie),
            _ => unimplemented!(),
        }
    }
}

// Get bed key from X key event
#[allow(non_upper_case_globals)]
pub(super) fn handle_key_event<F>(
    key_event: &mut XKeyEvent,
    modifers: &mut Modifiers,
    callback: &mut F,
) where
    F: FnMut(BedEvent),
{
    use x11::keysym::{
        XK_Alt_L, XK_Alt_R, XK_BackSpace, XK_Control_L, XK_Control_R, XK_Delete, XK_Down, XK_End,
        XK_Escape, XK_Home, XK_Insert, XK_KP_Delete, XK_KP_Down, XK_KP_End, XK_KP_Enter,
        XK_KP_Home, XK_KP_Insert, XK_KP_Left, XK_KP_Page_Down, XK_KP_Page_Up, XK_KP_Right,
        XK_KP_Space, XK_KP_Tab, XK_KP_Up, XK_Left, XK_Page_Down, XK_Page_Up, XK_Return, XK_Right,
        XK_Shift_L, XK_Shift_R, XK_Tab, XK_Up, XK_a, XK_b, XK_c, XK_d, XK_e, XK_f, XK_g, XK_h,
        XK_i, XK_j, XK_k, XK_l, XK_m, XK_n, XK_o, XK_p, XK_q, XK_r, XK_s, XK_space, XK_t, XK_u,
        XK_v, XK_w, XK_x, XK_y, XK_z, XK_0, XK_1, XK_2, XK_3, XK_4, XK_5, XK_6, XK_7, XK_8, XK_9,
        XK_A, XK_B, XK_C, XK_D, XK_E, XK_F, XK_F1, XK_F10, XK_F11, XK_F12, XK_F2, XK_F3, XK_F4,
        XK_F5, XK_F6, XK_F7, XK_F8, XK_F9, XK_G, XK_H, XK_I, XK_J, XK_K, XK_KP_0, XK_KP_1, XK_KP_2,
        XK_KP_3, XK_KP_4, XK_KP_5, XK_KP_6, XK_KP_7, XK_KP_8, XK_KP_9, XK_L, XK_M, XK_N, XK_O,
        XK_P, XK_Q, XK_R, XK_S, XK_T, XK_U, XK_V, XK_W, XK_X, XK_Y, XK_Z,
    };

    let mut str_buf = [0; 25];
    let mut keysym = 0;

    let state = if key_event.type_ == KeyPress {
        ElemState::Pressed
    } else {
        ElemState::Released
    };
    let len = unsafe {
        XLookupString(
            key_event,
            str_buf.as_mut_ptr() as _,
            str_buf.len() as _,
            &mut keysym,
            null_mut(),
        )
    };

    let mut update_modifiers = |new| {
        let old = *modifers;
        modifers.set(new, state == ElemState::Pressed);
        if old != *modifers {
            callback(BedEvent::Modifiers(*modifers));
        }
    };

    let converted = match keysym as u32 {
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
        XK_F1 => Some(Key::F1),
        XK_F2 => Some(Key::F2),
        XK_F3 => Some(Key::F3),
        XK_F4 => Some(Key::F4),
        XK_F5 => Some(Key::F5),
        XK_F6 => Some(Key::F6),
        XK_F7 => Some(Key::F7),
        XK_F8 => Some(Key::F8),
        XK_F9 => Some(Key::F9),
        XK_F10 => Some(Key::F10),
        XK_F11 => Some(Key::F11),
        XK_F12 => Some(Key::F12),
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
            update_modifiers(Modifiers::SHIFT);
            None
        }
        XK_Alt_L | XK_Alt_R => {
            update_modifiers(Modifiers::ALT);
            None
        }
        XK_Control_L | XK_Control_R => {
            update_modifiers(Modifiers::CTRL);
            None
        }
        _ => None,
    };

    if let Some(key) = converted {
        callback(BedEvent::Key { key, state })
    }

    if let Ok(s) = str::from_utf8(&str_buf[..len as usize]) {
        for ch in s.chars().filter(|ch| !ch.is_ascii_control()) {
            callback(BedEvent::Char { ch, state });
        }
    }
}

// Wrapper around XIDeviceEvent
pub(super) struct DeviceEvent {
    raw: *const XIDeviceEvent,
}

impl DeviceEvent {
    pub(super) fn from_raw(raw: *const XIDeviceEvent) -> DeviceEvent {
        DeviceEvent { raw }
    }

    pub(super) fn scroll_event<F>(
        &self,
        info: &mut ScrollDeviceInfo,
        cur_cursor: &mut Point2D<f64>,
        callback: &mut F,
    ) where
        F: FnMut(BedEvent),
    {
        let mut scroll_amount = vec2(0.0, 0.0);
        let data = unsafe { &*self.raw };

        let new_cursor = point2(data.root_x, data.root_y);
        if new_cursor.x != cur_cursor.x || new_cursor.y != cur_cursor.y {
            callback(BedEvent::MouseMotion(point2(data.event_x, data.event_y)));
            *cur_cursor = new_cursor;
        }

        let mask_len = data.valuators.mask_len;
        let mask = unsafe { slice::from_raw_parts(data.valuators.mask, mask_len as usize) };
        let mut value_ptr = data.valuators.values;

        for i in 0..mask_len {
            if !XIMaskIsSet(mask, i) {
                continue;
            }

            let value = unsafe {
                let val = *value_ptr;
                value_ptr = value_ptr.offset(1);
                val
            };

            if i == info.vertical_valuator {
                let delta = (value - info.vertical_value) / info.vertical_increment;
                scroll_amount.y += delta;
                info.vertical_value = value;
            } else if i == info.horizontal_valuator {
                let delta = (value - info.horizontal_value) / info.horizontal_increment;
                scroll_amount.x += delta;
                info.horizontal_value = value;
            }
        }

        if scroll_amount.x != 0.0 || scroll_amount.y != 0.0 {
            callback(BedEvent::Scroll(scroll_amount));
        }
    }
}
