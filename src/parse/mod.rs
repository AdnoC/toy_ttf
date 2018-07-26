mod name;
mod primitives;
pub(crate) mod font_directory;

pub trait Parse<'a> {
    /// Size of the object when serialized in the file
    fn file_size(&self) -> usize;
    fn parse(buf: &'a [u8]) -> (&'a [u8], Self);
}

// TODO: Use to verify font tables
#[allow(dead_code)]
fn table_check_sum(table: &[u32]) -> u32 {
    table.iter().sum()
    // C version
    // uint32 CalcTableChecksum(uint32 *table, uint32 numberOfBytesInTable) {
    //     uint32 sum = 0;
    //     uint32 nLongs = (numberOfBytesInTable + 3) / 4;
    //     while (nLongs-- > 0)
    //         sum += *table++;
    //     return sum;
    // }
}

pub fn load_font(font_buf: &[u8]) {
    { // TESTING
        test_parse(font_buf).expect("Test parse failed");

        // let font_buf = &font_buf[0..1024];

        // let fd = parse_font_directory(font_buf).unwrap();
        // println!("fd = {:#?}", fd);
        // let off = parse_offset_subtable(font_buf);
        // let st = off.and_then(|inp| table_directory_entry(inp.0));

        // let st = parse_font_directory(font_buf);
        // match st {
        //     Ok(val) => println!("ok {:#?}", val.1),
        //     Err(e) => println!("err {:?}", e),
        // };

        // FIXME: WILL BLOW UP, just for debug
        // tables::tables_parse::parse_name_table(font_buf);
        // tables::tables_parse::parse_name_record(font_buf, 0);
    }
}

fn test_parse(i: &[u8]) -> ::nom::IResult<&[u8], ()> {
    unimplemented!()
    //
    // use tables::TableTag;
    //
    // use nom::Offset;
    // let (i1, fd) = try_parse!(i, font_directory::parse_font_directory);
    // let eaten = i.offset(i1);
    // let name_offset = fd.table_dirs.0.iter()
    //     .find(|tdr| tdr.tag == TableTag::Name)
    //     .map(|tdr| tdr.offset)
    //     .expect("Coulnd't find name table");
    //
    // let name_offset = name_offset - eaten as u32;
    // let (i_name, _) = try_parse!(i1, take!(name_offset));
    // let (i_fin, nt) = try_parse!(i_name, name::parse_name_table);
    //
    // println!("name_table = {:#?}", nt);
    //
    // Ok((i_fin, ()))
}
