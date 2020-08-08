// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::time::Duration;

use crate::geom::{Point2D, Size2D, Vector2D};

// Whether the element (key, mouse button) was pressed or released
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ElemState {
    Pressed,
    Released,
}

// Special keys (non-unicode)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Key {
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Keypad0,
    Keypad1,
    Keypad2,
    Keypad3,
    Keypad4,
    Keypad5,
    Keypad6,
    Keypad7,
    Keypad8,
    Keypad9,
    Space,
    Up,
    Down,
    Left,
    Right,
    Escape,
    Tab,
    Enter,
    Insert,
    Delete,
    BackSpace,
    Home,
    End,
    PageUp,
    PageDown,
}

// Mouse buttons
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MouseButton {
    Left,
    Right,
    Middle,
}

// State of modifier keys
bitflags! {
    pub(crate) struct Modifers : u8 {
        const SHIFT = 1;
        const CTRL  = 2;
        const ALT   = 4;
    }
}

// Type of event available in the callback
#[derive(Debug)]
pub(crate) enum Event {
    Char {
        ch: char,
        state: ElemState,
    },
    Key {
        key: Key,
        state: ElemState,
    },
    MouseButton {
        button: MouseButton,
        state: ElemState,
    },
    Modifiers(Modifers),
    MouseMotion(Point2D<f64>),
    Resized(Size2D<u32>),
    Scroll(Vector2D<f64>),
    Refresh(Duration),
}

#[cfg(target_os = "linux")]
mod linux_x11;

#[cfg(target_os = "linux")]
pub(crate) use linux_x11::*;
