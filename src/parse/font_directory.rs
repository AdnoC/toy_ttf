use nom::{be_u16, be_u32, IResult};
use num_traits::FromPrimitive;

use tables::font_directory::*;
use tables::TableTag;

pub fn parse_font_directory(i: &[u8]) -> IResult<&[u8], FontDirectory> {
    let (table_dir_start, offsets) = try_parse!(i, parse_offset_subtable);
    let font_dir = FontDirectory {
        offsets,
        table_dir_start,
    };

    Ok((table_dir_start, font_dir))
}

// named!(pub parse_font_directory<FontDirectory>,
//        do_parse!(
//            offsets: parse_offset_subtable >>
//            table_dirs: apply!(parse_table_directory, offsets.num_tables) >>
//            (FontDirectory { offsets, table_dirs })
//        )
// );

named_args!(parse_table_directory(num_entries: u16)<TableDirectory>,
map!(
    count!(table_directory_record, num_entries as usize),
    |entries| TableDirectory(entries)
)
);
named!(pub table_directory_record<TableDirRecord>,
       do_parse!(
           tag: table_tag >>
           check_sum: be_u32 >>
           offset: be_u32 >>
           length: be_u32 >>
           (TableDirRecord { tag, check_sum, offset, length })
       )
);

named!(
    table_tag<TableTag>,
    map_opt!(::nom::le_u32, TableTag::from_u32)
);

named!(
    parse_offset_subtable<OffsetSubtable>,
    do_parse!(
        scaler_type: parse_font_scaler_type
            >> num_tables: be_u16
            >> search_range: be_u16
            >> entry_selector: be_u16
            >> range_shift: be_u16 >> (OffsetSubtable {
            scaler_type,
            num_tables,
            search_range,
            entry_selector,
            range_shift
        }) // (OffsetSubtable { scaler_type, num_tables:0, search_range:0, entry_selector:0, range_shift:0 })
    )
);

named!(
    parse_font_scaler_type<ScalerType>,
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
        };
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
        let tags = [0x74727565, 0x00010000, 0x74797031, 0x4F54544F];
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
