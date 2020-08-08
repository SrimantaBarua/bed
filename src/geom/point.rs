// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::{vec2, Num, Vector2D};

#[derive(Debug)]
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

    pub(crate) fn to_vec2(self) -> Vector2D<T> {
        vec2(self.x, self.y)
    }
}

pub(crate) fn point2<T>(x: T, y: T) -> Point2D<T>
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
