// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::features::*;

// Text direction
#[derive(Debug)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

impl Direction {
    pub(crate) fn features(&self) -> &'static [Features] {
        match self {
            Direction::LeftToRight => &HORIZONTAL_FEATURES,
            Direction::RightToLeft => &HORIZONTAL_FEATURES,
            Direction::TopToBottom => &VERTICAL_FEATURES,
            Direction::BottomToTop => &VERTICAL_FEATURES,
        }
    }
}
