use parse::Parse;

#[derive(Debug, Parse)]
struct CMap<'a> {
    version: u16,
    num_subtables: u16,
    #[arr_len_src = "num_subtables"]
    subtables: &'a [u8],
}
impl<'a> ::tables::PrimaryTable for CMap<'a> {
    fn tag() -> ::tables::TableTag {
        ::tables::TableTag::CharacterCodeMapping
    }
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
