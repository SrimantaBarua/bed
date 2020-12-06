// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::{vec2, Num, NumCast, Vector2D};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2D<T>
where
    T: Num,
{
    pub x: T,
    pub y: T,
}

impl<T> Point2D<T>
where
    T: Num,
{
    pub fn new(x: T, y: T) -> Point2D<T> {
        Point2D { x, y }
    }

    pub fn to_vec2(self) -> Vector2D<T> {
        vec2(self.x, self.y)
    }

    pub fn cast<U>(self) -> Point2D<U>
    where
        T: NumCast<U>,
        U: Num,
    {
        point2(self.x.cast(), self.y.cast())
    }
}

pub fn point2<T>(x: T, y: T) -> Point2D<T>
where
    T: Num,
{
    Point2D::new(x, y)
}

impl<T> Add<Vector2D<T>> for Point2D<T>
where
    T: Num,
{
    type Output = Point2D<T>;

    fn add(self, vec: Vector2D<T>) -> Point2D<T> {
        point2(self.x + vec.x, self.y + vec.y)
    }
}

impl<T> AddAssign<Vector2D<T>> for Point2D<T>
where
    T: Num,
{
    fn add_assign(&mut self, vec: Vector2D<T>) {
        self.x += vec.x;
        self.y += vec.y;
    }
}

impl<T> Sub<Vector2D<T>> for Point2D<T>
where
    T: Num,
{
    type Output = Point2D<T>;

    fn sub(self, vec: Vector2D<T>) -> Point2D<T> {
        point2(self.x - vec.x, self.y - vec.y)
    }
}

impl<T> SubAssign<Vector2D<T>> for Point2D<T>
where
    T: Num,
{
    fn sub_assign(&mut self, vec: Vector2D<T>) {
        self.x -= vec.x;
        self.y -= vec.y;
    }
}
