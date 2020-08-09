// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use super::{Num, NumCast};

#[derive(Debug)]
pub(crate) struct Size2D<T>
where
    T: Num,
{
    pub(crate) width: T,
    pub(crate) height: T,
}

impl<T> Size2D<T>
where
    T: Num,
{
    pub(crate) fn new(width: T, height: T) -> Size2D<T> {
        Size2D { width, height }
    }

    pub(crate) fn cast<U>(self) -> Size2D<U>
    where
        T: NumCast<U>,
        U: Num,
    {
        size2(self.width.cast(), self.height.cast())
    }
}

pub(crate) fn size2<T>(width: T, height: T) -> Size2D<T>
where
    T: Num,
{
    Size2D::new(width, height)
}
