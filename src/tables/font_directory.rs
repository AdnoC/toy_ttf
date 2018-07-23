use tables::{PrimaryTable, TableTag};

#[derive(Debug)]
pub struct FontDirectory<'file> {
    pub offsets: OffsetSubtable,
    pub table_dir_start: &'file [u8],
}

impl<'a> FontDirectory<'a> {
    pub fn table_record<T: PrimaryTable<'a>>(&self) -> Option<TableDirRecord> {
        self.table_records()
            .find(|record| record.tag == T::tag())
    }
    pub fn table_records(&self) -> TableRecords<'a> {
        TableRecords {
            next_record: self.table_dir_start,
            num_left: self.offsets.num_tables,
        }
    }
}

pub struct TableRecords<'file> {
    next_record: &'file [u8],
    num_left: u16,
}

impl<'a> Iterator for TableRecords<'a> {
    type Item = TableDirRecord;

    fn next(&mut self) -> Option<Self::Item> {
        use parse::font_directory::table_directory_record;

        if self.num_left < 1 {
            return None;
        }

        let (next_record, record) = table_directory_record(self.next_record).ok()?;

        self.num_left -= 1;
        self.next_record = next_record;

        Some(record)
    }
}

#[derive(Debug)]
pub struct TableDirectory(pub Vec<TableDirRecord>);

#[derive(Debug)]
pub struct TableDirRecord {
    pub tag: TableTag,
    pub check_sum: u32,
    pub offset: u32,
    pub length: u32,
}

#[derive(Debug)]
pub struct OffsetSubtable {
    pub scaler_type: ScalerType,
    pub num_tables: u16,
    pub search_range: u16, // (max power of two that is <= num_tables) * 16
    pub entry_selector: u16, // log_2(max power of two that is <= num_tables)
    pub range_shift: u16, // num_tables * 16 - search_range
}

#[derive(Debug, PartialEq, Eq)]
pub enum ScalerType {
    TTF, // 'true'
    PostScript, // 'typ1'
    OpenType, // 'OTTO'
}

#[cfg(test)]
mod tests {
    use font::Font;

    fn load_file() -> Vec<u8> {
        let name = "fonts/DejaVuSansMono.ttf";
        use std::fs::File;
        use std::io::BufReader;
        use std::io::prelude::*;

        let file = File::open(name).expect("unable to open file");

        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data).expect("error reading file");

        data
    }

    #[test]
    fn table_dir_record_iteration() {
        let buf = load_file();
        let font = Font::from_buffer(&buf).unwrap();

        let num_iter_results = font.font_dir.table_records().count();
        assert_eq!(num_iter_results, font.font_dir.offsets.num_tables as usize);
    }
}
