use parse::{Parse, BufView, split_buf_for_len};
use parse::primitives::{Em, FontUnit, TWIP};
use tables::{PrimaryTable, TableTag};

// Gomments taken from apple docs
#[derive(Debug)]
pub struct OS2<'a> {
    /// table version number
    version: u16,
    pub base_table: Version0Ext,
    pub v4_table: Version4Ext,
    v5_table: BufView<'a, u8>,
}

impl<'a> PrimaryTable for OS2<'a> {
    fn tag() -> TableTag {
        TableTag::CompatibilityMetrics 
    }
}

impl<'a> Parse<'a> for OS2<'a> {
    fn approx_file_size() -> usize {
        // Shouldn't need the size of a primary table
        unimplemented!()
    }
    fn parse(buf: &'a [u8]) -> (&'a [u8], Self) {
        use std::marker::PhantomData;

        let (buf, version) = u16::parse(buf);
        assert!(version >= 4);

        let (buf, base_table) = Version0Ext::parse(buf);
        let (buf, v4_table) = Version4Ext::parse(buf);

        let (v5_buf, rest) = if version >= 5 {
            split_buf_for_len::<Version5Ext>(buf, 1)
        } else {
            (buf, buf)
        };

        let v5_table = BufView(v5_buf, PhantomData);

        let os2 = OS2 {
            version,
            base_table,
            v4_table,
            v5_table,
        };

        (rest, os2)
    }
}

#[derive(Debug, Parse)]
pub struct Version0Ext {
    /// average weighted advance width of lower case letters and space
    x_avg_char_width: Em<i16>,
    /// visual weight (degree of blackness or thickness) of stroke in glyphs
    us_weight_class: u16,
    /// relative change from the normal aspect ratio (width to height ratio) as
    /// specified by a font designer for the glyphs in the font
    us_width_class: u16,
    /// characteristics and properties of this font (set undefined bits to zero)
    fs_type: i16,
    ///  recommended horizontal size in pixels for subscripts
    y_subscript_x_size: FontUnit<i16>,
    /// recommended vertical size in pixels for subscripts
    y_subscript_y_size: FontUnit<i16>,
    /// recommended horizontal offset for subscripts
    y_subscript_x_offset: FontUnit<i16>,
    /// recommended vertical offset form the baseline for subscripts
    y_subscript_y_offset: FontUnit<i16>,
    /// recommended horizontal size in pixels for superscripts
    y_superscript_x_size: FontUnit<i16>,
    /// recommended vertical size in pixels for superscripts
    y_superscript_y_size: FontUnit<i16>,
    /// recommended horizontal offset for superscripts
    y_superscript_x_offset: FontUnit<i16>,
    /// recommended vertical offset from the baseline for superscripts
    y_superscript_y_offset: FontUnit<i16>,
    /// width of the strikeout stroke
    y_strikeout_size: FontUnit<i16>,
    /// position of the strikeout stroke relative to the baseline
    y_strikeout_position: FontUnit<i16>,
    /// classification of font-family design.
    s_family_class: i16,
    /// 10 byte series of number used to describe the visual characteristics of a given typeface
    panose: [u8; 10],
    ///  Field is split into two bit fields of 96 and 36 bits each.
    ///  The low 96 bits are used to specify the Unicode blocks encompassed by
    ///  the font file.
    ///  The high 32 bits are used to specify the character or script sets covered
    ///  by the font file.
    ///  Bit assignments are pending.
    ///  Set to 0
    ul_char_range: [u32; 4],
    /// 4]   four character identifier for the font vendor
    ach_vend_id: [u8; 4],
    /// 2-byte bit field containing information concerning the nature of the font patterns
    fs_selection: u16,
    /// The minimum Unicode index in this font.
    fs_first_char_index: u16,
    /// The maximum Unicode index in this font.
    fs_last_char_index: u16,
}

#[derive(Debug, Parse)]
pub struct Version4Ext {
    /// The typographic ascender for this font.
    /// This is not necessarily the same as the ascender value in the 'hhea' table.
    pub s_typo_ascender: FontUnit<i16>,
    /// The typographic descender for this font.
    /// This is not necessarily the same as the descender value in the 'hhea' table.
    pub s_typo_descender: FontUnit<i16>,
    /// The typographic line gap for this font.
    /// This is not necessarily the same as the line gap value in the 'hhea' table.
    pub s_typo_line_gap: FontUnit<i16>,
    /// The ascender metric for Windows. usWinAscent is computed as the yMax
    /// for all characters in the Windows ANSI character set.
    us_win_ascent: FontUnit<u16>,
    /// The descender metric for Windows. usWinDescent is computed as the -yMin
    /// for all characters in the Windows ANSI character set.
    us_win_descent: FontUnit<u16>,
    /// Bits 0-31
    ul_code_page_range1: u32,
    /// Bits 32-63
    ul_code_page_range2: u32,
    /// The distance between the baseline and the approximate height of
    /// non-ascending lowercase letters measured in FUnits.
    sx_height: FontUnit<i16>,
    /// The distance between the baseline and the approximate height of
    /// uppercase letters measured in FUnits.
    s_cap_height: FontUnit<i16>,
    /// The default character displayed by Windows to represent an unsupported character.
    /// (Typically this should be 0.)
    us_default_char: u16,
    /// The break character used by Windows.
    us_break_char: u16,
    /// The maximum length of a target glyph OpenType context for any feature in this font.
    us_max_context: u16,
}

#[derive(Debug, Parse)]
pub struct Version5Ext {
    /// Proposed for version 5.
    /// The lowest size (in twentieths of a typographic point),
    /// at which the font starts to be used.
    /// This is an inclusive value.
    us_lower_point_size: TWIP<u16>,
    /// Proposed for version 5.
    /// The highest size (in twentieths of a typographic point),
    /// at which the font starts to be used.
    /// This is an exclusive value.
    /// Use 0xFFFFU to indicate no upper limit.
    us_upper_point_size: TWIP<u16>,
}
