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
    Up,
    Down,
    Left,
    Right,
    Escape,
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
    MouseMotion(Point2D<f64>),
    Resized(Size2D<u32>),
    Scroll(Vector2D<f64>),
    Refresh(Duration),
}

#[cfg(target_os = "linux")]
mod linux_x11;

#[cfg(target_os = "linux")]
pub(crate) use linux_x11::*;
