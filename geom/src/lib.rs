// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

mod bbox;
mod point;
mod rect;
mod size;
mod vector;

pub use bbox::*;
pub use point::*;
pub use rect::*;
pub use size::*;
pub use vector::*;

pub trait Num:
    Clone
    + Copy
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<Output = Self>
    + PartialEq
    + PartialOrd
{
}

pub trait NumCast<T>: Num {
    fn cast(self) -> T;
}

impl Num for i8 {}
impl Num for i16 {}
impl Num for i32 {}
impl Num for i64 {}
impl Num for u8 {}
impl Num for u16 {}
impl Num for u32 {}
impl Num for u64 {}
impl Num for f32 {}
impl Num for f64 {}

impl NumCast<i8> for i8 {
    fn cast(self) -> i8 {
        self as i8
    }
}
impl NumCast<i8> for i16 {
    fn cast(self) -> i8 {
        self as i8
    }
}
impl NumCast<i8> for i32 {
    fn cast(self) -> i8 {
        self as i8
    }
}
impl NumCast<i8> for i64 {
    fn cast(self) -> i8 {
        self as i8
    }
}
impl NumCast<i8> for u8 {
    fn cast(self) -> i8 {
        self as i8
    }
}
impl NumCast<i8> for u16 {
    fn cast(self) -> i8 {
        self as i8
    }
}
impl NumCast<i8> for u32 {
    fn cast(self) -> i8 {
        self as i8
    }
}
impl NumCast<i8> for u64 {
    fn cast(self) -> i8 {
        self as i8
    }
}
impl NumCast<i8> for f32 {
    fn cast(self) -> i8 {
        self as i8
    }
}
impl NumCast<i8> for f64 {
    fn cast(self) -> i8 {
        self as i8
    }
}

impl NumCast<i16> for i8 {
    fn cast(self) -> i16 {
        self as i16
    }
}
impl NumCast<i16> for i16 {
    fn cast(self) -> i16 {
        self as i16
    }
}
impl NumCast<i16> for i32 {
    fn cast(self) -> i16 {
        self as i16
    }
}
impl NumCast<i16> for i64 {
    fn cast(self) -> i16 {
        self as i16
    }
}
impl NumCast<i16> for u8 {
    fn cast(self) -> i16 {
        self as i16
    }
}
impl NumCast<i16> for u16 {
    fn cast(self) -> i16 {
        self as i16
    }
}
impl NumCast<i16> for u32 {
    fn cast(self) -> i16 {
        self as i16
    }
}
impl NumCast<i16> for u64 {
    fn cast(self) -> i16 {
        self as i16
    }
}
impl NumCast<i16> for f32 {
    fn cast(self) -> i16 {
        self as i16
    }
}
impl NumCast<i16> for f64 {
    fn cast(self) -> i16 {
        self as i16
    }
}

impl NumCast<i32> for i8 {
    fn cast(self) -> i32 {
        self as i32
    }
}
impl NumCast<i32> for i16 {
    fn cast(self) -> i32 {
        self as i32
    }
}
impl NumCast<i32> for i32 {
    fn cast(self) -> i32 {
        self as i32
    }
}
impl NumCast<i32> for i64 {
    fn cast(self) -> i32 {
        self as i32
    }
}
impl NumCast<i32> for u8 {
    fn cast(self) -> i32 {
        self as i32
    }
}
impl NumCast<i32> for u16 {
    fn cast(self) -> i32 {
        self as i32
    }
}
impl NumCast<i32> for u32 {
    fn cast(self) -> i32 {
        self as i32
    }
}
impl NumCast<i32> for u64 {
    fn cast(self) -> i32 {
        self as i32
    }
}
impl NumCast<i32> for f32 {
    fn cast(self) -> i32 {
        self as i32
    }
}
impl NumCast<i32> for f64 {
    fn cast(self) -> i32 {
        self as i32
    }
}

impl NumCast<i64> for i8 {
    fn cast(self) -> i64 {
        self as i64
    }
}
impl NumCast<i64> for i16 {
    fn cast(self) -> i64 {
        self as i64
    }
}
impl NumCast<i64> for i32 {
    fn cast(self) -> i64 {
        self as i64
    }
}
impl NumCast<i64> for i64 {
    fn cast(self) -> i64 {
        self as i64
    }
}
impl NumCast<i64> for u8 {
    fn cast(self) -> i64 {
        self as i64
    }
}
impl NumCast<i64> for u16 {
    fn cast(self) -> i64 {
        self as i64
    }
}
impl NumCast<i64> for u32 {
    fn cast(self) -> i64 {
        self as i64
    }
}
impl NumCast<i64> for u64 {
    fn cast(self) -> i64 {
        self as i64
    }
}
impl NumCast<i64> for f32 {
    fn cast(self) -> i64 {
        self as i64
    }
}
impl NumCast<i64> for f64 {
    fn cast(self) -> i64 {
        self as i64
    }
}

impl NumCast<u8> for i8 {
    fn cast(self) -> u8 {
        self as u8
    }
}
impl NumCast<u8> for i16 {
    fn cast(self) -> u8 {
        self as u8
    }
}
impl NumCast<u8> for i32 {
    fn cast(self) -> u8 {
        self as u8
    }
}
impl NumCast<u8> for i64 {
    fn cast(self) -> u8 {
        self as u8
    }
}
impl NumCast<u8> for u8 {
    fn cast(self) -> u8 {
        self as u8
    }
}
impl NumCast<u8> for u16 {
    fn cast(self) -> u8 {
        self as u8
    }
}
impl NumCast<u8> for u32 {
    fn cast(self) -> u8 {
        self as u8
    }
}
impl NumCast<u8> for u64 {
    fn cast(self) -> u8 {
        self as u8
    }
}
impl NumCast<u8> for f32 {
    fn cast(self) -> u8 {
        self as u8
    }
}
impl NumCast<u8> for f64 {
    fn cast(self) -> u8 {
        self as u8
    }
}

impl NumCast<u16> for i8 {
    fn cast(self) -> u16 {
        self as u16
    }
}
impl NumCast<u16> for i16 {
    fn cast(self) -> u16 {
        self as u16
    }
}
impl NumCast<u16> for i32 {
    fn cast(self) -> u16 {
        self as u16
    }
}
impl NumCast<u16> for i64 {
    fn cast(self) -> u16 {
        self as u16
    }
}
impl NumCast<u16> for u8 {
    fn cast(self) -> u16 {
        self as u16
    }
}
impl NumCast<u16> for u16 {
    fn cast(self) -> u16 {
        self as u16
    }
}
impl NumCast<u16> for u32 {
    fn cast(self) -> u16 {
        self as u16
    }
}
impl NumCast<u16> for u64 {
    fn cast(self) -> u16 {
        self as u16
    }
}
impl NumCast<u16> for f32 {
    fn cast(self) -> u16 {
        self as u16
    }
}
impl NumCast<u16> for f64 {
    fn cast(self) -> u16 {
        self as u16
    }
}

impl NumCast<u32> for i8 {
    fn cast(self) -> u32 {
        self as u32
    }
}
impl NumCast<u32> for i16 {
    fn cast(self) -> u32 {
        self as u32
    }
}
impl NumCast<u32> for i32 {
    fn cast(self) -> u32 {
        self as u32
    }
}
impl NumCast<u32> for i64 {
    fn cast(self) -> u32 {
        self as u32
    }
}
impl NumCast<u32> for u8 {
    fn cast(self) -> u32 {
        self as u32
    }
}
impl NumCast<u32> for u16 {
    fn cast(self) -> u32 {
        self as u32
    }
}
impl NumCast<u32> for u32 {
    fn cast(self) -> u32 {
        self as u32
    }
}
impl NumCast<u32> for u64 {
    fn cast(self) -> u32 {
        self as u32
    }
}
impl NumCast<u32> for f32 {
    fn cast(self) -> u32 {
        self as u32
    }
}
impl NumCast<u32> for f64 {
    fn cast(self) -> u32 {
        self as u32
    }
}

impl NumCast<u64> for i8 {
    fn cast(self) -> u64 {
        self as u64
    }
}
impl NumCast<u64> for i16 {
    fn cast(self) -> u64 {
        self as u64
    }
}
impl NumCast<u64> for i32 {
    fn cast(self) -> u64 {
        self as u64
    }
}
impl NumCast<u64> for i64 {
    fn cast(self) -> u64 {
        self as u64
    }
}
impl NumCast<u64> for u8 {
    fn cast(self) -> u64 {
        self as u64
    }
}
impl NumCast<u64> for u16 {
    fn cast(self) -> u64 {
        self as u64
    }
}
impl NumCast<u64> for u32 {
    fn cast(self) -> u64 {
        self as u64
    }
}
impl NumCast<u64> for u64 {
    fn cast(self) -> u64 {
        self as u64
    }
}
impl NumCast<u64> for f32 {
    fn cast(self) -> u64 {
        self as u64
    }
}
impl NumCast<u64> for f64 {
    fn cast(self) -> u64 {
        self as u64
    }
}

impl NumCast<f32> for i8 {
    fn cast(self) -> f32 {
        self as f32
    }
}
impl NumCast<f32> for i16 {
    fn cast(self) -> f32 {
        self as f32
    }
}
impl NumCast<f32> for i32 {
    fn cast(self) -> f32 {
        self as f32
    }
}
impl NumCast<f32> for i64 {
    fn cast(self) -> f32 {
        self as f32
    }
}
impl NumCast<f32> for u8 {
    fn cast(self) -> f32 {
        self as f32
    }
}
impl NumCast<f32> for u16 {
    fn cast(self) -> f32 {
        self as f32
    }
}
impl NumCast<f32> for u32 {
    fn cast(self) -> f32 {
        self as f32
    }
}
impl NumCast<f32> for u64 {
    fn cast(self) -> f32 {
        self as f32
    }
}
impl NumCast<f32> for f32 {
    fn cast(self) -> f32 {
        self as f32
    }
}
impl NumCast<f32> for f64 {
    fn cast(self) -> f32 {
        self as f32
    }
}

impl NumCast<f64> for i8 {
    fn cast(self) -> f64 {
        self as f64
    }
}
impl NumCast<f64> for i16 {
    fn cast(self) -> f64 {
        self as f64
    }
}
impl NumCast<f64> for i32 {
    fn cast(self) -> f64 {
        self as f64
    }
}
impl NumCast<f64> for i64 {
    fn cast(self) -> f64 {
        self as f64
    }
}
impl NumCast<f64> for u8 {
    fn cast(self) -> f64 {
        self as f64
    }
}
impl NumCast<f64> for u16 {
    fn cast(self) -> f64 {
        self as f64
    }
}
impl NumCast<f64> for u32 {
    fn cast(self) -> f64 {
        self as f64
    }
}
impl NumCast<f64> for u64 {
    fn cast(self) -> f64 {
        self as f64
    }
}
impl NumCast<f64> for f32 {
    fn cast(self) -> f64 {
        self as f64
    }
}
impl NumCast<f64> for f64 {
    fn cast(self) -> f64 {
        self as f64
    }
}
