// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ops::{Add, Mul, Sub};

mod point;
mod rect;
mod size;

pub(crate) use point::*;
pub(crate) use rect::*;
pub(crate) use size::*;

pub(crate) trait Num:
    Clone + Copy + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + PartialEq + PartialOrd
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
