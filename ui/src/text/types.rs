use std::fmt;
use std::ops::{Add, AddAssign, Mul, Sub};

/// Text size - a 33.3 bit signed floating point number.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(non_camel_case_types)]
pub(crate) struct TextUnit(i32);

impl From<f32> for TextUnit {
    fn from(f: f32) -> TextUnit {
        TextUnit((f * 8.0).round() as i32)
    }
}

impl TextUnit {
    pub(crate) fn to_f32(self) -> f32 {
        self.0 as f32 * 8.0
    }

    pub(crate) fn floor(self) -> TextUnit {
        TextUnit(self.0 & !7)
    }

    pub(super) fn from_freetype(raw: i32) -> TextUnit {
        TextUnit(raw >> 3)
    }

    pub(super) fn to_freetype(self) -> i32 {
        self.0 << 3
    }
}

impl fmt::Display for TextUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_f32())
    }
}

impl fmt::Debug for TextUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_f32())
    }
}

impl AddAssign for TextUnit {
    fn add_assign(&mut self, rhs: TextUnit) {
        self.0 += rhs.0
    }
}

impl Add for TextUnit {
    type Output = Self;

    fn add(self, rhs: TextUnit) -> Self {
        TextUnit(self.0 + rhs.0)
    }
}

impl Sub for TextUnit {
    type Output = Self;

    fn sub(self, rhs: TextUnit) -> Self {
        TextUnit(self.0 - rhs.0)
    }
}

impl Mul<f32> for TextUnit {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        TextUnit((self.0 as f32 * rhs).round() as i32)
    }
}
