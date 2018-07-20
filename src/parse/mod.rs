use nom::{be_u16, be_u32};

pub fn load_font(font_buf: &[u8]) {
    { // TESTING

        parse_offsets(font_buf); // 
    }
}

fn parse_offsets(font_buf: &[u8]) {
    let st = offset_subtable(font_buf);
    println!("{:#?}", st.unwrap().1);
}

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

named!(offset_subtable(&[u8]) -> OffsetSubtable,
do_parse!(
    scaler_type: font_scaler_type >>
    num_tables: be_u16 >>
    search_range: be_u16 >>
    entry_selector: be_u16 >>
    range_shift: be_u16 >>
    (OffsetSubtable { scaler_type, num_tables, search_range, entry_selector, range_shift })
)
);

named!(font_scaler_type(&[u8]) -> ScalerType,
    switch!(be_u32,
            0x74727565 => value!(ScalerType::TTF) |
            0x00010000 => value!(ScalerType::TTF) |
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
            assert_parse_equal!(*expected, font_scaler_type(buf));
        }
    }
}
