// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use super::{Num, NumCast};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size2D<T>
where
    T: Num,
{
    pub width: T,
    pub height: T,
}

impl<T> Size2D<T>
where
    T: Num,
{
    pub fn new(width: T, height: T) -> Size2D<T> {
        Size2D { width, height }
    }

    pub fn cast<U>(self) -> Size2D<U>
    where
        T: NumCast<U>,
        U: Num,
    {
        size2(self.width.cast(), self.height.cast())
    }
}

pub fn size2<T>(width: T, height: T) -> Size2D<T>
where
    T: Num,
{
    Size2D::new(width, height)
}
