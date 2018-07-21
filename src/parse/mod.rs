use nom::{be_u16, be_u32};
use num_traits::FromPrimitive;



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
    let (i_fin, nt) = try_parse!(i_name, tables::tables_parse::parse_name_table);

    println!("name_table = {:#?}", nt);

    Ok((i_fin, ()))
}

// Should be its own file
mod tables {
    use super::TableTag;
    use widestring::WideString;
    use std::str::Utf8Error;
    use byte_slice_cast::{AsSliceOf, Error};

    // FIXME: Should be put somewhere more sensible
    // TODO: Should hold reference to the string
    enum NameString {
        Unicode(String),
        Microsoft(WideString),
        Other(Vec<u8>)
    }
    impl ::std::fmt::Debug for NameString {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            use self::NameString::*;
            match self {
                Unicode(name) => f.debug_tuple("Unicode").field(name).finish(),
                Microsoft(name) => f.debug_tuple("Microsoft").field(&name.to_string_lossy()).finish(),
                Other(raw_name) => f.debug_tuple("Other").field(raw_name).finish()
            }
        }
    }

    impl NameString {
        fn new_unicode_from_raw(s: &[u8]) -> Result<NameString, Utf8Error> {
            // FIXME: UNICODE IS IN UTF-16 ENCODING, NOT UTF-8
            use std::str;
            str::from_utf8(s)
                .map(|s| s.to_string())
                .map(|s| NameString::Unicode(s))
        }

        fn new_microsoft_from_raw(s: &[u8]) -> Result<NameString, Error> { // TODO: Make a better error

            let manual_convert = |s: &[u8]| {
                let wchar_buf: Vec<u16> = s.chunks(2)
                    .map(|bytes| (bytes[0], bytes[1]))
                    .map(|(hi, lo)| (hi as u16) << 8 | lo as u16)
                    .collect();
                let ms_string = WideString::from_vec(wchar_buf);
                Ok(NameString::Microsoft(ms_string))
            };

            // If we can get away with it, try just casting the slice first
            #[cfg(target_endian = "big")]
            {
                if let Ok(wchar_slice) = s.as_slice_of::<u16>() {
                    let ms_string = WideString::from_vec(wchar_slice);
                    Ok(NameString::Microsoft(ms_string))
                } else {
                    manual_convert(s)
                }
            }
            // If the endians don't match we will always have to manually construct
            // the string
            #[cfg(target_endian = "little")]
            manual_convert(s)
        }

        fn new_other_from_raw(s: &[u8]) -> NameString {
            let buf = s.iter().cloned().collect();
            NameString::Other(buf)
        }
    }

    #[derive(Debug)]
    pub struct NameTable {
        format: u16, // Constant `0`
        count: u16,
        string_offset: u16,
        records: Vec<NameRecord>,
    }
    #[derive(Debug)]
    pub struct NameRecord {
        platform_id: u16,
        platform_specific_id: u16,
        language_id: u16,
        name_id: u16,
        length: u16,
        offset: u16,
        name: NameString,
    }
    pub mod tables_parse {
        use super::*;
        use nom::{be_u16, IResult, Offset};

        pub fn parse_name_table(i: &[u8]) -> IResult<&[u8], NameTable> {
            named!(partial_table<(u16, u16, u16)>,
               do_parse!(
                   format: verify!(be_u16, |val| val == 0) >>
                   count: be_u16 >>
                   string_offset: be_u16 >>
                   ((format, count, string_offset))
               )
            );

            let (i1, (format, count, string_offset)) = try_parse!(i, partial_table);

            let eaten = i.offset(i1);
                println!("eaten = {}", eaten);
            let new_offset = (string_offset as usize - eaten) as u16;
            let (i2, records) = try_parse!(i1, apply!(parse_name_records, count, new_offset));
            let nt = NameTable {
                format, count, string_offset, records
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
            };

            // FIXME: name isn't always unicode. Could be Microsoft encoding
            // (https://hexapdf.gettalong.org/api/HexaPDF/Font/TrueType/Table/Name/Record.html)

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
    }
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

// https://stackoverflow.com/questions/42199727/how-to-construct-const-integers-from-literal-byte-expressions
#[cfg(target_endian = "big")]
#[macro_export]
macro_rules! u32_code {
    ($w:expr) => {
        ((($w[3] as u32) <<  0) |
         (($w[2] as u32) <<  8) |
         (($w[1] as u32) << 16) |
         (($w[0] as u32) << 24) |
         ((*$w as [u8; 4])[0] as u32 * 0))
    }
}
#[cfg(target_endian = "little")]
#[macro_export]
macro_rules! u32_code {
    ($w:expr) => {
        ((($w[0] as u32) <<  0) |
         (($w[1] as u32) <<  8) |
         (($w[2] as u32) << 16) |
         (($w[3] as u32) << 24) |
         ((*$w as [u8; 4])[0] as u32 * 0))
    }
}

// Varius tags: http://scripts.sil.org/cms/scripts/page.php?site_id=nrsi&id=IWS-AppendixC
#[repr(u32)]
#[derive(Debug, FromPrimitive, PartialEq, Eq)]
enum TableTag {
    // Required
    Name = u32_code!(b"name"),
    GlyphOutline = u32_code!(b"glyf"),
    CharacterCodeMapping = u32_code!(b"cmap"),
    PostScriptGlyphName = u32_code!(b"post"),
    FontHeader = u32_code!(b"head"),
    HorizontalMetrics = u32_code!(b"hmtx"),
    HorizontalHeader = u32_code!(b"hhea"),
    HorizontalDeviceMetrics = u32_code!(b"hdmx"),
    GlyphLocation = u32_code!(b"loca"),
    MaximumProfile = u32_code!(b"maxp"),

    // Windows
    Compatibility = u32_code!(b"OS/2"),

    // Apple
    AccentAttachment = u32_code!(b"acnt"),
    AnchorPoint = u32_code!(b"ankr"),
    AxisVariation = u32_code!(b"avar"),
    BitmapData = u32_code!(b"bdat"),
    BitmapFontHeader = u32_code!(b"bhed"),
    BitmapLocation = u32_code!(b"bloc"),
    Baseline = u32_code!(b"bsln"),
    CVTVariation = u32_code!(b"cvar"),
    FontDescriptor = u32_code!(b"fdsc"),
    LayoutFeature = u32_code!(b"feat"),
    FontMetrics = u32_code!(b"fmtx"),
    FontFamilyCompat = u32_code!(b"fond"),
    FontVariation = u32_code!(b"fvar"),
    GlyphVariation = u32_code!(b"gvar"),
    Justification = u32_code!(b"just"),
    ExtendedKerning = u32_code!(b"kerx"),
    LigatureCaret = u32_code!(b"lcar"),
    LanguageTag = u32_code!(b"ltag"),
    Metadata = u32_code!(b"meta"),
    Metamorphosis = u32_code!(b"mort"),
    ExtendedMetamorphosis = u32_code!(b"morx"),
    OpticalBounds = u32_code!(b"opbd"),
    Properties = u32_code!(b"prop"),
    ExtendedBitmaps = u32_code!(b"sbix"),
    Tracking = u32_code!(b"trak"),
    CrossReference = u32_code!(b"xref"),
    GlyphReference = u32_code!(b"Zapf"),

    // OpenType
    GSUB = u32_code!(b"GSUB"),
    GPOS = u32_code!(b"GPOS"),
    GDEF = u32_code!(b"GDEF"),
    BASE = u32_code!(b"BASE"),
    JSFT = u32_code!(b"JSTF"),

    // Graphite
    Silf = u32_code!(b"Silf"),
    Glat = u32_code!(b"Glat"),
    Gloc = u32_code!(b"Gloc"),
    Feat = u32_code!(b"Feat"),

    // FontForge
    FFTimestamp = u32_code!(b"FFTM"), // FontForge timestamp table

    // Optional
    Kerning = u32_code!(b"kern"),
    LTHS = u32_code!(b"LTHS"),
    VerticalMetrics = u32_code!(b"vmtx"),
    VerticalHeader = u32_code!(b"vhea"),
    VDMX = u32_code!(b"VDMX"),
    DSIG = u32_code!(b"DSIG"),
    PCLT = u32_code!(b"PCLT"),
    GridFitAndScanConv = u32_code!(b"gasp"),
    ControlValueProgram = u32_code!(b"prep"),
    FontProgram = u32_code!(b"fpgm"),
    ControlValue = u32_code!(b"cvt "), // Tags less than 4 chars have trailing spaces
    CFF = u32_code!(b"CFF "),
    VORG = u32_code!(b"VORG"),
    EBDT = u32_code!(b"EBDT"),
    EBLC = u32_code!(b"EBLC"),
    EmbeddedBitmapScalingControl = u32_code!(b"EBSC"),
}

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
