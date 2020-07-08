// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

// Freetype font sizes
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct f26_6(i32);

impl From<f32> for f26_6 {
    fn from(f: f32) -> f26_6 {
        f26_6((f * 64.0).round() as i32)
    }
}

impl Into<f32> for f26_6 {
    fn into(self) -> f32 {
        (self.0 as f32) / 64.0
    }
}

impl f26_6 {
    pub(super) fn from_raw(raw: i32) -> f26_6 {
        f26_6(raw)
    }

    pub(super) fn to_raw(self) -> i32 {
        self.0
    }
}

impl fmt::Display for f26_6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let num: f32 = (*self).into();
        write!(f, "{}", num)
    }
}

impl fmt::Debug for f26_6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let num: f32 = (*self).into();
        write!(f, "{}", num)
    }
}
