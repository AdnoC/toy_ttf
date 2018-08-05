use itertools::Itertools;
use image::{ImageBuffer, Luma, GrayImage};
use imageproc::drawing::draw_antialiased_line_segment_mut; // TODO: Pick ONE draw_line func
use imageproc::drawing::draw_line_segment_mut;
use math::Point;
use tables::glyf::{Coordinate, SimpleCoordinates};

type GrayDirectedImage = ImageBuffer<Luma<i16>, Vec<i16>>;

pub struct Raster(pub GrayDirectedImage, u64);

impl Raster {
    pub fn new(width: u32, height: u32) -> Raster {
        Raster(
            GrayDirectedImage::from_pixel(width, height, Luma { data: [0] }),
            0
        )
    }

    pub fn into_gray_image(self) -> GrayImage  {
        use std::u8;
        use std::slice::Iter as SliceIter;
        use std::iter::{Map, Cloned};

        #[allow(dead_code)]
        fn fill_in<'a>(row: SliceIter<'a, i16>) -> Map<
            Cloned<SliceIter<'a, i16>>,
            impl FnMut(i16) -> u8> {
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
        }
        #[allow(dead_code)]
        fn just_outline<'a>(row: SliceIter<'a, i16>) -> Map<
            Cloned<SliceIter<'a, i16>>,
            impl FnMut(i16) -> u8> {
            row.into_iter()
                .cloned()
                // .map(|pix: i16| pix.abs().min(u8::MAX as i16) as u8)
                .map(|pix: i16| {
                    let pix = if pix > 0 {
                        u8::MAX as i16
                    } else if pix < 0 {
                        u8::MAX as i16 / 2
                    } else { 0 };
                    pix as u8
// pix.abs().min(u8::MAX as i16) as u8
                })
        }

        // let rgb_outline = |row| {
        //     row.into_iter()
        //         .cloned()
        //         .map(|pix| {
        //             if pix > 0 {
        //                 &[u8::MAX, 0, 0]
        //             } else if pix < 0 {
        //                 &[0, u8::MAX, 0]
        //             } else { &[0, 0, 0] }
        //         })
        //         .flatten()
        //         .cloned()
        // }

        let width = self.0.width();
        let height = self.0.height();
        let data: Vec<u8> = self.0.into_vec()
            .chunks(width as usize)
            .into_iter()
            .map(|row| fill_in(row.into_iter()))
            .flatten()
            .collect();
        GrayImage::from_vec(width, height, data).expect("Couldn't re-create GrayImage")
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, brightness: i16) {
        assert!(x < self.0.width());
        assert!(y < self.0.height());
        self.0.put_pixel(x, y, Luma { data: [brightness] });
    }

    pub fn draw_point(&mut self, p: Point, size: i32) {
        for dx in (-size)..size {
            for dy in (-size)..size {
                let x = p.x as i32 + dx;
                let y = p.y as i32 + dy;
                if x < self.0.width() as i32 && x > 0
                    && y < self.0.height() as i32 && y > 0 {
                    let x = x as u32;
                    let y = y as u32;
                    self.put_pixel(x, y, 0);
                }
            }
        }

    }

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

    pub fn draw_line(&mut self, start: Point, end: Point) {
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
    pub fn draw_curve(&mut self, start: Point, off_curve: Point, end: Point) {
        for (a, b) in CurveLines::new(start, off_curve, end) {
            self.draw_line(a, b);
        }
    }
}

pub struct CurveLines {
    start: Point,
    off_curve: Point,
    end: Point,
    prev: Point,
    num_points: f32,
    i: f32
}
impl CurveLines {
    fn new(start: Point, off_curve: Point, end: Point) -> CurveLines {
        let dist1 = start.distance_to(off_curve);
        let dist2 = end.distance_to(off_curve);
        let num_points = dist1 + dist2 + 2.;

        CurveLines {
            start,
            off_curve,
            end,
            prev: start,
            num_points,
            i: 0.,
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
        self.i += 1.;

        if self.i == self.num_points {
            return Some((self.prev, self.end));
        }


        let t = self.num_points.recip() * self.i;

        let p1 = self.start.lerp_to(self.off_curve, t);
        let p2 = self.off_curve.lerp_to(self.end, t);
        let p = p1.lerp_to(p2, t);
        // let p1 = start * (1. - t) * (1. - t);
        // let p2 = off_curve * 2. * t * (1. - t);
        // let p3 = end * t * t;
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
}
