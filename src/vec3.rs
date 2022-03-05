use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Vec3(pub f64, pub f64, pub f64);

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
        Vec3(
            self * rhs.0,
            self * rhs.1,
            self * rhs.2,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Point(pub Vec3);

impl Point {
    pub const fn origin() -> Point {
        Point(Vec3(0.0,0.0,0.0))
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

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Direction(pub Vec3);

impl Direction {
    pub const fn none() -> Self {
        Direction(Vec3(0.0,0.0,0.0))
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
        let r = self.dot(normal) * 2.0;
        Direction(self.0 - (normal.0 * r))
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
        Direction(self.0 + rhs.0)
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
        Direction(rhs.0 * self)
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
    fn reflect() {
        assert_eq!(
            Direction(Vec3(1.0, 1.0, 0.0)).reflect(&Direction(Vec3(0.0, 1.0, 0.0))),
            Direction(Vec3(1.0, -1.0, 0.0))
        );
    }
}
