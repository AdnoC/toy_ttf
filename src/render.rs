use image::GrayImage;
use imageproc::drawing::draw_antialiased_line_segment_mut;
use imageproc::pixelops::interpolate;
use math::Point;
use tables::glyf::{Coordinate, SimpleCoordinates};

pub struct Raster(pub GrayImage);

impl Raster {
    pub fn new(width: u32, height: u32) -> Raster {
        use image::Luma;
        Raster(GrayImage::from_pixel(width, height, Luma { data: [0xFF] }))
    }

    pub fn draw_line(&mut self, start: Point, end: Point) {
        use image::Luma;
        draw_antialiased_line_segment_mut(&mut self.0,
                                      (start.x as i32, start.y as i32),
                                      (end.x as i32, end.y as i32),
                                      Luma { data: [0] },
                                      interpolate
                                      );
    }
    pub fn draw_curve(&mut self, start: Point, off_curve: Point, end: Point) {
        // p(t) = (1-t)^2*p0 + 2*t(1-t)*p1 + t^2*p2

        let dist1 = start.distance_to(off_curve);
        let dist2 = end.distance_to(off_curve);
        let interp_points = dist1 + dist2 + 2.;


        let mut prev = start;
        for i in 1..(interp_points.ceil() as i32 - 1) {
            let t = interp_points.recip() * i as f32;

            let p1 = start.lerp_to(off_curve, t);
            let p2 = off_curve.lerp_to(end, t);
            let p = p1.lerp_to(p2, t);
            // let p1 = start * (1. - t) * (1. - t);
            // let p2 = off_curve * 2. * t * (1. - t);
            // let p3 = end * t * t;
            // let p = p1 + p2 + p3;

            self.draw_line(prev, p);
            prev = p;
        }
        self.draw_line(prev, end);

        // unimplemented!()
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
pub struct DrawCommands<'a> {
    coords: SimpleCoordinates<'a>,
    latest_on_curve: Option<Point>,
    prev_off_curve: Option<Point>,
    // Used to close the shape
    first_coord: Option<Coordinate>,
}

impl<'a> DrawCommands<'a> {
    pub fn from_coordinates(mut coords: SimpleCoordinates<'a>) -> DrawCommands<'a> {

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

impl<'a> Iterator for DrawCommands<'a> {
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
