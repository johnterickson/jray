use std::ops::{Add, AddAssign, Mul, MulAssign};

#[derive(Copy, Clone, PartialEq)]
pub struct Color(pub f64, pub f64, pub f64);

pub const RED: Color = Color(1.0, 0.0, 0.0);
pub const GREEN: Color = Color(0.0, 1.0, 0.0);
pub const BLUE: Color = Color(0.0, 0.0, 1.0);
pub const WHITE: Color = Color(1.0, 1.0, 1.0);
pub const BLACK: Color = Color(0.0, 0.0, 0.0);

impl Color {
    pub fn to_rgb(&self) -> [u8; 3] {
        [
            (256.0 * self.0).clamp(0.0, 255.0) as u8,
            (256.0 * self.1).clamp(0.0, 255.0) as u8,
            (256.0 * self.2).clamp(0.0, 255.0) as u8,
        ]
    }
}

impl AddAssign<Color> for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}

impl Add<Color> for Color {
    type Output = Self;

    fn add(self, rhs: Color) -> Self::Output {
        let mut result = self;
        result += rhs;
        result
    }
}

impl Mul<Color> for f64 {
    type Output = Color;

    fn mul(self, mut rhs: Color) -> Self::Output {
        rhs *= self;
        rhs
    }
}

impl MulAssign<f64> for Color {
    fn mul_assign(&mut self, rhs: f64) {
        self.0 *= rhs;
        self.1 *= rhs;
        self.2 *= rhs;
    }
}

impl Mul<f64> for Color {
    type Output = Color;

    fn mul(self, rhs: f64) -> Self::Output {
        rhs * self
    }
}

impl Mul<Color> for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color(rhs.0 * self.0, rhs.1 * self.1, rhs.2 * self.2)
    }
}
