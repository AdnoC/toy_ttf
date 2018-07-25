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
    subtables_start: &[u8],
}

struct CMapEncodingRecord {
    platform_id: u16,
    platform_specific_id: u16,
    offset: u32,
}

enum CMapMappings {
    Format0(Format0),
    Format2(Format2),
    Format4(Format4),
    Format6(Format6),
    Format8_0(Format8_0),
    Format10_0(Format10_0),
    Format12_0(Format12_0),
    Format13_0(Format13_0),
    Format14(Format14A,)
}

struct Format0 {
    mapping: [u8; 256],
}

impl Format0 {
    fn map(&self, char_code: u8) -> u8 {
        self.mapping[char_code] // = glyph_index
    }
}
