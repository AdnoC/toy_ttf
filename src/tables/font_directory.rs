use tables::TableTag;

#[derive(Debug)]
pub struct FontDirectory {
    pub offsets: OffsetSubtable,
    pub table_dirs: TableDirectory,
}

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
