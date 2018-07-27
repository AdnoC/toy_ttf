use parse::{BufView, DynArr, Parse};
use tables::RecordIterator;

#[derive(Debug, Parse)]
pub struct CMap<'a> {
    table: BufView<'a>,
    version: u16,
    num_records: u16,
    #[arr_len_src = "num_records"]
    records: DynArr<'a, CMapEncodingRecord>,
}
impl<'a> ::tables::PrimaryTable for CMap<'a> {
    fn tag() -> ::tables::TableTag {
        ::tables::TableTag::CharacterCodeMapping
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
    // fn formats(&self) -> 
    pub fn format4(&self) -> Option<Format4<'a>> {
        self.encoding_records()
            .map(|record| record.offset as usize)
            .map(|offset| &self.table.0[offset..])
            .find(|fmt_table| fmt_table_has_format(fmt_table, 4))
            .map(|fmt_table| Format4::parse(fmt_table).1)
    }
}

#[derive(Debug, Parse)]
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
    // Format8_0(Format8_0),
    // Format10_0(Format10_0),
    // Format12_0(Format12_0),
    // Format13_0(Format13_0),
    // Format14(Format14A,)
}

fn halve_u16(val: u16) -> u16 { val / 2 }

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
    glyph_ids: BufView<'a>, // u16
}

impl<'a> Format4<'a> {
    pub fn lookup_glyph_id(&self, code_point: u16) -> Option<u16> {
        let end_val = self.end_counts.iter()
            .enumerate()
            .rev()
            .find(|(i, val)| *val < code_point);
        unimplemented!()
        //
        //
        //
        // let end_counts_position = Self::get_end_counts_position();
        // let seg_count = self.get_seg_count();
        // let mut size = seg_count - 1;
        // /// middle of array
        // let mut index = size / 2;
        // while size > 0 {
        //     /// End of array
        //     let search = end_counts_position + index * 2;
        //     /// Get last value
        //     let end_value = get_u16(self.0, search as usize).unwrap();
        //     if end_value >= code_point {
        //         let start_pos = Self::get_start_counts_position(seg_count) + 2 * index;
        //         let start_value = get_u16(self.0, start_pos as usize).unwrap();
        //         if start_value > code_point {
        //             size /= 2;
        //             index -= size;
        //         } else {
        //             return self.extract_glyph_id(code_point, start_value, seg_count, index);
        //         }
        //     } else {
        //         size /= 2;
        //         index += size;
        //     }
        // }
        // None
    }
}
