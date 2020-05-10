// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::default::Default;

use euclid::{size2, Size2D};

use crate::common::{PixelSize, DPI};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum TextWeight {
    Medium,
    Light,
    Bold,
}

impl TextWeight {
    /*
    pub(crate) fn from_str(s: &str) -> Option<TextWeight> {
        match s {
            "medium" => Some(TextWeight::Medium),
            "light" => Some(TextWeight::Light),
            "bold" => Some(TextWeight::Bold),
            _ => None,
        }
    }
    */
}

impl Default for TextWeight {
    fn default() -> TextWeight {
        TextWeight::Medium
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum TextSlant {
    Roman,
    Italic,
    Oblique,
}

impl TextSlant {
    /*
    pub(crate) fn from_str(s: &str) -> Option<TextSlant> {
        match s {
            "roman" => Some(TextSlant::Roman),
            "italic" => Some(TextSlant::Italic),
            "oblique" => Some(TextSlant::Oblique),
            _ => None,
        }
    }
    */
}

impl Default for TextSlant {
    fn default() -> TextSlant {
        TextSlant::Roman
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) struct TextStyle {
    pub(crate) weight: TextWeight,
    pub(crate) slant: TextSlant,
}

impl TextStyle {
    pub(crate) fn new(weight: TextWeight, slant: TextSlant) -> TextStyle {
        TextStyle {
            weight: weight,
            slant: slant,
        }
    }
}

// Text size in points
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub(crate) struct TextSize(u16);

impl TextSize {
    const SCALE: f32 = 4.0;
    const SHIFT: usize = 2;

    pub(crate) fn from_f32(f: f32) -> TextSize {
        TextSize((f * TextSize::SCALE) as u16)
    }

    pub(crate) fn to_f32(self) -> f32 {
        (self.0 as f32) / TextSize::SCALE
    }

    pub(crate) fn to_64th_point(self) -> i64 {
        (self.0 as i64) << (6 - TextSize::SHIFT)
    }

    pub(crate) fn to_pixel_size(self, dpi: Size2D<u32, DPI>) -> Size2D<f32, PixelSize> {
        let val = self.to_f32() / 72.0;
        size2(val * dpi.width as f32, val * dpi.height as f32)
    }
}
