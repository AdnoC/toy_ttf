use parse::{Parse, BufView, DynArr};
use tables::{PrimaryTable, TableTag};

// Total # of glyphs is `num_glyphs` in MaxP table
// Loca table provides index of glyph by glyph_id

#[derive(Debug, Parse)]
pub struct Glyf<'a>(BufView<'a, u8>);

impl<'a> PrimaryTable for Glyf<'a> {
    fn tag() -> TableTag {
        TableTag::GlyphOutline
    }
}

impl<'a> Glyf<'a> {
    pub fn at_offset(&self, offset: usize) -> Option<Glyph<'a>> {
        use std::marker::PhantomData;

        if offset > (self.0).0.len() {
            return None;
        }

        let start = &(self.0).0[offset..];
        let (contents, header) = Header::parse(start);
        assert!(header.x_min < header.x_max);
        assert!(header.y_min < header.y_max);
        let desc = if header.number_of_contours > 0 {
            let end_pts_size = u16::approx_file_size() * header.number_of_contours as usize;

            let (end_points_of_contours_buf, buf) = contents.split_at(end_pts_size);
            let end_points_of_contours = DynArr(end_points_of_contours_buf, PhantomData);

            let (buf, instruction_length) = u16::parse(buf);

            let (instructions_buf, buf) = buf.split_at(instruction_length as usize);
            let instructions = DynArr(instructions_buf, PhantomData);

            let coords = BufView(buf, PhantomData);

            Description::Simple(SimpleGlyph {
                end_points_of_contours,
                instruction_length,
                instructions,
                coords,
            })
        } else {
            unimplemented!()
            // Description::Composite // TODO
        };

        Some(Glyph {
            header, desc,
        })
    }
}

#[derive(Debug, Parse)]
pub struct Header {
    number_of_contours: i16,
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
}

pub struct Glyph<'a> {
    pub header: Header,
    pub desc: Description<'a>,
}
impl<'a> Glyph<'a> {
    // TODO: Name for compoind glyph
    pub fn coordinates(&self) -> SimpleCoordinates<'a> {
        use std::marker::PhantomData;

        let simp = match self.desc {
            Description::Simple(ref simp) => simp,
            _ => unimplemented!()
        };

        let flags = simp.coords.cast::<SimpleFlags>();
        let mut idx = 0;
        let mut xs_len = 0;
        let mut ys_len = 0;
        // Last point index in each countour is the highest,
        // last countour has the highest end point index. TODO: Verify
        let last_point_index_offset = simp.end_points_of_contours.len() - 1;
        // Plus one since the index is zero-based
        let mut points_left = 1 + simp.end_points_of_contours.at(last_point_index_offset) as usize;
        while points_left > 0 {
            let flag = flags.at(idx);
            assert!(!flag.intersects(SimpleFlags::RESERVED));

            let repeat_count = if flag.contains(SimpleFlags::REPEAT_FLAG) {
                idx += 1;
                let count_buf = flags.split_at(idx).1;
                // Plus 1 since the repeat flag wouldn't be used for just 1
                count_buf.cast::<u8>().at(0) + 1
            } else { 1 } as usize;

            let x_size = if flag.contains(SimpleFlags::X_SHORT_VEC) {
                u8::approx_file_size()
            } else if flag.contains(SimpleFlags::X_IS_SAME) {
                0
            } else {
                i16::approx_file_size()
            };
            let y_size = if flag.contains(SimpleFlags::Y_SHORT_VEC) {
                u8::approx_file_size()
            } else if flag.contains(SimpleFlags::Y_IS_SAME) {
                0
            } else {
                i16::approx_file_size()
            };

            xs_len += repeat_count * x_size;
            ys_len += repeat_count * y_size;

            points_left -= repeat_count;
            idx += 1;
        }
        let (flags_buf, rest) = flags.split_at(idx);
        let flags = DynArr(flags_buf.0, PhantomData);
        let (delta_xs, rest) = rest.0.split_at(xs_len);
        let delta_ys = &rest[..ys_len];

        SimpleCoordinates {
            flags,
            delta_xs,
            delta_ys,
            repeat_count: 0,
            // First point is relative to (0,0)
            x: 0,
            y: 0,
        }
    }
}

pub enum Description<'a> {
    Simple(SimpleGlyph<'a>),
    Composite, // TODO
}

pub struct SimpleGlyph<'a> {
    end_points_of_contours: DynArr<'a, u16>,
    instruction_length: u16,
    instructions: DynArr<'a, u8>,
    coords: BufView<'a, u8>,
    // These are derived from coords
    // flags: BufView<'a, u8>,
    // x_coords: BufView<'a, u8>,
    // y_coords: BufView<'a, u8>,
}

bitflags! {
    #[derive(Parse)]
    struct SimpleFlags: u8 {
        const ON_CURVE_POINT                               = 0b00000001; // 0x1
        const X_SHORT_VEC                                  = 0b00000010; // 0x2
        const Y_SHORT_VEC                                  = 0b00000100; // 0x4
        const REPEAT_FLAG                                  = 0b00001000; // 0x8
        const X_IS_SAME                                    = 0b00010000; // 0x10
        const POSITIVE_X_SHORT_VECTOR                      = 0b00010000; // 0x10
        const Y_IS_SAME                                    = 0b00100000; // 0x20
        const POSITIVE_Y_SHORT_VECTOR                      = 0b00100000; // 0x20
        const RESERVED                                     = 0b11000000;
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct SimpleCoordinate {
    pub on_curve: bool,
    pub x: i16,
    pub y: i16,
}
pub struct SimpleCoordinates<'a> {
    flags: DynArr<'a, SimpleFlags>,
    delta_xs: &'a [u8],
    delta_ys: &'a [u8],
    repeat_count: u8,
    // coordinate values are relative to the previous point
    x: i16,
    y: i16,
}
impl<'a> Iterator for SimpleCoordinates<'a> {
    type Item = SimpleCoordinate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.flags.len() == 0 {
            return None;
        }

        let flag = self.flags.at(0);

        if self.repeat_count == 0 && flag.contains(SimpleFlags::REPEAT_FLAG) {
            let count_buf = self.flags.split_at(1).1;
            self.repeat_count = u8::parse(count_buf.0).1 + 1;
        }

        let on_curve = flag.contains(SimpleFlags::ON_CURVE_POINT);

        let dx = if flag.contains(SimpleFlags::X_SHORT_VEC) {
            let (rest, dx) = u8::parse(self.delta_xs);
            let dx = dx as i16;
            self.delta_xs = rest;

            if flag.contains(SimpleFlags::POSITIVE_X_SHORT_VECTOR) {
                dx
            } else {
                -dx
            }
        } else if !flag.contains(SimpleFlags::X_IS_SAME) {
            let (rest, dx) = i16::parse(self.delta_xs);
            self.delta_xs = rest;
            dx
        } else { 0 };

        let dy = if flag.contains(SimpleFlags::Y_SHORT_VEC) {
            let (rest, dy) = u8::parse(self.delta_ys);
            let dy = dy as i16;
            self.delta_ys = rest;

            if flag.contains(SimpleFlags::POSITIVE_Y_SHORT_VECTOR) {
                dy
            } else {
                -dy
            }
        } else if !flag.contains(SimpleFlags::Y_IS_SAME) {
            let (rest, dy) = i16::parse(self.delta_ys);
            self.delta_ys = rest;
            dy
        } else { 0 };

        self.x += dx;
        self.y += dy;

        if self.repeat_count > 0 {
            self.repeat_count -= 1;
            // Get to the count so that we then get to the next one
            if self.repeat_count == 0 {
                self.flags = self.flags.split_at(1).1;
            }
        }

        if self.repeat_count == 0 {
            self.flags = self.flags.split_at(1).1;
        }

        Some(SimpleCoordinate {
            on_curve,
            x: self.x,
            y: self.y,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use font::*;
    use test_utils::font_buf;

    #[test]
    fn simple_coordinates() {
        use tables::cmap::CMap;
        use tables::loca::Loca;
        use tables::glyf::Glyf;

        let expecteds = &[
            SimpleCoordinate { on_curve: true, x: 1012, y: 1442 },
            SimpleCoordinate { on_curve: true, x: 1012, y: 1237 },
            SimpleCoordinate { on_curve: false, x: 920, y: 1296 },
            SimpleCoordinate { on_curve: false, x: 735, y: 1356 },
            SimpleCoordinate { on_curve: true, x: 641, y: 1356 },
            SimpleCoordinate { on_curve: false, x: 498, y: 1356 },
            SimpleCoordinate { on_curve: false, x: 332, y: 1223 },
            SimpleCoordinate { on_curve: true, x: 332, y: 1110 },
            SimpleCoordinate { on_curve: false, x: 332, y: 1011 },
            SimpleCoordinate { on_curve: false, x: 441, y: 907 },
            SimpleCoordinate { on_curve: true, x: 590, y: 872 },
            SimpleCoordinate { on_curve: true, x: 696, y: 848 },
            SimpleCoordinate { on_curve: false, x: 906, y: 799 },
            SimpleCoordinate { on_curve: false, x: 1098, y: 589 },
            SimpleCoordinate { on_curve: true, x: 1098, y: 408 },
            SimpleCoordinate { on_curve: false, x: 1098, y: 195 },
            SimpleCoordinate { on_curve: false, x: 834, y: -29 },
            SimpleCoordinate { on_curve: true, x: 582, y: -29 },
            SimpleCoordinate { on_curve: false, x: 477, y: -29 },
            SimpleCoordinate { on_curve: false, x: 265, y: 16 },
            SimpleCoordinate { on_curve: true, x: 158, y: 61 },
            SimpleCoordinate { on_curve: true, x: 158, y: 276 },
            SimpleCoordinate { on_curve: false, x: 273, y: 203 },
            SimpleCoordinate { on_curve: false, x: 478, y: 135 },
            SimpleCoordinate { on_curve: true, x: 582, y: 135 },
            SimpleCoordinate { on_curve: false, x: 735, y: 135 },
            SimpleCoordinate { on_curve: false, x: 905, y: 272 },
            SimpleCoordinate { on_curve: true, x: 905, y: 395 },
            SimpleCoordinate { on_curve: false, x: 905, y: 507 },
            SimpleCoordinate { on_curve: false, x: 788, y: 625 },
            SimpleCoordinate { on_curve: true, x: 643, y: 657 },
            SimpleCoordinate { on_curve: true, x: 535, y: 682 },
            SimpleCoordinate { on_curve: false, x: 327, y: 729 },
            SimpleCoordinate { on_curve: false, x: 139, y: 919 },
            SimpleCoordinate { on_curve: true, x: 139, y: 1079 },
            SimpleCoordinate { on_curve: false, x: 139, y: 1279 },
            SimpleCoordinate { on_curve: false, x: 408, y: 1520 },
            SimpleCoordinate { on_curve: true, x: 631, y: 1520 },
            SimpleCoordinate { on_curve: false, x: 717, y: 1520 },
            SimpleCoordinate { on_curve: false, x: 907, y: 1481 },
        ];

        let buf = font_buf();
        let font = Font::from_buffer(&buf).unwrap();

        let cmap: CMap = font.get_table().unwrap();
        let format4 = cmap.format4().unwrap();
        let glyph_id = format4.lookup_glyph_id('S' as u8 as u16).unwrap();

        let loca: Loca = font.get_table().unwrap();
        let glyph_offset = loca.at(glyph_id as usize);

        let glyf: Glyf = font.get_table().unwrap();
        let glyph = glyf.at_offset(glyph_offset as usize).unwrap();
        for (actual, &expected) in glyph.coordinates().zip(expecteds.iter()) {
            assert_eq!(actual, expected);
        }
    }
}
