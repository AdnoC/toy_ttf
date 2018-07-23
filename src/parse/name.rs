use tables::name::*;
use nom::{be_u32, be_u16, IResult, Offset};

// TODO: Handle all these encodings
// use encoding::all::{
//     MAC_ROMAN,
//     UTF_16BE, // UCS-2
//     WINDOWS_31J, // ShiftJIS
//     GB18030, // PRC
//     BIG5_2003, // Big5
//     WINDOWS_949, // Wansung
//     // JOHAB - Not supported by encoding crate
// };
// UCS-4 = utf-32

fn parse_utf32(i: &[u8]) -> IResult<&[u8], String> {
    use std::char;
    do_parse!(i,
               res: fold_many1!(
                   map_opt!(complete!(be_u32), char::from_u32),
                   String::with_capacity(i.len() / 4),
                   |mut p_str: String, ch: char| { p_str.push(ch); p_str }
               ) >>
               (res)
    )
}

pub fn parse_name_table(i: &[u8]) -> IResult<&[u8], NameTable> {
    named!(partial_table<(u16, u16)>,
    do_parse!(
        // We only handle format 0 tables
        _format: verify!(be_u16, |val| val == 0) >>
        count: be_u16 >>
        string_offset: be_u16 >>
        ((count, string_offset))
        )
    );

    let (i1, (count, string_offset)) = try_parse!(i, partial_table);

    let eaten = i.offset(i1);
    let new_offset = (string_offset as usize - eaten) as u16;
    let (i2, records) = try_parse!(i1, apply!(parse_name_records, count, new_offset));
    let nt = NameTable {
        string_offset, records
    };
    Ok((i2, nt))
}

// TODO: Make a combinator out of this
fn parse_name_records(i: &[u8], count: u16, names_start: u16) -> IResult<&[u8], Vec<NameRecord>> {
    let mut records = Vec::with_capacity(count as usize);
    let mut furthest = i;
    let mut next_pos = i;
    for _ in 0..count {
        let eaten = i.offset(next_pos) as u16;
        println!("eaten = {}", eaten);
        let names_start = names_start - eaten;
        let (post_name, (post_record, nr)) = try_parse!(next_pos, apply!(parse_name_record, names_start));

        records.push(nr);
        next_pos = post_record;
        if i.offset(furthest) < i.offset(post_name) {
            furthest = post_name;
        }
    }
    Ok((furthest, records))
}
fn parse_name_record(i: &[u8], names_start: u16) -> IResult<&[u8], (&[u8], NameRecord)> {
    named!(partial_record<(u16, u16, u16, u16, u16, u16)>,
    do_parse!(
        platform_id: be_u16 >>
        platform_specific_id: be_u16 >>
        language_id: be_u16 >>
        name_id: be_u16 >>
        length: be_u16 >>
        offset: be_u16 >>
        ((platform_id, platform_specific_id, language_id, name_id, length, offset))
        )
    );

    let (i1, (platform_id, platform_specific_id, language_id,
              name_id, length, offset)) = try_parse!(i, partial_record);

    let eaten = i.offset(i1) as u16;
    let offset_to_name = names_start - eaten + offset;
    // TODO: Make this a parser
    let (i2, _) = try_parse!(i1, take!(offset_to_name));
    let (i3, name) = try_parse!(i2, recognize!(take!(length)));
    let (i3, name) = match platform_id {
        2 => return Err(::nom::Err::Error(error_position!(i3, ::nom::ErrorKind::Tag))),
        0 => try_parse!(i3, expr_res!(NameString::new_unicode_from_raw(name))),
        3 => try_parse!(i3, expr_res!(NameString::new_microsoft_from_raw(name))),
        _ => try_parse!(i3, value!(NameString::new_other_from_raw(name))),
        // 1 should be MacOSRoman encoding?
    };

    let nr = NameRecord {
        platform_id,
        platform_specific_id,
        language_id,
        name_id,
        offset,
        name,
    };
    Ok((i3, (i1, nr)))
}

#[cfg(test)]
mod tests {
    #[test]
    fn utf32_decoder() {
        use super::parse_utf32;
        #[cfg(target_endian = "little")]
        fn to_be(inp: &str) -> Vec<u8> {
            inp.chars()
                .map(|ch| ch as u32)
                .map(|ch| (ch, ch >> 8, ch >> 16, ch >> 24))
                .map(|(u32_1, u32_2, u32_3, u32_4)| (u32_1 as u8, u32_2 as u8,
                                                    u32_3 as u8, u32_4 as u8))
                .map(|(b1, b2, b3, b4)| (b4, b3, b2, b1))
                .fold(Vec::with_capacity(inp.len() * 4),
                    |mut acc, (b1, b2, b3, b4)| {
                        acc.push(b1);
                        acc.push(b2);
                        acc.push(b3);
                        acc.push(b4);
                        acc
                    })
        }
        #[cfg(target_endian = "big")]
        fn to_be(inp: &str) -> Vec<u8> {
            inp.bytes().collect()
        }

        let mess = "Hello, World!";
        let jap_mess = "In japanese: こんにちは世界！";

        // Just make sure that we have some multi-byte chars in the japanes message
        assert!(jap_mess.len() >  jap_mess.chars().count());

        let be_mess = to_be(&mess);
        let parsed_mess = parse_utf32(&be_mess).unwrap().1;
        assert_eq!(mess, parsed_mess);

        let be_jap_mess = to_be(&jap_mess);
        let parsed_jap_mess = parse_utf32(&be_jap_mess).unwrap().1;
        assert_eq!(jap_mess, parsed_jap_mess);
    }
}
