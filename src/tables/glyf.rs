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
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
}

pub struct Glyph<'a> {
    header: Header,
    desc: Description<'a>,
}
impl<'a> Glyph<'a> {
    // TODO: Name for compoind glyph
    pub fn coordinates(&self) -> SimpleCoordinates<'a> {
        use std::marker::PhantomData;

        let simp = match self.desc {
            Description::Simple(ref simp) => simp,
            _ => unimplemented!()
        };

        // TODO: Test

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
        const ON_CURVE_POINT                               = 0b00000001;
        const X_SHORT_VEC                                  = 0b00000010;
        const Y_SHORT_VEC                                  = 0b00000100;
        const REPEAT_FLAG                                  = 0b00001000;
        const X_IS_SAME                                    = 0b00010000;
        const POSITIVE_X_SHORT_VECTOR                      = 0b00010000;
        const Y_IS_SAME                                    = 0b00100000;
        const POSITIVE_Y_SHORT_VECTOR                      = 0b00100000;
        const RESERVED                                     = 0b11000000;
    }
}

#[derive(Debug)]
pub struct SimpleCoordinate {
    on_curve: bool,
    x: isize,
    y: isize,
}
pub struct SimpleCoordinates<'a> {
    flags: DynArr<'a, SimpleFlags>,
    delta_xs: &'a [u8],
    delta_ys: &'a [u8],
    repeat_count: u8,
    // coordinate values are relative to the previous point
    x: isize,
    y: isize,
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
            self.repeat_count = u8::parse(count_buf.0).1;
        }

        let on_curve = flag.contains(SimpleFlags::ON_CURVE_POINT);

        let dx = if flag.contains(SimpleFlags::X_SHORT_VEC) {
            let (rest, dx) = u8::parse(self.delta_xs);
            let dx = dx as isize;
            self.delta_xs = rest;

            if flag.contains(SimpleFlags::POSITIVE_X_SHORT_VECTOR) {
                dx
            } else {
                -dx
            }
        } else if !flag.contains(SimpleFlags::X_IS_SAME) {
            let (rest, dx) = i16::parse(self.delta_xs);
            self.delta_xs = rest;
            dx as isize
        } else { 0 };

        let dy = if flag.contains(SimpleFlags::Y_SHORT_VEC) {
            let (rest, dy) = u8::parse(self.delta_ys);
            let dy = dy as isize;
            self.delta_ys = rest;

            if flag.contains(SimpleFlags::POSITIVE_Y_SHORT_VECTOR) {
                dy
            } else {
                -dy
            }
        } else if !flag.contains(SimpleFlags::Y_IS_SAME) {
            let (rest, dy) = i16::parse(self.delta_ys);
            self.delta_ys = rest;
            dy as isize
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
    #[test]
    fn simple_coordinates() {
        // TODO
    }
}
