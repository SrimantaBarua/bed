// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::{Num, NumCast};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector2D<T>
where
    T: Num,
{
    pub x: T,
    pub y: T,
}

impl<T> Vector2D<T>
where
    T: Num,
{
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

pub fn vec2<T>(x: T, y: T) -> Vector2D<T>
where
    T: Num,
{
    Vector2D::new(x, y)
}

impl<T> Add<Vector2D<T>> for Vector2D<T>
where
    T: Num,
{
    type Output = Vector2D<T>;

    fn add(self, vec: Vector2D<T>) -> Vector2D<T> {
        vec2(self.x + vec.x, self.y + vec.y)
    }
}

impl<T> AddAssign<Vector2D<T>> for Vector2D<T>
where
    T: Num,
{
    fn add_assign(&mut self, vec: Vector2D<T>) {
        self.x += vec.x;
        self.y += vec.y;
    }
}

impl<T> Sub<Vector2D<T>> for Vector2D<T>
where
    T: Num,
{
    type Output = Vector2D<T>;

    fn sub(self, vec: Vector2D<T>) -> Vector2D<T> {
        vec2(self.x - vec.x, self.y - vec.y)
    }
}

impl<T> SubAssign<Vector2D<T>> for Vector2D<T>
where
    T: Num,
{
    fn sub_assign(&mut self, vec: Vector2D<T>) {
        self.x -= vec.x;
        self.y -= vec.y;
    }
}
