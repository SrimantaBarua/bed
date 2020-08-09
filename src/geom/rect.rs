// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use super::{Num, NumCast, Point2D, Size2D};

#[derive(Debug)]
pub(crate) struct Rect<T>
where
    T: Num,
{
    pub(crate) origin: Point2D<T>,
    pub(crate) size: Size2D<T>,
}

impl<T> Rect<T>
where
    T: Num,
{
    pub(crate) fn new(origin: Point2D<T>, size: Size2D<T>) -> Rect<T> {
        Rect { origin, size }
    }

    pub(crate) fn area(&self) -> T {
        self.size.width * self.size.height
    }

    pub(crate) fn cast<U>(self) -> Rect<U>
    where
        T: NumCast<U>,
        U: Num,
    {
        rect(self.origin.cast(), self.size.cast())
    }
}

pub(crate) fn rect<T>(origin: Point2D<T>, size: Size2D<T>) -> Rect<T>
where
    T: Num,
{
    Rect::new(origin, size)
}
