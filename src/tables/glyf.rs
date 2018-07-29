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
        let mut points_left = simp.end_points_of_contours.at(last_point_index_offset) as usize;
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
            } else if flag.contains(SimpleFlags::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR) {
                0
            } else {
                i16::approx_file_size()
            };
            let y_size = if flag.contains(SimpleFlags::Y_SHORT_VEC) {
                u8::approx_file_size()
            } else if flag.contains(SimpleFlags::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR) {
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
        let (xs, rest) = rest.0.split_at(xs_len);
        let ys = &rest[..ys_len];

        SimpleCoordinates {
            flags, xs, ys
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
        const ON_CURVE                                     = 0b00000001;
        const X_SHORT_VEC                                  = 0b00000010;
        const Y_SHORT_VEC                                  = 0b00000100;
        const REPEAT_FLAG                                  = 0b00001000;
        const X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR         = 0b00010000;
        const Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR         = 0b00100000;
        const RESERVED                                     = 0b11000000;
    }
}

#[derive(Debug)]
pub struct SimpleCoordinate {
    on_curve: bool,
    x: i16,
    y: i16,
}
pub struct SimpleCoordinates<'a> {
    flags: DynArr<'a, SimpleFlags>,
    xs: &'a [u8],
    ys: &'a [u8],
}
impl<'a> Iterator for SimpleCoordinates<'a> {
    type Item = SimpleCoordinate;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
