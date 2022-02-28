use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Vec3 {
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
}

impl Vec3 {
    pub fn magnitude(&self) -> f32 {
        (self.X * self.X + self.Y * self.Y + self.Z * self.Z).sqrt()
    }

    pub fn normalize(&mut self) -> () {
        let mag = self.magnitude();
        self.X /= mag;
        self.Y /= mag;
        self.Z /= mag;
    }

    pub fn normalized(mut self) -> Vec3 {
        self.normalize();
        self
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.X += rhs.X;
        self.Y += rhs.Y;
        self.Z += rhs.Z;
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
        self.X -= rhs.X;
        self.Y -= rhs.Y;
        self.Z -= rhs.Z;
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