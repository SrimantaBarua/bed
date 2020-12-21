// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use super::{Num, NumCast, Point2D, Size2D};

#[derive(Clone, Copy, PartialEq)]
pub struct Rect<T: Num> {
    pub origin: Point2D<T>,
    pub size: Size2D<T>,
}

impl<T: Num> Rect<T> {
    pub fn new(origin: Point2D<T>, size: Size2D<T>) -> Rect<T> {
        Rect { origin, size }
    }

    pub fn area(&self) -> T {
        self.size.width * self.size.height
    }

    pub fn cast<U: Num>(self) -> Rect<U>
    where
        T: NumCast<U>,
    {
        rect(self.origin.cast(), self.size.cast())
    }
}

pub fn rect<T: Num>(origin: Point2D<T>, size: Size2D<T>) -> Rect<T> {
    Rect::new(origin, size)
}

impl<T: Num + fmt::Debug> fmt::Debug for Rect<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Rect({:?}, {:?})", self.origin, self.size)
    }
}
