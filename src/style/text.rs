// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::convert::TryFrom;
use std::default::Default;

use euclid::{size2, Size2D};
use serde::Deserialize;

use crate::common::{PixelSize, DPI};

#[serde(try_from = "&str")]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
pub(crate) enum TextWeight {
    Medium,
    Light,
    Bold,
}

impl TryFrom<&str> for TextWeight {
    type Error = String;

    fn try_from(s: &str) -> Result<TextWeight, Self::Error> {
        match s {
            "medium" => Ok(TextWeight::Medium),
            "light" => Ok(TextWeight::Light),
            "bold" => Ok(TextWeight::Bold),
            _ => Err(format!("invalid text weight: {}", s)),
        }
    }
}

impl Default for TextWeight {
    fn default() -> TextWeight {
        TextWeight::Medium
    }
}

#[serde(try_from = "&str")]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
pub(crate) enum TextSlant {
    Roman,
    Italic,
    Oblique,
}

impl TryFrom<&str> for TextSlant {
    type Error = String;

    fn try_from(s: &str) -> Result<TextSlant, Self::Error> {
        match s {
            "roman" => Ok(TextSlant::Roman),
            "italic" => Ok(TextSlant::Italic),
            "oblique" => Ok(TextSlant::Oblique),
            _ => Err(format!("invalid text slant: {}", s)),
        }
    }
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
