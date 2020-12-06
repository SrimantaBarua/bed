// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use super::{Num, NumCast, Point2D, Size2D};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect<T>
where
    T: Num,
{
    pub origin: Point2D<T>,
    pub size: Size2D<T>,
}

impl<T> Rect<T>
where
    T: Num,
{
    pub fn new(origin: Point2D<T>, size: Size2D<T>) -> Rect<T> {
        Rect { origin, size }
    }

    pub fn area(&self) -> T {
        self.size.width * self.size.height
    }

    pub fn cast<U>(self) -> Rect<U>
    where
        T: NumCast<U>,
        U: Num,
    {
        rect(self.origin.cast(), self.size.cast())
    }
}

pub fn rect<T>(origin: Point2D<T>, size: Size2D<T>) -> Rect<T>
where
    T: Num,
{
    Rect::new(origin, size)
}
