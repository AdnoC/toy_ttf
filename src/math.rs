use std::ops::{Add, Mul, Index, IndexMut};
use std::convert::From;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32
}
impl Point {
    pub fn distance_to(&self, other: Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn lerp_to(&self, other: Point, amount: f32) -> Point {
        Point {
            x: self.x + amount * (other.x - self.x),
            y: self.y + amount * (other.y - self.y),
        }
    }
}

impl<T: Into<f32>> From<(T, T)> for Point {
    fn from(p: (T, T)) -> Point {
        Point {
            x: p.0.into(),
            y: p.1.into(),
        }
    }
}

impl Mul<f32> for Point {
    type Output = Point;

    fn mul(self, rhs: f32) -> Point {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
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

pub struct Matrix<T> {
    pub data: Vec<T>,
    pub width: usize,
    pub height: usize,
}

impl<T: Copy + Default> Matrix<T> {
    // Why does `usize` not impl `From<u32`?
    pub fn new(width: u32, height: u32) -> Matrix<T> {
        Self::with_value(Default::default(), width, height)
    }

    pub fn with_value(val: T, width: u32, height: u32) -> Matrix<T> {
        let width = width as usize;
        let height = height as usize;
        let data: Vec<T> = vec![val; width * height];
        Matrix {
            data,
            width,
            height,
        }
    }
}

impl<T> Index<(usize, usize)> for Matrix<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.data[x + y * self.width]
    }
}
impl<T> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut<'a>(&'a mut self, (x, y): (usize, usize)) -> &'a mut Self::Output {
        &mut self.data[x + y * self.width]
    }
}

#[derive(Debug)]
pub struct LineSegment(Point, Point);

impl From<(Point, Point)> for LineSegment {
    fn from((start, end): (Point, Point)) -> Self {
        LineSegment(start, end)
    }
}

impl LineSegment {
    /// Calculate the intersection with the horizontal line `f(x) = y`.
    ///
    /// Returns `None` if no intersection exists.
    ///
    /// If an intersection exists returns the x-coordinate of the intersection
    pub fn horiz_line_intersects(&self, y: f32) -> Option<f32> {
        // Reorder so that the line goes upward
        let (start, end) = if self.0.y < self.1.y {
            (self.0, self.1)
        } else {
            (self.1, self.0)
        };

        // Short circuit on the answer existing
        if y < start.y || y > end.y {
            return None;
        }

        // y = m*(x - x1) + y1
        // (y - y1)/m + x1 = x;

        let slope = (end.y - start.y) / (end.x - start.x);
        let x = (y - start.y) / slope + start.x;

        Some(x)
    }

    /// Returns the winding rule value for this line segment.
    ///
    /// Positive if count should be incremented, negative if it should be
    /// decremented.
    ///
    /// Assuming rays travel along the positive x-axis
    ///
    /// From
    /// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM02/Chap2.html#distinguishing
    ///
    /// Add one to the count each time a glyph contour crosses the ray from right
    /// to left or bottom to top. (Such a crossing is termed an on-transition
    /// because the TrueType scan converter scans from left to right and bottom to top.)
    ///
    /// Subtract one from the count each time a contour of the glyph crosses the
    /// ray from left to right or top to bottom. (Such a crossing is termed an
    /// off-transition.)
    pub fn winding_value(&self) -> i8 {
        if self.0.y == self.1.y {
            // Right -> Left
            if self.0.x > self.1.x {
                1
            // Left -> Right
            } else {
                -1
            }
        // Top -> Bottom
        } else if self.0.y > self.1.y {
            -1
        // Bottom -> Top
        } else {
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_segment_horiz_line_intersect() {
        let ls: LineSegment = ((-1., -1.).into(),(1., 1.).into()).into();

        assert_eq!(ls.horiz_line_intersects(0.), Some(0.));
        assert_eq!(ls.horiz_line_intersects(1.), Some(1.));
        assert_eq!(ls.horiz_line_intersects(-1.), Some(-1.));
    }
}
