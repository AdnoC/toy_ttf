use parse::primitives::{Fixed, LongDateTime};
use parse::{DynArr, Parse};
use tables::{PrimaryTable, TableTag};

#[derive(Debug, Parse, PartialEq)]
pub struct Head {
    major_version: u16,
    minor_version: u16,
    font_revision: Fixed,
    check_sum_adjustment: u32,
    magic_number: u32,
    flags: u16,
    units_per_em: u16,
    created: LongDateTime,
    modified: LongDateTime,
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
    mac_style: u16,
    lowest_recPPEM: u16,
    font_direction_hint: i16,
    pub(crate) index_to_loc_format: IndexToLocFormat,
    glyph_data_format: i16,
}

impl PrimaryTable for Head {
    fn tag() -> TableTag {
        TableTag::FontHeader
    }
}

#[repr(i16)]
#[derive(Debug, FromPrimitive, PartialEq, Eq)]
pub enum IndexToLocFormat {
    Short = 0,
    Long = 1,
}
derive_parse_from_primitive!(IndexToLocFormat, i16);

#[cfg(test)]
mod tests {
    use super::*;
    use font::*;
    use test_utils::font_buf;

    #[test]
    fn head() {
        use parse::primitives::Fixed;

        let buf = font_buf();
        let font = Font::from_buffer(&buf).unwrap();
        let head: Head = font.get_table().unwrap();
        let expected = Head {
            major_version: 1,
            minor_version: 0,
            font_revision: Fixed(0x2, 0x5eb8),
            check_sum_adjustment: 400577649,
            magic_number: 1594834165,
            flags: 31,
            units_per_em: 2048,
            created: 3552717816,
            modified: 3552717816,
            x_min: -1142,
            y_min: -767,
            x_max: 1470,
            y_max: 2105,
            mac_style: 0,
            lowest_recPPEM: 8,
            font_direction_hint: 0,
            index_to_loc_format: IndexToLocFormat::Long,
            glyph_data_format: 0,
        };

        assert_eq!(head, expected);
    }
}
