use parse::{Parse};
use parse::primitives::{Fixed, FWord, UFWord, PhantomLifetime};
use tables::{PrimaryTable, TableTag};

#[derive(Debug, Parse)]
pub struct VHEA<'a> {
    _lifetime_use: PhantomLifetime<'a>,
    /// Version number of the Vertical Header Table (0x00011000 for the current version).
    version: Fixed,
    /// The vertical typographic ascender for this font. It is the distance in
    /// FUnits from the vertical center baseline to the right of the design space.
    /// This will usually be set to half the horizontal advance of full-width glyphs.
    /// For example, if the full width is 1000 FUnits, this field will be set to 500.
    vert_typo_ascender: i16,
    /// The vertical typographic descender for this font. It is the distance in
    /// FUnits from the vertical center baseline to the left of the design space.
    /// This will usually be set to half the horizontal advance of full-width glyphs.
    /// For example, if the full width is 1000 FUnits, this field will be set to -500.
    vert_typo_descender: i16,
    /// The vertical typographic line gap for this font.
    vert_typo_line_gap: i16,
    /// The maximum advance height measurement in FUnits found in the font.
    /// This value must be consistent with the entries in the vertical metrics table.
    advance_height_max: i16,
    /// The minimum top side bearing measurement in FUnits found in the font, in FUnits.
    /// This value must be consistent with the entries in the vertical metrics table.
    min_top_side_bearing: i16,
    /// The minimum bottom side bearing measurement in FUnits found in the font, in FUnits.
    /// This value must be consistent with the entries in the vertical metrics table.
    min_bottom_side_bearing: i16,
    /// This is defined as the value of the minTopSideBearing field added to the result
    /// of the value of the yMin field subtracted from the value of the yMax field.
    y_max_extent: i16,
    /// The value of the caretSlopeRise field divided by the value of the caretSlopeRun field
    /// determines the slope of the caret.
    /// A value of 0 for the rise and a value of 1 for the run specifies a horizontal caret.
    /// A value of 1 for the rise and a value of 0 for the run specifies a vertical caret.
    /// A value between 0 for the rise and 1 for the run is desirable for fonts whose
    /// glyphs are oblique or italic. For a vertical font, a horizontal caret is best.
    caret_slope_rise: i16,
    /// See the caretSlopeRise field. Value = 0 for non-slanted fonts.
    caret_slope_run: i16,
    /// The amount by which the highlight on a slanted glyph needs to be shifted
    /// away from the glyph in order to produce the best appearance.
    /// Set value equal to 0 for non-slanted fonts.
    caret_offset: i16,
    /// Set to 0.
    reserved1: i16,
    /// Set to 0.
    reserved2: i16,
    /// Set to 0.
    reserved3: i16,
    /// Set to 0.
    reserved4: i16,
    /// Set to 0.
    metric_data_format: i16,
    /// Number of advance heights in the Vertical Metrics table.
    pub num_vert_metrics: u16,
}
impl<'a> PrimaryTable for VHEA<'a> {
    fn tag() -> TableTag {
        TableTag::VerticalHeader
    }
}

