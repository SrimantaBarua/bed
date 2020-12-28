// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::max;
use std::convert::TryFrom;
use std::default::Default;
use std::ops::{Add, AddAssign};

use serde::Deserialize;

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
        TextStyle { weight, slant }
    }
}

#[serde(from = "u16")]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub(crate) struct TextSize(pub(crate) u16);

impl TextSize {
    pub(crate) fn to_i64(self) -> i64 {
        self.0 as i64
    }

    pub(crate) fn to_f32(self) -> f32 {
        self.0 as f32
    }

    pub(crate) fn scale(self, scale: f64) -> TextSize {
        TextSize((self.0 as f64 * scale).round() as u16)
    }
}

impl From<u16> for TextSize {
    fn from(u: u16) -> TextSize {
        TextSize(u)
    }
}

impl Add<i16> for TextSize {
    type Output = TextSize;

    fn add(self, i: i16) -> TextSize {
        TextSize(max(self.0 as i16 + i, 0) as u16)
    }
}

impl AddAssign<i16> for TextSize {
    fn add_assign(&mut self, i: i16) {
        self.0 = max(self.0 as i16 + i, 0) as u16
    }
}
