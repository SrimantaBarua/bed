// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::{Num, NumCast};

#[derive(Clone, Copy, PartialEq)]
pub struct Vector2D<T: Num> {
    pub x: T,
    pub y: T,
}

impl<T: Num> Vector2D<T> {
    pub fn new(x: T, y: T) -> Vector2D<T> {
        Vector2D { x, y }
    }

    pub fn cast<U>(self) -> Vector2D<U>
    where
        T: NumCast<U>,
        U: Num,
    {
        vec2(self.x.cast(), self.y.cast())
    }
}

pub fn vec2<T: Num>(x: T, y: T) -> Vector2D<T> {
    Vector2D::new(x, y)
}

impl<T: Num> Add<Vector2D<T>> for Vector2D<T> {
    type Output = Vector2D<T>;

    fn add(self, vec: Vector2D<T>) -> Vector2D<T> {
        vec2(self.x + vec.x, self.y + vec.y)
    }
}

impl<T: Num> AddAssign<Vector2D<T>> for Vector2D<T> {
    fn add_assign(&mut self, vec: Vector2D<T>) {
        self.x += vec.x;
        self.y += vec.y;
    }
}

impl<T: Num> Sub<Vector2D<T>> for Vector2D<T> {
    type Output = Vector2D<T>;

    fn sub(self, vec: Vector2D<T>) -> Vector2D<T> {
        vec2(self.x - vec.x, self.y - vec.y)
    }
}

impl<T: Num> SubAssign<Vector2D<T>> for Vector2D<T> {
    fn sub_assign(&mut self, vec: Vector2D<T>) {
        self.x -= vec.x;
        self.y -= vec.y;
    }
}

impl<T: Num + fmt::Debug> fmt::Debug for Vector2D<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?},{:?})", self.x, self.y)
    }
}
