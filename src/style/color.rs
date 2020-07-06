// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::convert::TryFrom;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Color {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
    pub(crate) a: u8,
}

impl Color {
    pub(crate) const fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }

    pub(crate) fn opacity(mut self, percentage: u8) -> Color {
        self.a = ((((self.a as u16) * (percentage as u16)) / 100) & 0xff) as u8;
        self
    }

    pub(crate) fn to_opengl_color(&self) -> (f32, f32, f32, f32) {
        (
            (self.r as f32) / 255.0,
            (self.g as f32) / 255.0,
            (self.b as f32) / 255.0,
            (self.a as f32) / 255.0,
        )
    }
}

impl TryFrom<&str> for Color {
    type Error = String;

    fn try_from(s: &str) -> Result<Color, Self::Error> {
        if (s.len() != 7 && s.len() != 9) || s.as_bytes()[0] != b'#' {
            return Err(format!("invalid hex-formatted color: {}", s));
        }
        let mut val = u32::from_str_radix(&s[1..], 16)
            .map_err(|_| format!("failed to parse hex-formatted color: {}", s))?;
        if s.len() == 7 {
            val = (val << 8) | 0xff;
        }
        Ok(Color {
            r: ((val >> 24) & 0xff) as u8,
            g: ((val >> 16) & 0xff) as u8,
            b: ((val >> 8) & 0xff) as u8,
            a: (val & 0xff) as u8,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(
            Color::try_from("#aabbcc").unwrap(),
            Color::new(0xaa, 0xbb, 0xcc, 0xff)
        );
        assert_eq!(
            Color::try_from("#aabbccdd").unwrap(),
            Color::new(0xaa, 0xbb, 0xcc, 0xdd)
        );
        assert!(Color::try_from("aabbcc").is_err());
        assert!(Color::try_from("#aabb").is_err());
        assert!(Color::try_from("#aabbccdde").is_err());
        assert!(Color::try_from("#xyzabc").is_err());
    }
}
