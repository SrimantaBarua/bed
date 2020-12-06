// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use super::{Num, NumCast, Point2D};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BBox<T>
where
    T: Num,
{
    pub min: Point2D<T>,
    pub max: Point2D<T>,
}

impl<T> BBox<T>
where
    T: Num,
{
    pub fn new(min: Point2D<T>, max: Point2D<T>) -> BBox<T> {
        BBox { min, max }
    }

    pub fn height(&self) -> T {
        self.max.y - self.min.y
    }

    pub fn width(&self) -> T {
        self.max.x - self.min.x
    }

    pub fn area(&self) -> T {
        self.height() * self.width()
    }

    pub fn cast<U>(self) -> BBox<U>
    where
        T: NumCast<U>,
        U: Num,
    {
        bbox(self.min.cast(), self.max.cast())
    }
}

pub fn bbox<T>(min: Point2D<T>, max: Point2D<T>) -> BBox<T>
where
    T: Num,
{
    BBox::new(min, max)
}
