// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use super::{Num, NumCast};

#[derive(Clone, Copy, PartialEq)]
pub struct Size2D<T: Num> {
    pub width: T,
    pub height: T,
}

impl<T: Num> Size2D<T> {
    pub fn new(width: T, height: T) -> Size2D<T> {
        Size2D { width, height }
    }

    pub fn cast<U: Num>(self) -> Size2D<U>
    where
        T: NumCast<U>,
    {
        size2(self.width.cast(), self.height.cast())
    }
}

pub fn size2<T: Num>(width: T, height: T) -> Size2D<T> {
    Size2D::new(width, height)
}

impl<T: Num + fmt::Debug> fmt::Debug for Size2D<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}x{:?}", self.width, self.height)
    }
}
