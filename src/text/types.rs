// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;
use std::ops::{Add, AddAssign, Mul, Sub};

// Freetype font sizes
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(non_camel_case_types)]
pub(crate) struct f26_6(i32);

impl From<f32> for f26_6 {
    fn from(f: f32) -> f26_6 {
        f26_6((f * 8.0).round() as i32)
    }
}

impl f26_6 {
    pub(super) fn from_raw(raw: i32) -> f26_6 {
        f26_6(raw >> 3)
    }

    pub(super) fn to_raw(self) -> i32 {
        self.0 << 3
    }

    pub(crate) fn to_f32(self) -> f32 {
        (self.0 as f32) / 8.0
    }

    pub(crate) fn floor(self) -> f26_6 {
        f26_6(self.0 & !7)
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
    fn add_assign(&mut self, rhs: f26_6) {
        self.0 += rhs.0
    }
}

impl Add for f26_6 {
    type Output = Self;

    fn add(self, rhs: f26_6) -> Self {
        f26_6(self.0 + rhs.0)
    }
}

impl Sub for f26_6 {
    type Output = Self;

    fn sub(self, rhs: f26_6) -> Self {
        f26_6(self.0 - rhs.0)
    }
}

impl Mul<f32> for f26_6 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        f26_6((self.0 as f32 * rhs).round() as i32)
    }
}
