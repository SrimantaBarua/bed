// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use geom::{size2, vec2, Num, NumCast, Size2D, Vector2D};

/// Wrapper around glyphs
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct GlyphID(pub(crate) u32);

impl fmt::Debug for GlyphID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Shaped glyph information
#[derive(Debug)]
pub(crate) struct GlyphInfo {
    pub(crate) glyph: GlyphID,
    pub(crate) bearing: Vector2D<i16>,
    pub(crate) size: Size2D<u16>,
    pub(crate) offset: Vector2D<i16>,
    pub(crate) advance: Vector2D<u16>,
}

impl GlyphInfo {
    // Scale everything
    pub(crate) fn scale(&self, scale: Size2D<f32>) -> ScaledGlyphInfo {
        let bearing = vec2(
            self.bearing.x as f32 * scale.width,
            self.bearing.y as f32 * scale.height,
        )
        .cast();
        let size = size2(
            self.size.width as f32 * scale.width,
            self.size.height as f32 * scale.height,
        )
        .cast();
        let offset = vec2(
            self.offset.x as f32 * scale.width,
            self.offset.y as f32 * scale.height,
        )
        .cast();
        let advance = vec2(
            self.advance.x as f32 * scale.width,
            self.advance.y as f32 * scale.height,
        )
        .cast();
        ScaledGlyphInfo {
            glyph: self.glyph,
            bearing,
            size,
            offset,
            advance,
        }
    }
}

/// Scaled glyph information
#[derive(Debug)]
pub struct ScaledGlyphInfo {
    pub glyph: GlyphID,
    pub bearing: Vector2D<F26p6>,
    pub size: Size2D<F26p6>,
    pub offset: Vector2D<F26p6>,
    pub advance: Vector2D<F26p6>,
}

/// Imitiate floating point number with 6 bits of decimal precision
#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct F26p6(i32);

impl Add for F26p6 {
    type Output = Self;

    fn add(self, other: Self) -> F26p6 {
        F26p6(self.0 + other.0)
    }
}

impl AddAssign for F26p6 {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0
    }
}

impl Sub for F26p6 {
    type Output = Self;

    fn sub(self, other: Self) -> F26p6 {
        F26p6(self.0 - other.0)
    }
}

impl SubAssign for F26p6 {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0
    }
}

impl Mul for F26p6 {
    type Output = Self;

    fn mul(self, other: Self) -> F26p6 {
        F26p6((((self.0 as i64) * (other.0 as i64)) >> 6) as i32)
    }
}

impl Num for F26p6 {}

impl NumCast<F26p6> for i8 {
    fn cast(self) -> F26p6 {
        F26p6::from_raw_int(self as i16)
    }
}

impl NumCast<F26p6> for i16 {
    fn cast(self) -> F26p6 {
        F26p6::from_raw_int(self)
    }
}

impl NumCast<F26p6> for i32 {
    fn cast(self) -> F26p6 {
        F26p6::from_raw_int(self as i16)
    }
}

impl NumCast<F26p6> for i64 {
    fn cast(self) -> F26p6 {
        F26p6::from_raw_int(self as i16)
    }
}

impl NumCast<F26p6> for u8 {
    fn cast(self) -> F26p6 {
        F26p6::from_raw_int(self as i16)
    }
}

impl NumCast<F26p6> for u16 {
    fn cast(self) -> F26p6 {
        F26p6::from_raw_int(self as i16)
    }
}

impl NumCast<F26p6> for u32 {
    fn cast(self) -> F26p6 {
        F26p6::from_raw_int(self as i16)
    }
}

impl NumCast<F26p6> for u64 {
    fn cast(self) -> F26p6 {
        F26p6::from_raw_int(self as i16)
    }
}

impl NumCast<F26p6> for f32 {
    fn cast(self) -> F26p6 {
        F26p6::from_raw_float(self)
    }
}

impl NumCast<F26p6> for f64 {
    fn cast(self) -> F26p6 {
        F26p6::from_raw_float(self as f32)
    }
}

impl NumCast<i8> for F26p6 {
    fn cast(self) -> i8 {
        self.int_part() as i8
    }
}

impl NumCast<i16> for F26p6 {
    fn cast(self) -> i16 {
        self.int_part() as i16
    }
}

impl NumCast<i32> for F26p6 {
    fn cast(self) -> i32 {
        self.int_part()
    }
}

impl NumCast<i64> for F26p6 {
    fn cast(self) -> i64 {
        self.int_part() as i64
    }
}

impl NumCast<u8> for F26p6 {
    fn cast(self) -> u8 {
        self.int_part() as u8
    }
}

impl NumCast<u16> for F26p6 {
    fn cast(self) -> u16 {
        self.int_part() as u16
    }
}

impl NumCast<u32> for F26p6 {
    fn cast(self) -> u32 {
        self.int_part() as u32
    }
}

impl NumCast<u64> for F26p6 {
    fn cast(self) -> u64 {
        self.int_part() as u64
    }
}

impl NumCast<f32> for F26p6 {
    fn cast(self) -> f32 {
        self.to_f32()
    }
}

impl NumCast<f64> for F26p6 {
    fn cast(self) -> f64 {
        self.to_f32() as f64
    }
}

impl fmt::Debug for F26p6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.int_part(), self.frac_part())
    }
}

impl F26p6 {
    pub fn to_f32(self) -> f32 {
        (self.0 as f32) / 64.0
    }

    fn from_raw_int(i: i16) -> F26p6 {
        F26p6((i as i32) << 6)
    }

    fn from_raw_float(f: f32) -> F26p6 {
        F26p6((f * 64.0).round() as i32)
    }

    fn int_part(self) -> i32 {
        (self.0 >> 6) & 0x3ffffff
    }

    fn frac_part(self) -> i32 {
        self.0 & 0x3f
    }
}
