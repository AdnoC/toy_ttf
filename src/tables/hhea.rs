use parse::{Parse};
use parse::primitives::{Fixed, FWord, UFWord, PhantomLifetime};
use tables::{PrimaryTable, TableTag};

// Comments taken from Apple hhea table docs
#[derive(Debug, Parse)]
pub struct HHEA<'a> {
    _lifetime_use: PhantomLifetime<'a>,
    /// 0x00010000 (1.0)
    version: Fixed,
    /// Distance from baseline of highest ascender
    ascent: FWord,
    /// Distance from baseline of lowest descender
    descent: FWord,
    /// typographic line gap
    line_gap: FWord,
    /// must be consistent with horizontal metrics
    advance_width_max: UFWord,
    /// must be consistent with horizontal metrics
    min_left_side_bearing: FWord,
    /// must be consistent with horizontal metrics
    min_right_side_bearing: FWord,
    /// max(lsb + (xMax-xMin))
    x_max_extent: FWord,
    /// used to calculate the slope of the caret (rise/run) set to 1 for vertical caret
    caret_slope_rise: i16,
    /// 0 for vertical
    caret_slope_run: i16,
    /// set value to 0 for non-slanted fonts
    caret_offset: FWord,
    /// set value to 0
    reserved1: i16,
    /// set value to 0
    reserved2: i16,
    /// set value to 0
    reserved3: i16,
    /// set value to 0
    reserved4: i16,
    /// 0 for current format
    metric_data_format: i16,
    /// number of advance widths in metrics table
    pub num_horiz_metrics: u16,
}
impl<'a> PrimaryTable for HHEA<'a> {
    fn tag() -> TableTag {
        TableTag::HorizontalHeader
    }
}
