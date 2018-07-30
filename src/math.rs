#[derive(Debug, PartialEq, PartialOrd)]
pub struct Point {
    pub x: f32,
    pub y: f32
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Affine {
    // top-left square matrix
    pub square: [[f32; 2]; 2],
    // column to the right of the square
    pub tranlation: [f32; 2],
}
