// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::{vec2, Num, NumCast, Vector2D};

#[derive(Clone, Copy, PartialEq)]
pub struct Point2D<T: Num> {
    pub x: T,
    pub y: T,
}

impl<T: Num> Point2D<T> {
    pub fn new(x: T, y: T) -> Point2D<T> {
        Point2D { x, y }
    }

    pub fn to_vec2(self) -> Vector2D<T> {
        vec2(self.x, self.y)
    }

    pub fn cast<U: Num>(self) -> Point2D<U>
    where
        T: NumCast<U>,
    {
        point2(self.x.cast(), self.y.cast())
    }
}

pub fn point2<T: Num>(x: T, y: T) -> Point2D<T> {
    Point2D::new(x, y)
}

impl<T: Num> Add<Vector2D<T>> for Point2D<T> {
    type Output = Point2D<T>;

    fn add(self, vec: Vector2D<T>) -> Point2D<T> {
        point2(self.x + vec.x, self.y + vec.y)
    }
}

impl<T: Num> AddAssign<Vector2D<T>> for Point2D<T> {
    fn add_assign(&mut self, vec: Vector2D<T>) {
        self.x += vec.x;
        self.y += vec.y;
    }
}

impl<T: Num> Sub<Vector2D<T>> for Point2D<T> {
    type Output = Point2D<T>;

    fn sub(self, vec: Vector2D<T>) -> Point2D<T> {
        point2(self.x - vec.x, self.y - vec.y)
    }
}

impl<T: Num> SubAssign<Vector2D<T>> for Point2D<T> {
    fn sub_assign(&mut self, vec: Vector2D<T>) {
        self.x -= vec.x;
        self.y -= vec.y;
    }
}

impl<T: Num + fmt::Debug> fmt::Debug for Point2D<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?},{:?})", self.x, self.y)
    }
}
