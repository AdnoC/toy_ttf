use nom::{be_u16, be_u32};
use num_traits::FromPrimitive;

// TODO: Use to verify font tables
#[allow(dead_code)]
fn table_check_sum(table: &[u32]) -> u32 {
    table.iter().sum()
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
    let font_buf = &font_buf[0..1024];
    // let fd = parse_font_directory(font_buf).unwrap();
    // println!("fd = {:#?}", fd);
    // let off = parse_offset_subtable(font_buf);
    // let st = off.and_then(|inp| table_directory_entry(inp.0));
    let st = parse_font_directory(font_buf);
    match st {
        Ok(val) => println!("ok {:#?}", val.1),
        Err(e) => println!("err {:?}", e),
    };
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
#[derive(Debug, FromPrimitive)]
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
