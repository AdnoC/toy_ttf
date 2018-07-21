use nom::{be_u16, be_u32};
use num_traits::FromPrimitive;

use tables::TableTag;

mod name;


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

        parse_offsets(font_buf);
    }
}

fn parse_offsets(font_buf: &[u8]) {
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

fn test_parse(i: &[u8]) -> ::nom::IResult<&[u8], ()> {
    use nom::Offset;
    let (i1, fd) = try_parse!(i, parse_font_directory);
    let eaten = i.offset(i1);
    let name_offset = fd.table_dirs.0.iter()
        .find(|tdr| tdr.tag == TableTag::Name)
        .map(|tdr| tdr.offset)
        .expect("Coulnd't find name table");

    let name_offset = name_offset - eaten as u32;
    let (i_name, _) = try_parse!(i1, take!(name_offset));
    let (i_fin, nt) = try_parse!(i_name, name::parse_name_table);

    println!("name_table = {:#?}", nt);

    Ok((i_fin, ()))
}

#[derive(Debug)]
struct FontDirectory {
    offsets: OffsetSubtable,
    table_dirs: TableDirectory,
}

named!(parse_font_directory<FontDirectory>,
       do_parse!(
           offsets: parse_offset_subtable >>
           table_dirs: apply!(parse_table_directory, offsets.num_tables) >>
           (FontDirectory { offsets, table_dirs })
       )
);

#[derive(Debug)]
struct TableDirectory(Vec<TableDirEntry>);

#[derive(Debug)]
struct TableDirEntry {
    tag: TableTag,
    check_sum: u32,
    offset: u32,
    length: u32,
}

named_args!(parse_table_directory(num_entries: u16)<TableDirectory>,
map!(
    count!(table_directory_entry, num_entries as usize),
    |entries| TableDirectory(entries)
)
);
named!(table_directory_entry<TableDirEntry>,
       do_parse!(
           tag: table_tag >>
           check_sum: be_u32 >>
           offset: be_u32 >>
           length: be_u32 >>
           (TableDirEntry { tag, check_sum, offset, length })
       )
);

named!(table_tag<TableTag>,
       map_opt!(::nom::le_u32,
                TableTag::from_u32)
);

#[derive(Debug)]
struct OffsetSubtable {
    scaler_type: ScalerType,
    num_tables: u16,
    search_range: u16, // (max power of two that is <= num_tables) * 16
    entry_selector: u16, // log_2(max power of two that is <= num_tables)
    range_shift: u16, // num_tables * 16 - search_range
}

#[derive(Debug, PartialEq, Eq)]
enum ScalerType {
    TTF, // 'true'
        PostScript, // 'typ1'
        OpenType, // 'OTTO'
}

named!(parse_offset_subtable<OffsetSubtable>,
do_parse!(
    scaler_type: parse_font_scaler_type >>
    num_tables: be_u16 >>
    search_range: be_u16 >>
    entry_selector: be_u16 >>
    range_shift: be_u16 >>
    (OffsetSubtable { scaler_type, num_tables, search_range, entry_selector, range_shift })
    // (OffsetSubtable { scaler_type, num_tables:0, search_range:0, entry_selector:0, range_shift:0 })
)
);

named!(parse_font_scaler_type<ScalerType>,
switch!(be_u32,
        0x74727565 => value!(ScalerType::TTF) |
        0x00010000 => value!(ScalerType::TTF) | // MUST be used for Windows or Adobe products
        0x74797031 => value!(ScalerType::PostScript) |
        0x4F54544F => value!(ScalerType::OpenType)
)
);

#[cfg(test)]
mod tests {
    use super::*;
    use byte_conv::As as AsBytes;
    use nom::IResult;
    macro_rules! assert_parse_equal {
        ($expected:expr, $actual:expr) => {
            let val = $actual.unwrap().1;
            assert_eq!(val, $expected);
        }
    }


    macro_rules! make_byte_slice {
        ($($val:expr),*) => {
            {
                // use byte_conv::As as AsBytes;
                let mut v: Vec<u8> = Vec::new();
                $(
                    let val_be = $val.to_be();
                    let val_arr = val_be.as_bytes();
                    v.extend(val_arr);
                )*
                    v
            }
        }
    }
    #[test]
    fn parse_scaler_type() {
        let tags = [
            0x74727565,
            0x00010000,
            0x74797031,
            0x4F54544F,
        ];
        let expecteds = [
            ScalerType::TTF,
            ScalerType::TTF,
            ScalerType::PostScript,
            ScalerType::OpenType,
        ];
        for (tag, expected) in tags.into_iter().zip(expecteds.into_iter()) {
            println!("tag = {:x}, exp = {:?}", tag, expected);
            let buf: &[u8] = &make_byte_slice![*tag as u32];
            assert_parse_equal!(*expected, parse_font_scaler_type(buf));
        }
    }
}
