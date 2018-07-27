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
    fn at_offset(&self, offset: usize) -> Option<Glyph<'a>> {
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
            Description::Composite // TODO
        };

        Some(Glyph {
            header, desc,
        })
    }
}

#[derive(Debug, Parse)]
struct Header {
    number_of_contours: i16,
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
}

struct Glyph<'a> {
    header: Header,
    desc: Description<'a>,
}

enum Description<'a> {
    Simple(SimpleGlyph<'a>),
    Composite, // TODO
}

struct SimpleGlyph<'a> {
    end_points_of_contours: DynArr<'a, u16>,
    instruction_length: u16,
    instructions: DynArr<'a, u8>,
    coords: BufView<'a, u8>,
    // These are derived from coords
    // flags: BufView<'a, u8>,
    // x_coords: BufView<'a, u8>,
    // y_coords: BufView<'a, u8>,
}
