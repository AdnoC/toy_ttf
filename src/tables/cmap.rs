use parse::{BufView, DynArr, Parse};
use tables::RecordIterator;
use tables::{PrimaryTable, TableTag};
use std::cmp::{PartialEq, PartialOrd, Ordering};

#[derive(Debug, Parse)]
pub struct CMap<'a> {
    table: BufView<'a, u8>,
    version: u16,
    num_records: u16,
    #[arr_len_src = "num_records"]
    records: DynArr<'a, CMapEncodingRecord>,
}
impl<'a> PrimaryTable for CMap<'a> {
    fn tag() -> TableTag {
        TableTag::CharacterCodeMapping
    }
}

impl<'a> CMap<'a> {
    pub fn encoding_records(&self) -> RecordIterator<'a, CMapEncodingRecord> {
        use std::marker::PhantomData;
        RecordIterator {
            next_record: self.records.0,
            num_left: self.num_records,
            _marker: PhantomData,
        }
    }
    pub fn get_format<T: CMapFormatTable<'a>>(&self) -> Option<T> {
        self.encoding_records()
            .map(|record| record.offset as usize)
            .map(|offset| &self.table.0[offset..])
            .find(|fmt_table| fmt_table_has_format(fmt_table, T::format_identifier()))
            .map(|fmt_table| T::parse(fmt_table).1)
    }
    // fn formats(&self) ->
    pub fn format4(&self) -> Option<Format4<'a>> {
        self.encoding_records()
            .map(|record| record.offset as usize)
            .map(|offset| &self.table.0[offset..])
            .find(|fmt_table| fmt_table_has_format(fmt_table, 4))
            .map(|fmt_table| Format4::parse(fmt_table).1)
    }
}

pub trait CMapFormatTable<'a>: Parse<'a> {
    fn format_identifier() -> u16;
}

#[derive(Debug, Parse, PartialEq)]
pub struct CMapEncodingRecord {
    platform_id: u16,
    platform_specific_id: u16,
    offset: u32,
}

fn fmt_table_has_format<'a>(fmt_table: &'a [u8], format: u16) -> bool {
    let table_format = u16::parse(fmt_table).1;
    table_format == format
}

enum CMapMappings<'a> {
    // Format0(Format0),
    // Format2(Format2),
    Format4(Format4<'a>),
    // Format6(Format6),
    // Format8(Format8),
    // Format10(Format10),
    Format12(Format12<'a>),
    // Format13(Format13),
    // Format14(Format14A,)
}

fn halve_u16(val: u16) -> u16 {
    val / 2
}

/// Standard on Windows for Unicode BMP characters
#[derive(Debug, Parse)]
pub struct Format4<'a> {
    format: u16, // = 4
    #[len_src]
    length: u16,
    language: u16,
    #[parse_mod = "halve_u16"]
    seg_count: u16,
    search_range: u16,
    entry_selector: u16,
    range_shift: u16,
    #[arr_len_src = "seg_count"]
    end_counts: DynArr<'a, u16>,
    reserved_padding: u16, // = 0
    #[arr_len_src = "seg_count"]
    start_counts: DynArr<'a, u16>,
    #[arr_len_src = "seg_count"]
    id_deltas: DynArr<'a, i16>,
    #[arr_len_src = "seg_count"]
    id_range_offsets: DynArr<'a, u16>,
    glyph_ids: BufView<'a, u16>,
}

impl<'a> Format4<'a> {
    fn get_glyph_id(&self, code_point: u16, start_idx: usize, start_code: u16) -> Option<u16> {
        let id_delta = self.id_deltas.at(start_idx);
        let id_range_offset = self.id_range_offsets.at(start_idx);
        if id_range_offset != 0 {
            // We don't use pointer tricks, so we have to shift the index into
            // the glyph_id array down based on the distance to the start of
            // the array
            let glyph_shift = self.seg_count - start_idx as u16;
            let glyph_idx = id_range_offset / 2 + (code_point - start_code) - glyph_shift;

            let glyph_val = self.glyph_ids.at(glyph_idx as usize);
            if glyph_val != 0 {
                // id_delta arithmetic is modulo 2^16
                // TODO: Check whether commenting this out is correct
                // let glyph_val = glyph_val.wrapping_add(id_delta as u16);
                Some(glyph_val)
            } else {
                None
            }
        } else {
            // id_delta arithmetic is modulo 2^16
            let glyph_val = code_point.wrapping_add(id_delta as u16);
            Some(glyph_val)
        }
    }
    pub fn lookup_glyph_id(&self, code_point: u16) -> Option<u16> {
        use byteorder::{ByteOrder, BE};
        for (idx, end_code) in self.end_counts.iter().enumerate() {
            if end_code >= code_point {
                let start_code = self.start_counts.at(idx);
                if start_code <= code_point {
                    return self.get_glyph_id(code_point, idx, start_code);
                } else {
                    return None;
                }
            }
        }
        None
    }
}

impl<'a> CMapFormatTable<'a> for Format4<'a> {
    fn format_identifier() -> u16 { 4 }
}

/// Standard on Windows for Unicode supplementary-plane characters
#[derive(Debug, Parse)]
pub struct Format12<'a> {
    format: u16, // = 12
    reserved: u16, // = 0
    length: u32,
    language: u32,
    #[len_src]
    num_groups: u32,
    // Sorted by increasing `start_char_code`
    #[arr_len_src = "num_groups"]
    groups: DynArr<'a, SequentialMapGroup>,
}
impl<'a> CMapFormatTable<'a> for Format12<'a> {
    fn format_identifier() -> u16 { 12 }
}

impl<'a> Format12<'a> {
    pub fn lookup_glyph_id(&self, code_point: u32) -> Option<u32> {
        let group = self.groups
            .binary_search_by(|group| group.partial_cmp(&code_point).unwrap())?;
        group.lookup_glyph_id(code_point)
    }
}

#[derive(Debug, Parse)]
struct SequentialMapGroup {
    /// Inclusive
    start_char_code: u32,
    /// Inclusive
    end_char_code: u32,
    /// "Glyph index corresponding to the starting character code;
    /// subsequent charcters are mapped to sequential glyphs"
    /// (Apple CMap table docs)
    start_glyph_id: u32,
}

impl SequentialMapGroup {
    pub fn lookup_glyph_id(&self, code_point: u32) -> Option<u32> {
        if code_point < self.start_char_code || self.end_char_code < code_point {
            return None;
        }

        let delta_cp = code_point - self.start_char_code;
        let glyph_id = self.start_glyph_id + delta_cp;
        Some(glyph_id)
    }
}

impl PartialEq<u32> for SequentialMapGroup {
    fn eq(&self, code_point: &u32) -> bool {
        let code_point = *code_point;
        self.start_char_code <= code_point && code_point <= self.end_char_code
    }
}
impl PartialOrd<u32> for SequentialMapGroup {
    fn partial_cmp(&self, code_point: &u32) -> Option<Ordering> {
        use std::cmp::Ordering::*;
        let code_point = *code_point;
        if code_point < self.start_char_code {
            Some(Greater)
        } else if self.end_char_code < code_point {
            Some(Less)
        } else {
            Some(Equal)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use font::*;
    use test_utils::font_buf;

    #[test]
    fn cmap_primary() {
        let buf = font_buf();
        let font = Font::from_buffer(&buf).unwrap();
        let cmap: CMap = font.get_table().unwrap();

        assert_eq!(cmap.version, 0);
        assert_eq!(cmap.num_records, 5);
    }

    #[test]
    fn cmap_records() {
        let buf = font_buf();
        let font = Font::from_buffer(&buf).unwrap();
        let cmap: CMap = font.get_table().unwrap();

        let expecteds = &[
            CMapEncodingRecord {
                platform_id: 0,
                platform_specific_id: 3,
                offset: 44,
            },
            CMapEncodingRecord {
                platform_id: 0,
                platform_specific_id: 10,
                offset: 2100,
            },
            CMapEncodingRecord {
                platform_id: 1,
                platform_specific_id: 0,
                offset: 5188,
            },
            CMapEncodingRecord {
                platform_id: 3,
                platform_specific_id: 1,
                offset: 44,
            },
            CMapEncodingRecord {
                platform_id: 3,
                platform_specific_id: 10,
                offset: 2100,
            },
        ];

        for (ref actual, expected) in cmap.encoding_records().zip(expecteds) {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn cmap_format4() {
        let buf = font_buf();
        let font = Font::from_buffer(&buf).unwrap();
        let cmap: CMap = font.get_table().unwrap();
        let f4 = cmap.format4().unwrap();

        assert_eq!(f4.format, 4);
        assert_eq!(f4.length, 2056);
        assert_eq!(f4.language, 0);
        assert_eq!(f4.seg_count, 255);
        assert_eq!(f4.search_range, 256);
        assert_eq!(f4.entry_selector, 7);
        assert_eq!(f4.range_shift, 254);
        assert_eq!(f4.end_counts.0.len(), 510);
        assert_eq!(f4.reserved_padding, 0);
        assert_eq!(f4.start_counts.0.len(), 510);
        assert_eq!(f4.id_deltas.0.len(), 510);
        assert_eq!(f4.id_range_offsets.0.len(), 510);
        assert_eq!(f4.glyph_ids.0.len(), 321944);
    }

    #[test]
    fn glyph_id_simple() {
        let buf = font_buf();
        let font = Font::from_buffer(&buf).unwrap();
        let cmap: CMap = font.get_table().unwrap();
        let f4 = cmap.format4().unwrap();

        let expecteds: &[u16] = &[36, 68, 70];
        let code_points: &[u16] = &[65, 97, 99];
        for (&code_point, &expected) in code_points.into_iter().zip(expecteds) {
            let glyph_id = f4.lookup_glyph_id(code_point).unwrap();
            assert_eq!(glyph_id, expected);
        }
    }

    #[test]
    fn glyph_id_fancy() {
        use test_utils::{load_font_buf, ROBOTO};
        let buf = load_font_buf(ROBOTO);
        let font = Font::from_buffer(&buf).unwrap();
        let cmap: CMap = font.get_table().unwrap();
        let f4 = cmap.format4().unwrap();

        let glyph_id = f4.lookup_glyph_id(192).unwrap();
        assert_eq!(glyph_id, 639);
    }
}
