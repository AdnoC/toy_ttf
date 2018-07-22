use tables::name::*;
use nom::{be_u16, IResult, Offset};

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
    println!("eaten = {}", eaten);
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
        length,
        offset,
        name,
    };
    Ok((i3, (i1, nr)))
}
