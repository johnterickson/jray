use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Vec3(pub f32, pub f32, pub f32);

impl Vec3 {
    pub fn magnitude(&self) -> f32 {
        (self.0 * self.0 + self.1 * self.1 + self.2 * self.2).sqrt()
    }

    pub fn normalize(&mut self) -> () {
        let mag = self.magnitude();
        self.0 /= mag;
        self.1 /= mag;
        self.2 /= mag;
    }

    pub fn normalized(mut self) -> Vec3 {
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

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
        self.1 *= rhs;
        self.2 *= rhs;
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        let mut result = self;
        result *= rhs;
        result
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Point(pub Vec3);

impl Sub for Point {
    type Output = Direction;

    fn sub(self, rhs: Self) -> Self::Output {
        Direction(self.0 - rhs.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Direction(pub Vec3);

impl Direction {
    pub fn dot(&self, rhs: &Self) -> f32 {
        self.0.0 * rhs.0.0 + 
        self.0.1 * rhs.0.1 +
        self.0.2 * rhs.0.2
    }
    
    pub fn cross(&self, rhs: &Self) -> Self {
        Direction(Vec3(
            self.0.1*rhs.0.2 - self.0.2*rhs.0.1,
            self.0.2*rhs.0.0 - self.0.0*rhs.0.2,
            self.0.0*rhs.0.1 - self.0.1*rhs.0.0
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Ray(pub Point, pub Direction);

impl Ray {
    fn from_points(from: Point, to: Point) -> Ray {
        Ray(from, to - from)
    }
}

mod test {
    use super::*;

    #[test]
    fn add(){
        assert_eq!(Vec3(1.0,2.0,3.0) + Vec3(4.0,6.0,9.0), Vec3(5.0,8.0,12.0));
    }

    #[test]
    fn cross(){
        assert_eq!(
            Direction(Vec3(1.0,0.0,0.0)).cross(&Direction(Vec3(0.0,1.0,0.0))),
            Direction(Vec3(0.0,0.0,1.0)));
    }
}