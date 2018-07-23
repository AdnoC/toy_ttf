use tables::{ParseTableError, PrimaryTable, ParseTableErrorInner};
use tables::font_directory::FontDirectory;
use parse::font_directory::parse_font_directory;

pub struct Font<'file> {
    buf: &'file [u8],
    pub(crate) font_dir: FontDirectory<'file>,
}

impl<'a> Font<'a> {
    pub fn from_buffer(buf: &'a [u8]) -> Result<Font<'a>, ReadFontError> {
        let font_dir = parse_font_directory(buf)?.1;

        let font = Font {
            buf, font_dir
        };

        Ok(font)
    }

    pub fn get_table<T: PrimaryTable<'a>>(&self) -> Result<T, ParseTableError> {
        let table_start = self.get_table_start::<T>()
            .ok_or(::nom::Err::Error(error_position!(self.buf, ::nom::ErrorKind::Custom(ParseTableErrorInner::TableNotFound))))?;

        T::parse(table_start)
    }

    fn get_table_start<T: PrimaryTable<'a>>(&self) -> Option<&'a [u8]> {
        self.font_dir.table_record::<T>()
            .map(|record| &self.buf[(record.offset as usize)..])
    }

}


// FIXME: Change from just a typedef of IResult's error type
type ReadFontError<'a> = ::nom::Err<&'a [u8], u32>;
// enum ReadFontError {
//
// }
