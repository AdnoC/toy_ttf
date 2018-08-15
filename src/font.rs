use parse::font_directory::parse_font_directory;
use parse::Parse;
use tables::font_directory::FontDirectory;
use tables::loca::Loca;
use tables::glyf::Glyph;
use tables::hmtx::HMTX;
use tables::{ParseTableError, ParseTableErrorInner, PrimaryTable};
use render::*;
use render::compositor::{GlyphPlacementMetrics};
use image::GrayImage;
use math::Affine;

// TODO: Canonical glyph_id type

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

    pub fn get_glyph_for_id(&self, glyph_id: u32) -> Option<Glyph<'a>> {
        use tables::glyf::Glyf;
        let loca: Loca = self.get_table()?;
        let glyf: Glyf = self.get_table()?;

        let glyph_offset = loca.at(glyph_id as usize)?;
        println!("\tGlyph_offset = {}", glyph_offset);

        glyf.at_offset(glyph_offset as usize)
    }

    pub fn get_glyph(&self, code_point: char) -> Option<Glyph<'a>> {
        let glyph_id = self.get_glyph_id(code_point)?;

        println!("[{}] Glyph_id = {}", code_point, glyph_id);
        self.get_glyph_for_id(glyph_id)
    }

    pub fn get_glyph_id(&self, code_point: char) -> Option<u32> {
        use tables::cmap::{CMap, Format4, Format12};
        use std::u16;

        let cmap: CMap = self.get_table()?;

        if (code_point as u32) < (u16::MAX as u32) {
            let format4: Format4 = cmap.get_format()?;
            format4.lookup_glyph_id(code_point as u32 as u16).map(|val| val as u32)
        } else {
            let format12: Format12 = cmap.get_format()?;
            format12.lookup_glyph_id(code_point as u32)
        }
    }

    pub fn render_glyph(&self, glyph: Glyph<'a>, size: usize) -> GrayImage {
        use tables::head::Head;

        let width = glyph.header.x_max - glyph.header.x_min;
        let height = glyph.header.y_max - glyph.header.y_min;
        let x_shift = -glyph.header.x_min;
        let y_shift = -glyph.header.y_min;
        let affine = Affine::translation(x_shift, y_shift);

        let head: Head = self.get_table().unwrap();
        let scale = size as f32 / head.units_per_em as f32;
        let affine = Affine::scale(scale, scale) * affine;
        let width = (width as f32 * scale).ceil();
        let height = (height as f32 * scale).ceil();

        println!("Raster (w, h) = ({}, {})", width as u32, height as u32);
        let mut raster = FillInRaster::new(width as u32, height as u32);

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

    fn placement_metrics(&self, code_point: char, size: usize) -> Option<GlyphPlacementMetrics> {
        use tables::head::Head;

        let glyph_id = self.get_glyph_id(code_point)?;

        let shift = {
            let glyph = self.get_glyph_for_id(glyph_id)?;

            let width = glyph.header.x_max - glyph.header.x_min;
            let height = glyph.header.y_max - glyph.header.y_min;
            let x_shift = -glyph.header.x_min;
            let y_shift = -glyph.header.y_min;
            let affine = Affine::translation(x_shift, y_shift);

            let head: Head = self.get_table().unwrap();
            let scale = size as f32 / head.units_per_em as f32;
            let affine = Affine::scale(scale, scale) * affine;

            affine.translation
        };

        let horiz_metrics = {
            let hmtx: HMTX = self.get_table()?;
            hmtx.metrics_for_glyph(glyph_id)
        };


        unimplemented!()
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

        // TODO: Move this into tables/loca.rs

        let loca = match format {
            IndexToLocFormat::Short => Loca::Short(S(DynArr(loca_buf, PhantomData))),
            IndexToLocFormat::Long => Loca::Long(L(DynArr(loca_buf, PhantomData))),
        };

        Some(loca)
    }

}

impl<'a> GetTable<HMTX<'a>> for Font<'a> {
    fn get_table(&self) -> Option<HMTX<'a>> {
        use tables::hhea::HHEA;

        let hhea: HHEA = self.get_table()?;
        let num_horiz_metrics = hhea.num_horiz_metrics;

        let hmtx_buf = self.get_table_slice::<HMTX>()?;

        let hmtx = HMTX::parse_metrics(hmtx_buf, num_horiz_metrics);

        Some(hmtx)
    }
}

// FIXME: Change from just a typedef of IResult's error type
type ReadFontError<'a> = ::nom::Err<&'a [u8], u32>;
// enum ReadFontError {
//
// }
