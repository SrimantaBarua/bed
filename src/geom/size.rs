// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use super::Num;

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
}

pub(crate) fn size2<T>(width: T, height: T) -> Size2D<T>
where
    T: Num,
{
    Size2D::new(width, height)
}
