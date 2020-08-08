// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use super::Num;

pub(crate) struct Point2D<T>
where
    T: Num,
{
    pub(crate) x: T,
    pub(crate) y: T,
}

impl<T> Point2D<T>
where
    T: Num,
{
    pub(crate) fn new(x: T, y: T) -> Point2D<T> {
        Point2D { x, y }
    }
}

pub(crate) fn point2<T>(x: T, y: T) -> Point2D<T>
where
    T: Num,
{
    Point2D::new(x, y)
}
