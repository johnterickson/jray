use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Vec3(pub f64, pub f64, pub f64);

impl Debug for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:.2}, {:.2}, {:.2}]", self.0, self.1, self.2)
    }
}

impl Vec3 {
    pub fn magnitude(&self) -> f64 {
        (self.0 * self.0 + self.1 * self.1 + self.2 * self.2).sqrt()
    }

    pub fn normalize(&mut self) -> () {
        let mag = self.magnitude();
        self.0 /= mag;
        self.1 /= mag;
        self.2 /= mag;
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        let mut sum = self;
        sum += rhs;
        sum
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
        self.2 -= rhs.2;
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result -= rhs;
        result
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.0 *= rhs;
        self.1 *= rhs;
        self.2 *= rhs;
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Self::Output {
        let mut result = self;
        result *= rhs;
        result
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3(self * rhs.0, self * rhs.1, self * rhs.2)
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Point(pub Vec3);

impl Point {
    pub const fn origin() -> Point {
        Point(Vec3(0.0, 0.0, 0.0))
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point:{:?}", self.0)
    }
}

impl Add<Direction> for Point {
    type Output = Point;

    fn add(self, rhs: Direction) -> Self::Output {
        Point(self.0 + rhs.0)
    }
}

impl Sub<Direction> for Point {
    type Output = Point;

    fn sub(self, rhs: Direction) -> Self::Output {
        Point(self.0 - rhs.0)
    }
}

impl Sub for Point {
    type Output = Direction;

    fn sub(self, rhs: Self) -> Self::Output {
        Direction(self.0 - rhs.0)
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Direction(pub Vec3);

impl Debug for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Direction:{:?}", self.0)
    }
}

impl Direction {
    pub const fn none() -> Self {
        Direction(Vec3(0.0, 0.0, 0.0))
    }

    pub fn dot(&self, rhs: &Self) -> f64 {
        self.0 .0 * rhs.0 .0 + self.0 .1 * rhs.0 .1 + self.0 .2 * rhs.0 .2
    }

    pub fn cross(&self, rhs: &Self) -> Self {
        Direction(Vec3(
            self.0 .1 * rhs.0 .2 - self.0 .2 * rhs.0 .1,
            self.0 .2 * rhs.0 .0 - self.0 .0 * rhs.0 .2,
            self.0 .0 * rhs.0 .1 - self.0 .1 * rhs.0 .0,
        ))
    }

    pub fn reflect(&self, normal: &Self) -> Self {
        *self - 2.0*(self.dot(normal))*normal
    }

    pub fn mirror(&self, normal: &Self) -> Self {
        // https://mathworld.wolfram.com/Reflection.html
        // dbg!(&self);
        // dbg!(normal);
        let xn = self.dot(normal) * normal;
        // dbg!(xn);
        let to_normal = xn - *self;
        // dbg!(to_normal);
        *self + 2.0 * to_normal
    }

    pub fn normalize(&mut self) -> () {
        let mag = self.0.magnitude();
        self.0 .0 /= mag;
        self.0 .1 /= mag;
        self.0 .2 /= mag;
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }
}

impl Add<Direction> for Direction {
    type Output = Self;

    fn add(self, rhs: Direction) -> Self::Output {
        Direction(self.0 + rhs.0)
    }
}

impl Sub<Direction> for Direction {
    type Output = Self;

    fn sub(self, rhs: Direction) -> Self::Output {
        Direction(self.0 - rhs.0)
    }
}

impl Mul<f64> for Direction {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Direction(rhs * self.0)
    }
}

impl Mul<Direction> for f64 {
    type Output = Direction;

    fn mul(self, rhs: Direction) -> Self::Output {
        self * (&rhs)
    }
}

impl Mul<&Direction> for f64 {
    type Output = Direction;

    fn mul(self, rhs: &Direction) -> Self::Output {
        Direction(self * (rhs.0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Ray(pub Point, pub Direction);

impl Ray {
    pub fn from_points(from: Point, to: Point) -> Ray {
        Ray(from, Direction((to.0 - from.0).normalized()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add() {
        assert_eq!(
            Vec3(1.0, 2.0, 3.0) + Vec3(4.0, 6.0, 9.0),
            Vec3(5.0, 8.0, 12.0)
        );
    }

    #[test]
    fn cross() {
        assert_eq!(
            Direction(Vec3(1.0, 0.0, 0.0)).cross(&Direction(Vec3(0.0, 1.0, 0.0))),
            Direction(Vec3(0.0, 0.0, 1.0))
        );
    }

    #[test]
    fn mirror() {
        assert_eq!(
            Direction(Vec3(-1.0, 0.0, 0.0)).normalized().mirror(&Direction(Vec3(0.0, 1.0, 0.0))),
            Direction(Vec3(1.0, 0.0, 0.0)).normalized()
        );

        assert_eq!(
            Direction(Vec3(-1.0, -1.0, 1.0)).normalized().mirror(&Direction(Vec3(0.0, 0.0, 1.0))),
            Direction(Vec3(1.0, 1.0, 1.0)).normalized()
        );
    }

    #[test]
    fn reflect() {
        assert_eq!(
            Direction(Vec3(1.0, -1.0, 0.0)).normalized().reflect(&Direction(Vec3(0.0, 1.0, 0.0))),
            Direction(Vec3(1.0, 1.0, 0.0)).normalized()
        );

        assert_eq!(
            Direction(Vec3(1.0, 1.0, -1.0)).normalized().reflect(&Direction(Vec3(0.0, 0.0, 1.0))),
            Direction(Vec3(1.0, 1.0, 1.0)).normalized()
        );
    }
}
