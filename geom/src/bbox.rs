// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use super::{Num, NumCast, Point2D};

#[derive(Clone, Copy, PartialEq)]
pub struct BBox<T: Num> {
    pub min: Point2D<T>,
    pub max: Point2D<T>,
}

impl<T: Num> BBox<T> {
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

    pub fn cast<U: Num>(self) -> BBox<U>
    where
        T: NumCast<U>,
    {
        bbox(self.min.cast(), self.max.cast())
    }
}

pub fn bbox<T: Num>(min: Point2D<T>, max: Point2D<T>) -> BBox<T> {
    BBox::new(min, max)
}

impl<T: Num + fmt::Debug> fmt::Debug for BBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BBox(({:?}), ({:?}))", self.min, self.max)
    }
}
