use parse::font_directory::parse_font_directory;
use parse::Parse;
use tables::font_directory::FontDirectory;
use tables::loca::Loca;
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


    fn get_table_start<T: PrimaryTable>(&self) -> Option<&'a [u8]> {
        self.font_dir
            .table_record::<T>()
            .map(|record| &self.buf[(record.offset as usize)..])
    }

}

pub trait GetTable<T> {
    fn get_table(&self) -> Option<T> ;
}

impl<'a, T: Parse<'a> + PrimaryTable> GetTable<T> for Font<'a> {
    fn get_table(&self) -> Option<T> {
        let table_start = self.get_table_start::<T>()?;

        let table = T::parse(table_start).1;
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

        let loca_start = self.get_table_start::<Loca>()?;

        let size = (num_glyphs + 1) * match format {
            IndexToLocFormat::Short => <u16 as Parse>::approx_file_size(),
            IndexToLocFormat::Long => <u32 as Parse>::approx_file_size(),
        };

        let loca_buf = &loca_start[..size];
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
