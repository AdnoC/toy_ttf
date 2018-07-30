use std::ops::Mul;
use std::convert::From;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Point {
    pub x: f32,
    pub y: f32
}

impl<T: Into<f32>> From<(T, T)> for Point {
    fn from(p: (T, T)) -> Point {
        Point {
            x: p.0.into(),
            y: p.1.into(),
        }
    }
}

/// Represents the following matrix:
///
/// [square[0][0], square[0][1], translation[0]]
/// [square[1][0], square[1][1], translation[1]]
/// [   0,              0,              1      ]
///
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Affine {
    // top-left square matrix
    pub square: [[f32; 2]; 2],
    // column to the right of the square
    pub translation: [f32; 2],
}

impl<T: Into<Point>> Mul<T> for Affine {
    type Output = Point;
    fn mul(self, rhs: T) -> Self::Output {
        let point = rhs.into();

        Point {
            x: self.square[0][0] * point.x + self.square[0][1] * point.y + self.translation[0],
            y: self.square[1][0] * point.x + self.square[1][1] * point.y + self.translation[1],
        }
    }
}

impl Mul for Affine {
    type Output = Affine;
    fn mul(self, rhs: Self) -> Self::Output {
        Affine {
            square: [
                [
                    self.square[0][0] * rhs.square[0][0] + self.square[0][1] * rhs.square[1][0],
                    self.square[0][0] * rhs.square[0][1] + self.square[0][1] * rhs.square[1][1],
                ],
                [
                    self.square[1][0] * rhs.square[0][0] + self.square[1][1] * rhs.square[1][0],
                    self.square[1][0] * rhs.square[0][1] + self.square[1][1] * rhs.square[1][1],
                ],
            ],
            translation: [
                self.square[0][0] * rhs.translation[0]
                   + self.square[0][1] * rhs.translation[1]
                   + self.translation[0],
                self.square[1][0] * rhs.translation[0]
                   + self.square[1][1] * rhs.translation[1]
                   + self.translation[1]
            ]
        }
    }
}
