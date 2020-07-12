// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;
use std::ops::{Add, AddAssign, Sub};

// Freetype font sizes
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(non_camel_case_types)]
pub(crate) struct f26_6(i32);

impl From<f32> for f26_6 {
    fn from(f: f32) -> f26_6 {
        f26_6((f * 64.0).round() as i32)
    }
}

impl f26_6 {
    pub(super) fn from_raw(raw: i32) -> f26_6 {
        f26_6(raw)
    }

    pub(super) fn to_raw(self) -> i32 {
        self.0
    }

    pub(super) fn to_f32(self) -> f32 {
        (self.0 as f32) / 64.0
    }

    pub(super) fn floor(self) -> f26_6 {
        f26_6(self.0 & !63)
    }
}

impl fmt::Display for f26_6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_f32())
    }
}

impl fmt::Debug for f26_6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_f32())
    }
}

impl AddAssign for f26_6 {
    fn add_assign(&mut self, other: f26_6) {
        self.0 += other.0
    }
}

impl Add for f26_6 {
    type Output = Self;

    fn add(self, b: f26_6) -> Self {
        f26_6(self.0 + b.0)
    }
}

impl Sub for f26_6 {
    type Output = Self;

    fn sub(self, b: f26_6) -> Self {
        f26_6(self.0 - b.0)
    }
}
