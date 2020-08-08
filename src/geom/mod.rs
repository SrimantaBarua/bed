// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

mod point;
mod rect;
mod size;
mod vector;

pub(crate) use point::*;
pub(crate) use rect::*;
pub(crate) use size::*;
pub(crate) use vector::*;

pub(crate) trait Num:
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
