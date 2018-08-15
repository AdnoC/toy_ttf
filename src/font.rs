use parse::font_directory::parse_font_directory;
use parse::Parse;
use tables::font_directory::FontDirectory;
use tables::loca::Loca;
use tables::glyf::Glyph;
use tables::{ParseTableError, ParseTableErrorInner, PrimaryTable};
use render::*;
use image::GrayImage;
use math::Affine;

pub struct Font<'file> {
    buf: &'file [u8],
    pub(crate) font_dir: FontDirectory<'file>,
}

impl<'a> Font<'a> {
    pub fn from_buffer(buf: &'a [u8]) -> Result<Font<'a>, ReadFontError> {
        let font_dir = parse_font_directory(buf)?.1;

        let font = Font { buf, font_dir };

        Ok(font)
    }


    fn get_table_slice<T: PrimaryTable>(&self) -> Option<&'a [u8]> {
        self.font_dir
            .table_record::<T>()
            .map(|record|
                 &self.buf[(record.offset as usize)..(record.offset as usize +
                                                      record.length as usize)])
    }

    pub fn get_glyph(&self, code_point: char) -> Option<Glyph<'a>> {
        use tables::cmap::{CMap, Format4, Format12};
        use tables::glyf::Glyf;
        use std::u16;

        let loca: Loca = self.get_table()?;
        let cmap: CMap = self.get_table()?;
        let glyf: Glyf = self.get_table()?;

        let glyph_id = if (code_point as u32) < (u16::MAX as u32) {
            let format4: Format4 = cmap.get_format()?;
            format4.lookup_glyph_id(code_point as u32 as u16)? as u32
        } else {
            let format12: Format12 = cmap.get_format()?;
            format12.lookup_glyph_id(code_point as u32)?
        };
        println!("[{}] Glyph_id = {}", code_point, glyph_id);
        let glyph_offset = loca.at(glyph_id as usize)?;
        println!("[{}] Glyph_offset = {}", code_point, glyph_offset);

        glyf.at_offset(glyph_offset as usize)
    }

    pub fn render_glyph(&self, glyph: Glyph<'a>, size: usize) -> GrayImage {
        use tables::head::Head;

        const PADDING: u32 = 0;
        let width = glyph.header.x_max - glyph.header.x_min;
        let height = glyph.header.y_max - glyph.header.y_min;
        let x_shift = (PADDING as i16 / 2) - glyph.header.x_min;
        let y_shift = (PADDING as i16 / 2) - glyph.header.y_min;
        let affine = Affine::translation(x_shift, y_shift);

        let head: Head = self.get_table().unwrap();
        let scale = size as f32 / head.units_per_em as f32;
        let affine = Affine::scale(scale, scale) * affine;
        let width = (width as f32 * scale).ceil();
        let height = (height as f32 * scale).ceil();

        println!("Raster (w, h) = ({}, {})", width as u32 + PADDING, height as u32 + PADDING);
        let mut raster = FillInRaster::new(width as u32 + PADDING, height as u32 + PADDING);
        // let mut raster = OutlineRaster::new(width as u32 + PADDING, height as u32 + PADDING);

        self.render_glyph_inner(&mut raster, affine, glyph);

        raster.into_dynamic().to_luma()
    }

    fn render_glyph_inner(&self, raster: &mut impl Raster, affine: Affine, glyph: Glyph<'a>) {
        use tables::glyf::{Coordinate, Description};

        match glyph.desc {
            Description::Simple(glyph) => {
                for contour in glyph.contours() {
                    for (start, end) in FlattenedDrawCommands::from_coordinates(contour.into_iter()) {
                        raster.add_line(affine * start, affine * end);
                    }
                }
            },
            Description::Composite(glyph) => {
                use tables::glyf::Glyf;
                use tables::loca::Loca;
                for (sub_idx, sub_affine) in glyph.coordinates() {
                    let glyf: Glyf = self.get_table().unwrap();
                    let loca: Loca = self.get_table().unwrap();
                    let offset = loca.at(sub_idx).unwrap();
                    let sub_glyph = glyf.at_offset(offset as usize).unwrap();

                    self.render_glyph_inner(raster, affine * sub_affine, sub_glyph);
                }
            },
        };
    }
}

pub trait GetTable<T> {
    fn get_table(&self) -> Option<T> ;
}

impl<'a, T: Parse<'a> + PrimaryTable> GetTable<T> for Font<'a> {
    fn get_table(&self) -> Option<T> {
        let table_slice = self.get_table_slice::<T>()?;

        let table = T::parse(table_slice).1;
        Some(table)
    }
}

impl<'a> GetTable<Loca<'a>> for Font<'a> {
    fn get_table(&self) -> Option<Loca<'a>> {
        use parse::DynArr;
        use std::marker::PhantomData;
        use tables::head::{Head, IndexToLocFormat};
        use tables::loca::{L, S};
        use tables::maxp::MaxP;
        let head: Head = self.get_table()?;
        let format = head.index_to_loc_format;

        let maxp: MaxP = self.get_table()?;
        let num_glyphs = maxp.num_glyphs as usize;

        let loca_buf = self.get_table_slice::<Loca>()?;

        let loca = match format {
            IndexToLocFormat::Short => Loca::Short(S(DynArr(loca_buf, PhantomData))),
            IndexToLocFormat::Long => Loca::Long(L(DynArr(loca_buf, PhantomData))),
        };

        Some(loca)
    }

}

// FIXME: Change from just a typedef of IResult's error type
type ReadFontError<'a> = ::nom::Err<&'a [u8], u32>;
// enum ReadFontError {
//
// }
