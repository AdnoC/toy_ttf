type ShortFrac = u16;
type Fixed = (u16, u16);
type FWord = u16;
type uFWord = u16;
type F2Dot14 = u16;
type LongDateTime = i64;

#[derive(Debug)]
struct CMap<'a> {
    version: u16,
    num_subtables: u16,
    subtables: &'a [u8],
}
impl ::table::PrimaryTable<'a> for Cmap<'a> {
    fn tag() -> ::table::TableTag {
        ::table::TableTag::CharacterCodeMapping
    }
    fn parse(table_buf: &'a [u8]) -> Result<Self, ::table::ParseTableError> {

    }
}

pub trait PrimaryTable<'file>: Sized {
    fn tag() -> TableTag;
    fn parse(table_start: &'file [u8]) -> Result<Self, ParseTableError>;
}

struct CMapEncodingRecord {
    platform_id: u16,
    platform_specific_id: u16,
    offset: u32,
}

enum CMapMappings<'a> {
    // Format0(Format0),
    // Format2(Format2),
    Format4(Format4<'a>),
    // Format6(Format6),
    // Format8_0(Format8_0),
    // Format10_0(Format10_0),
    // Format12_0(Format12_0),
    // Format13_0(Format13_0),
    // Format14(Format14A,)
}

struct Format4<'a> {
    // format: u16, = 4
    length: u16,
    language: u16,
    seg_count_x2: u16,
    search_range: u16,
    entry_selector: u16,
    range_shift: u16,
    end_counts: &'a [u8],
    // reverse_pad: u16, = 0
    start_counts: &'a [u8],
    id_deltas: &'a [u8],
    id_range_offsets: &'a [u8],
    glyph_ids: &'a [u8],
}
