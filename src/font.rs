use tables::{ParseTableError, PrimaryTable, ParseTableErrorInner};
use tables::font_directory::FontDirectory;
use parse::font_directory::parse_font_directory;
use parse::Parse;

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

    pub fn get_table<T: Parse<'a> + PrimaryTable>(&self) -> Option<T> {
        let table_start = self.get_table_start::<T>()?;

        let table = T::parse(table_start).1;
        Some(table)
    }

    fn get_table_start<T: PrimaryTable>(&self) -> Option<&'a [u8]> {
        self.font_dir.table_record::<T>()
            .map(|record| &self.buf[(record.offset as usize)..])
    }

}


// FIXME: Change from just a typedef of IResult's error type
type ReadFontError<'a> = ::nom::Err<&'a [u8], u32>;
// enum ReadFontError {
//
// }
