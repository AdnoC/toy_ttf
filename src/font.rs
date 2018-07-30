use parse::font_directory::parse_font_directory;
use parse::Parse;
use tables::font_directory::FontDirectory;
use tables::loca::Loca;
use tables::glyf::Glyph;
use tables::{ParseTableError, ParseTableErrorInner, PrimaryTable};

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
        use tables::cmap::CMap;
        use tables::glyf::Glyf;
        use std::u8;
        assert!(code_point < u8::MAX as char); // Only ascii for now
        let loca: Loca = self.get_table()?;
        let cmap: CMap = self.get_table()?;
        let format4 = cmap.format4()?;
        let glyf: Glyf = self.get_table()?;

        let glyph_id = format4.lookup_glyph_id(code_point as u8 as u16)?;
        let glyph_offset = loca.at(glyph_id as usize);
        glyf.at_offset(glyph_offset as usize)
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

        let loca_slice = self.get_table_slice::<Loca>()?;

        let size = (num_glyphs + 1) * match format {
            IndexToLocFormat::Short => <u16 as Parse>::approx_file_size(),
            IndexToLocFormat::Long => <u32 as Parse>::approx_file_size(),
        };

        let loca_buf = &loca_slice[..size];
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
