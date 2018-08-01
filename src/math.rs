use std::ops::Mul;
use std::convert::From;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
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

impl Affine {
    pub fn translation<T: Into<f32>>(tx: T, ty: T) -> Affine {
        Affine {
            square: [[1., 0.], [0., 1.]],
            translation: [tx.into(), ty.into()],
        }
    }
    pub fn scale<T: Into<f32>>(sx: T, sy: T) -> Affine {
        Affine {
            square: [[sx.into(), 0.], [0., sy.into()]],
            translation: [0., 0.],
        }
    }
    pub fn shear<T: Into<f32>>(shx: T, shy: T) -> Affine {
        Affine {
            square: [[1., shx.into()], [shy.into(), 1.]],
            translation: [0., 0.],
        }
    }
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
