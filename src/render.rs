use itertools::Itertools;
use image::{ImageBuffer, Luma, GrayImage, DynamicImage, RgbImage};
use imageproc::drawing::draw_antialiased_line_segment_mut; // TODO: Pick ONE draw_line func
use imageproc::drawing::draw_line_segment_mut;
use math::{LineSegment, Matrix, Point};
use tables::glyf::{Coordinate, SimpleCoordinates};

type GrayDirectedImage = ImageBuffer<Luma<i16>, Vec<i16>>;

/// Contains things for composing bitmaps of individual characters onto a
/// string of text.
pub mod compositor {
    use image::GrayImage;
    use tables::hhea::HHEA;
    /// Abstraction over a string of text to render, who's characters have been
    /// turned into bitmaps.
    pub struct RenderedText {
        img: GrayImage,
        /// Distance from baseline to highest grid coordinate to place an outline point.
        ascent: i16,
        /// Distance from baseline to lowest grid coordinate to place an outline point.
        descent: i16,
        /// Distance that must be placed between two lines of text.
        line_gap: i16,
        /// y-coordinate of the current baseline.
        current_baseline_y: u32,
        /// What direction are we writing the text?
        text_direction: TextDirection,
    }

    impl RenderedText {
        pub fn new_horizontal<'a>(horiz_header: &HHEA<'a>, text_direction: TextDirection) -> RenderedText {
            RenderedText {
                img: GrayImage::new(0, 0),
                ascent: horiz_header.ascent,
                descent: horiz_header.descent,
                line_gap: horiz_header.line_gap,
                current_baseline_y: 0,
                text_direction,
            }

        }
        pub fn new_left_to_right<'a>(horiz_header: &HHEA<'a>) -> RenderedText {
            Self::new_horizontal(horiz_header, TextDirection::Right)
        }
        pub fn add_glyph(&mut self) {
            unimplemented!()
        }
        pub fn newline(&mut self) {
            unimplemented!()
        }
    }

    pub enum TextDirection {
        Left,
        Right,
        Up,
        Down,
    }

    /// Abstraction over a single rendered glyph. Includes the information needed
    /// to place it with the other rendered text.
    ///
    /// # Implicit Metrics
    ///
    /// * `width = img.width()`
    ///
    /// * `height = img.height()`
    ///
    /// * `right_bearing = placement_metrics.Advance::Width - width`
    ///
    /// * `bottom_bearing = placement_metrics.Advance::Height - height`
    pub struct RenderedGlyph {
        /// The bitmap of the glyph.
        ///
        /// Shouldn't have extra padding or stuff.
        img: GrayImage,

        placement_metrics: GlyphPlacementMetrics,
    }

    pub struct GlyphPlacementMetrics {
        /// How much `img` is shifted away from (0, 0)
        ///
        /// (Since glyphs are meant to "rest" on the baseline)
        ///
        /// Is this needed?
        pub shift: [f32; 2],
        /// Horizontal distance from origin to `img` (left edge of bbox)
        pub left_bearing: i16,
        /// Vertical distance from origin to `img` (top edge of bbox)
        pub top_bearing: i16,
        /// After drawing a glyph you move the "pen" this amount right
        pub horiz_advance: Option<u16>,
        /// After drawing a glyph you move the "pen" this amount down
        pub vert_advance: Option<u16>,
    }
}

pub trait Raster {
    fn new(width: u32, height: u32) -> Self; // Just for convenience of not needing another impl block
    fn add_line(&mut self, start: Point, end: Point);
    fn into_dynamic(self) -> DynamicImage;
}

pub struct FillInRaster {
    windings: Matrix<isize>,
    lines: Vec<LineSegment>,
}

impl Raster for FillInRaster {
    fn new(width: u32, height: u32) -> Self {
        FillInRaster {
            windings: Matrix::new(width, height),
            lines: Vec::new(),
        }
    }

    fn add_line(&mut self, start: Point, end: Point) {
        // Horizontal lines screw up intersection finding code
        if start.y != end.y {
            self.lines.push((start, end).into());
        }
    }

    fn into_dynamic(mut self) -> DynamicImage {
        use std::u8;
        // TODO: Handle dropouts

        // Just asking to be parallellized
        for (y, row) in self.windings.data.chunks_mut(self.windings.width).enumerate() {
            let y = y as f32 + 0.5; // Scanline is at center of a pixel
            for line in self.lines.iter() {
                if let Some(x) = line.horiz_line_intersects(y) {
                    let wind_val = line.winding_value() as isize;
                    let x = x.round() as usize;
                    for winding in &mut row[x..] {
                        *winding += wind_val;
                    }
                }
            }
        }

        let img_data = self.windings.data.into_iter()
            .map(|wind_val| wind_val.abs().min(1))
            .map(|pix_on| pix_on as u8 * u8::MAX)
            .collect();

        let img = GrayImage::from_vec(self.windings.width as u32,
                                      self.windings.height as u32,
                                      img_data)
            .expect("Couldn't re-create GrayImage");
        DynamicImage::ImageLuma8(img)
    }
}


pub struct OutlineRaster(pub GrayImage);
impl Raster for OutlineRaster {
    fn new(width: u32, height: u32) -> Self {
        OutlineRaster(
            GrayImage::from_pixel(width, height, Luma { data: [0] }),
        )
    }
    fn add_line(&mut self, start: Point, end: Point) {
        use std::u8;
        draw_line_segment_mut(&mut self.0,
                              (start.x as f32, start.y as f32),
                              (end.x as f32, end.y as f32),
                              Luma { data: [u8::MAX] });
    }
    fn into_dynamic(self) -> DynamicImage {
        DynamicImage::ImageLuma8(self.0)
    }
}

pub struct ColorDirectedRaster(pub GrayDirectedImage);
impl ColorDirectedRaster {
    /// Returns a pixel value that follows the winding rule.
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
    fn directioned_value(start: Point, end: Point) -> i16 {
        use std::u8;
        if start.y == end.y {
            // Right -> Left
            if start.x > end.x {
                u8::MAX as i16
            // Left -> Right
            } else {
                -(u8::MAX as i16)
            }
        // Top -> Bottom
        } else if start.y > end.y {
            -(u8::MAX as i16)
        // Bottom -> Top
        } else {
            u8::MAX as i16
        }
    }
}
impl Raster for ColorDirectedRaster {
    fn new(width: u32, height: u32) -> Self {
        ColorDirectedRaster(
            GrayDirectedImage::from_pixel(width, height, Luma { data: [0] }),
        )
    }
    fn add_line(&mut self, start: Point, end: Point) {
        fn interpolate_directed(a: Luma<i16>, b: Luma<i16>, weight: f32) -> Luma<i16> {
            let a = a.data[0] as f32;
            let b = b.data[0] as f32;

            let weighted_a = a * weight;
            let weighted_b = b * (1. - weight);

            let abs_result = weighted_a.abs() + weighted_b.abs();

            let highest_mag_val = if weighted_a.abs() > weighted_b.abs() {
                weighted_a
            } else {
                weighted_b
            };

            let result = if highest_mag_val > 0. {
                abs_result
            } else {
                -abs_result
            };

            Luma { data: [result as i16] }
        }
        // self.draw_point(start, 5);
        // self.draw_point(end, 5);
        // let pix_val = Self::directioned_value(start, end);
        let pix_val = Self::directioned_value(start, end);
        draw_line_segment_mut(&mut self.0,
                              (start.x as f32, start.y as f32),
                              (end.x as f32, end.y as f32),
                              Luma { data: [pix_val] });
        // draw_antialiased_line_segment_mut(&mut self.0,
        //                                   (start.x as i32, start.y as i32),
        //                                   (end.x as i32, end.y as i32),
        //                                   Luma { data: [pix_val] },
        //                                   interpolate_directed);
    }
    fn into_dynamic(self) -> DynamicImage {
        use std::u8;

        let width = self.0.width();
        let height = self.0.height();
        let data: Vec<u8> = self.0.into_vec()
            .chunks(width as usize)
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .cloned()
                    .map(|pix| {
                        if pix > 0 {
                            &[u8::MAX, 0, 0]
                        } else if pix < 0 {
                            &[0, u8::MAX, 0]
                        } else { &[0, 0, 0] }
                    })
                    .flatten()
                    .cloned()
            })
            .flatten()
            .collect();
        let rgb = RgbImage::from_vec(width, height, data)
            .expect("Couldn't re-create RgbImage");
        DynamicImage::ImageRgb8(rgb)
    }
}

pub struct BadFilledRaster(pub GrayDirectedImage);
impl BadFilledRaster {
    /// Returns a pixel value that follows the winding rule.
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
    fn directioned_value(start: Point, end: Point) -> i16 {
        use std::u8;
        if start.y == end.y {
            // Right -> Left
            if start.x > end.x {
                u8::MAX as i16
            // Left -> Right
            } else {
                -(u8::MAX as i16)
            }
        // Top -> Bottom
        } else if start.y > end.y {
            -(u8::MAX as i16)
        // Bottom -> Top
        } else {
            u8::MAX as i16
        }
    }
}

impl Raster for BadFilledRaster {
    fn new(width: u32, height: u32) -> Self {
        BadFilledRaster(
            GrayDirectedImage::from_pixel(width, height, Luma { data: [0] }),
        )
    }
    fn add_line(&mut self, start: Point, end: Point) {
        fn interpolate_directed(a: Luma<i16>, b: Luma<i16>, weight: f32) -> Luma<i16> {
            let a = a.data[0] as f32;
            let b = b.data[0] as f32;

            let weighted_a = a * weight;
            let weighted_b = b * (1. - weight);

            let abs_result = weighted_a.abs() + weighted_b.abs();

            let highest_mag_val = if weighted_a.abs() > weighted_b.abs() {
                weighted_a
            } else {
                weighted_b
            };

            let result = if highest_mag_val > 0. {
                abs_result
            } else {
                -abs_result
            };

            Luma { data: [result as i16] }
        }
        // self.draw_point(start, 5);
        // self.draw_point(end, 5);
        // let pix_val = Self::directioned_value(start, end);
        let pix_val = Self::directioned_value(start, end);
        draw_line_segment_mut(&mut self.0,
                              (start.x as f32, start.y as f32),
                              (end.x as f32, end.y as f32),
                              Luma { data: [pix_val] });
        // draw_antialiased_line_segment_mut(&mut self.0,
        //                                   (start.x as i32, start.y as i32),
        //                                   (end.x as i32, end.y as i32),
        //                                   Luma { data: [pix_val] },
        //                                   interpolate_directed);
    }
    fn into_dynamic(self) -> DynamicImage {
        use std::u8;

        let width = self.0.width();
        let height = self.0.height();
        let data: Vec<u8> = self.0.into_vec()
            .chunks(width as usize)
            .into_iter()
            .map(|row| {
                let mut count = 0;
                let apply_winding_rule = move |pix: i16| {
                    if pix > 0 {
                        count += 1;
                    } else if pix < 0 {
                        count -= 1;
                    }
                    if count != 0 {
                        u8::MAX
                    } else {
                        0
                    }
                };
                row.into_iter().cloned().map(apply_winding_rule)
            })
            .flatten()
            .collect();
        let img = GrayImage::from_vec(width, height, data)
            .expect("Couldn't re-create GrayImage");
        DynamicImage::ImageLuma8(img)
    }
}
// impl GenRaster {
//     pub fn put_pixel(&mut self, x: u32, y: u32, brightness: i16) {
//         assert!(x < self.0.width());
//         assert!(y < self.0.height());
//         self.0.put_pixel(x, y, Luma { data: [brightness] });
//     }
//
//     pub fn draw_point(&mut self, p: Point, size: i32) {
//         for dx in (-size)..size {
//             for dy in (-size)..size {
//                 let x = p.x as i32 + dx;
//                 let y = p.y as i32 + dy;
//                 if x < self.0.width() as i32 && x > 0
//                     && y < self.0.height() as i32 && y > 0 {
//                     let x = x as u32;
//                     let y = y as u32;
//                     self.put_pixel(x, y, 0);
//                 }
//             }
//         }
//
//     }
//
//
//
//     pub fn draw_curve(&mut self, start: Point, off_curve: Point, end: Point) {
//         for (a, b) in CurveLines::new(start, off_curve, end) {
//             self.draw_line(a, b);
//         }
//     }
// }

pub struct CurveLines {
    start: Point,
    off_curve: Point,
    end: Point,
    prev: Point,
    num_points: u32,
    i: u32
}
impl CurveLines {
    pub fn new(start: Point, off_curve: Point, end: Point) -> CurveLines {
        let dist1 = start.distance_to(off_curve);
        let dist2 = end.distance_to(off_curve);
        let num_points = dist1 + dist2 + 2.;
        let num_points = num_points.floor() as u32;

        CurveLines {
            start,
            off_curve,
            end,
            prev: start,
            num_points,
            i: 0,
        }
    }
}

impl Iterator for CurveLines {
    type Item = (Point, Point);
    fn next(&mut self) -> Option<Self::Item> {
        // p(t) = (1-t)^2*p0 + 2*t(1-t)*p1 + t^2*p2
        if self.i >= self.num_points {
            return None;
        }
        self.i += 1;

        if self.i == self.num_points {
            return Some((self.prev, self.end));
        }


        let t = (self.num_points as f32).recip() * self.i as f32;

        let p1 = self.start.lerp_to(self.off_curve, t);
        let p2 = self.off_curve.lerp_to(self.end, t);
        let p = p1.lerp_to(p2, t);
        // let p1 = self.start * (1. - t) * (1. - t);
        // let p2 = self.off_curve * 2. * t * (1. - t);
        // let p3 = self.end * t * t;
        // let p = p1 + p2 + p3;

        let segment = (self.prev, p);
        self.prev = p;
        Some(segment)
    }
}

fn coord_to_point(coord: Coordinate) -> Point {
    Point {
        x: coord.x as f32,
        y: coord.y as f32
    }
}

// Iterator from Coordinate(s) -> DrawCommand
// End points of contours can be derived from points
// on_curve + on_curve = line
// on_curve + off_curve + _ = curve
// off_curve + off_curve = 2 curves with implied on_curve between the two
pub struct DrawCommands<I: Iterator<Item = Coordinate>> {
    coords: I,
    latest_on_curve: Option<Point>,
    prev_off_curve: Option<Point>,
    // Used to close the shape
    first_coord: Option<Coordinate>,
}

impl<I: Iterator<Item = Coordinate>> DrawCommands<I> {
    pub fn from_coordinates(mut coords: I) -> DrawCommands<I> {
        let first_coord = coords.next();
        if let Some(first_coord) = &first_coord {
            assert!(first_coord.on_curve);
        }
        DrawCommands {
            coords,
            first_coord,
            latest_on_curve: first_coord.map(coord_to_point),
            prev_off_curve: None,
        }
    }
}

impl<I: Iterator<Item=Coordinate>> Iterator for DrawCommands<I> {
    type Item = DrawCommand;

    fn next(&mut self) -> Option<Self::Item> {
        let next_coord = match self.coords.next() {
            Some(coord) => coord,
            None => match self.first_coord.take() {
                // To close the shape
                Some(coord) => coord,
                // Stopping condition
                None => return None,
            },
        };
        let next_point = coord_to_point(next_coord);

        let latest_on_curve = match self.latest_on_curve {
            Some(latest_on_curve) => latest_on_curve,
            None => panic!("Should always have a previous on-curve point"),
        };

        if next_coord.on_curve {
            self.latest_on_curve = Some(next_point);
        }

        let command = match self.prev_off_curve.take() {
            Some(prev_off_curve) => {
                if next_coord.on_curve {
                    DrawCommand::Curve(latest_on_curve, prev_off_curve, next_point)
                } else {
                    self.prev_off_curve = Some(next_point);
                    let interp_point = prev_off_curve.lerp_to(next_point, 0.5);
                    self.latest_on_curve = Some(interp_point);
                    DrawCommand::Curve(latest_on_curve, prev_off_curve, interp_point)
                }
            },
            None => {
                if next_coord.on_curve {
                    DrawCommand::Line(latest_on_curve, next_point)
                } else {
                    self.prev_off_curve = Some(next_point);

                    return self.next();
                }
            }
        };
        Some(command)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DrawCommand {
    Line(Point, Point),
    Curve(Point, Point, Point),
}

pub struct FlattenedDrawCommands<I: Iterator<Item=Coordinate>> {
    inner: DrawCommands<I>,
    current_curve: Option<CurveLines>,
}
impl<I: Iterator<Item = Coordinate>> FlattenedDrawCommands<I> {
    pub fn from_coordinates(coords: I) -> FlattenedDrawCommands<I> {
        FlattenedDrawCommands {
            inner: DrawCommands::from_coordinates(coords),
            current_curve: None,
        }
    }
}
impl<I: Iterator<Item=Coordinate>> Iterator for FlattenedDrawCommands<I> {
    type Item = (Point, Point);
    fn next(&mut self) -> Option<Self::Item> {
        let curve_line = self.current_curve.as_mut().and_then(|inner| inner.next());
        if curve_line.is_some() {
            return curve_line;
        } else {
            self.current_curve = None;
        }
        let dc = self.inner.next()?;

        match dc {
            DrawCommand::Line(start, end) => Some((start, end)),
            DrawCommand::Curve(start, off_curve, end) => {
                let mut curve = CurveLines::new(start, off_curve, end);
                let segment = curve.next();
                self.current_curve = Some(curve);
                segment
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use font::*;
    use test_utils::font_buf;

    #[test]
    fn draw_commands() {
        use tables::glyf::Description;

        let expecteds = &[
            DrawCommand::Line(Point { x: 1012.0, y: 1442.0 }, Point { x: 1012.0, y: 1237.0 }),
            DrawCommand::Curve(Point { x: 1012.0, y: 1237.0 }, Point { x: 735.0, y: 1356.0 }, Point { x: 827.5, y: 1326.0 }),
            DrawCommand::Curve(Point { x: 827.5, y: 1326.0 }, Point { x: 735.0, y: 1356.0 }, Point { x: 641.0, y: 1356.0 }),
            DrawCommand::Curve(Point { x: 641.0, y: 1356.0 }, Point { x: 332.0, y: 1223.0 }, Point { x: 415.0, y: 1289.5 }),
            DrawCommand::Curve(Point { x: 415.0, y: 1289.5 }, Point { x: 332.0, y: 1223.0 }, Point { x: 332.0, y: 1110.0 }),
            DrawCommand::Curve(Point { x: 332.0, y: 1110.0 }, Point { x: 441.0, y: 907.0 }, Point { x: 386.5, y: 959.0 }),
            DrawCommand::Curve(Point { x: 386.5, y: 959.0 }, Point { x: 441.0, y: 907.0 }, Point { x: 590.0, y: 872.0 }),
            DrawCommand::Line(Point { x: 590.0, y: 872.0 }, Point { x: 696.0, y: 848.0 }),
            DrawCommand::Curve(Point { x: 696.0, y: 848.0 }, Point { x: 1098.0, y: 589.0 }, Point { x: 1002.0, y: 694.0 }),
            DrawCommand::Curve(Point { x: 1002.0, y: 694.0 }, Point { x: 1098.0, y: 589.0 }, Point { x: 1098.0, y: 408.0 }),
            DrawCommand::Curve(Point { x: 1098.0, y: 408.0 }, Point { x: 834.0, y: -29.0 }, Point { x: 966.0, y: 83.0 }),
            DrawCommand::Curve(Point { x: 966.0, y: 83.0 }, Point { x: 834.0, y: -29.0 }, Point { x: 582.0, y: -29.0 }),
            DrawCommand::Curve(Point { x: 582.0, y: -29.0 }, Point { x: 265.0, y: 16.0 }, Point { x: 371.0, y: -6.5 }),
            DrawCommand::Curve(Point { x: 371.0, y: -6.5 }, Point { x: 265.0, y: 16.0 }, Point { x: 158.0, y: 61.0 }),
            DrawCommand::Line(Point { x: 158.0, y: 61.0 }, Point { x: 158.0, y: 276.0 }),
            DrawCommand::Curve(Point { x: 158.0, y: 276.0 }, Point { x: 478.0, y: 135.0 }, Point { x: 375.5, y: 169.0 }),
            DrawCommand::Curve(Point { x: 375.5, y: 169.0 }, Point { x: 478.0, y: 135.0 }, Point { x: 582.0, y: 135.0 }),
            DrawCommand::Curve(Point { x: 582.0, y: 135.0 }, Point { x: 905.0, y: 272.0 }, Point { x: 820.0, y: 203.5 }),
            DrawCommand::Curve(Point { x: 820.0, y: 203.5 }, Point { x: 905.0, y: 272.0 }, Point { x: 905.0, y: 395.0 }),
            DrawCommand::Curve(Point { x: 905.0, y: 395.0 }, Point { x: 788.0, y: 625.0 }, Point { x: 846.5, y: 566.0 }),
            DrawCommand::Curve(Point { x: 846.5, y: 566.0 }, Point { x: 788.0, y: 625.0 }, Point { x: 643.0, y: 657.0 }),
            DrawCommand::Line(Point { x: 643.0, y: 657.0 }, Point { x: 535.0, y: 682.0 }),
            DrawCommand::Curve(Point { x: 535.0, y: 682.0 }, Point { x: 139.0, y: 919.0 }, Point { x: 233.0, y: 824.0 }),
            DrawCommand::Curve(Point { x: 233.0, y: 824.0 }, Point { x: 139.0, y: 919.0 }, Point { x: 139.0, y: 1079.0 }),
            DrawCommand::Curve(Point { x: 139.0, y: 1079.0 }, Point { x: 408.0, y: 1520.0 }, Point { x: 273.5, y: 1399.5 }),
            DrawCommand::Curve(Point { x: 273.5, y: 1399.5 }, Point { x: 408.0, y: 1520.0 }, Point { x: 631.0, y: 1520.0 }),
            DrawCommand::Curve(Point { x: 631.0, y: 1520.0 }, Point { x: 907.0, y: 1481.0 }, Point { x: 812.0, y: 1500.5 }),
            DrawCommand::Curve(Point { x: 812.0, y: 1500.5 }, Point { x: 907.0, y: 1481.0 }, Point { x: 1012.0, y: 1442.0 }),
        ];

        let font_buf = font_buf();
        let font = Font::from_buffer(&font_buf).unwrap();
        let glyph = font.get_glyph('S').unwrap(); // Codepoint is 188
        let dcs = match glyph.desc {
            Description::Simple(glyph) => DrawCommands::from_coordinates(glyph.coordinates()),
            _ => panic!("Should be simple"),
        };

        for (actual, &expected) in dcs.zip(expecteds.into_iter()) {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn curve_iterator() {
        let expecteds = &[
            (Point { x: 0.0, y: 0.0 }, Point { x: 0.0625, y: 0.5 }),
            (Point { x: 0.0625, y: 0.5 }, Point { x: 0.25, y: 1.0 }),
            (Point { x: 0.25, y: 1.0 }, Point { x: 0.5625, y: 1.5 }),
            (Point { x: 0.5625, y: 1.5 }, Point { x: 1.0, y: 2.0 }),
            (Point { x: 1.0, y: 2.0 }, Point { x: 1.5625, y: 2.5 }),
            (Point { x: 1.5625, y: 2.5 }, Point { x: 2.25, y: 3.0 }),
            (Point { x: 2.25, y: 3.0 }, Point { x: 3.0625, y: 3.5 }),
            (Point { x: 3.0625, y: 3.5 }, Point { x: 4.0, y: 4.0 }),
        ];
        let p1 = (0., 0.).into();
        let p2 = (4., 4.).into();
        let off = (0., 2.).into();

        let curve = CurveLines::new(p1, off, p2);
        for (actual, &expected) in curve.zip(expecteds) {
            assert_eq!(actual, expected);
        }
    }
}
