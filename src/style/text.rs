// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::convert::TryFrom;
use std::default::Default;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub(crate) struct TextSize(pub(crate) u16);

impl TextSize {
    pub(crate) fn to_i64(self) -> i64 {
        self.0 as i64
    }

    pub(crate) fn to_f32(self) -> f32 {
        self.0 as f32
    }

    pub(crate) fn scale(self, scale: f32) -> TextSize {
        TextSize((self.0 as f32 * scale).round() as u16)
    }
}
