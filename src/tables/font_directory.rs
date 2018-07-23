use tables::TableTag;

#[derive(Debug)]
pub struct FontDirectory<'file> {
    pub offsets: OffsetSubtable,
    pub table_dir_start: &'file [u8],
}

impl<'a> FontDirectory<'a> {
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

// impl<'a> Iterator for TableRecords<'a> {
//     type Item = 
// }

#[derive(Debug)]
pub struct TableDirectory(pub Vec<TableDirEntry>);

#[derive(Debug)]
pub struct TableDirEntry {
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
