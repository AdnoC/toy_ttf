use parse::{DynArr, Parse};
use parse::primitives::FWord;
use tables::{PrimaryTable, TableTag};

#[derive(Debug)]
pub struct HMTX<'a> {
    horiz_metrics: DynArr<'a, LongHorizMetric>,
    left_bearings: DynArr<'a, FWord>,
}

impl<'a> PrimaryTable for HMTX<'a> {
    fn tag() -> TableTag {
        TableTag::HorizontalMetrics
    }
}

impl<'a> HMTX<'a> {
    pub fn parse_metrics(buf: &'a [u8], num_horiz_metrics: u16) -> HMTX<'a> {
        use std::marker::PhantomData;

        let (horiz_metric_buf, left_bearings_buf) =
            DynArr::<LongHorizMetric>::split_buf_for_len(buf, num_horiz_metrics as usize);
        let horiz_metrics = DynArr(horiz_metric_buf, PhantomData);
        let left_bearings = DynArr(left_bearings_buf, PhantomData);
        // assert_eq: left_bearings.len(), num_glyphs - num_horiz_metrics

        HMTX {
            horiz_metrics,
            left_bearings,
        }
    }

    pub fn metrics_for_glyph(&self, glyph_id: u32) -> HorizMetric {
        let glyph_id = glyph_id as usize;
        if glyph_id < self.horiz_metrics.len() {
            self.horiz_metrics.at(glyph_id).into()
        } else {
            let idx = glyph_id - self.horiz_metrics.len();
            self.left_bearings.at(idx).into()
        }
    }
}

#[derive(Debug, Parse)]
pub struct LongHorizMetric {
    advance_width: u16,
    left_bearing: i16,
}

#[derive(Debug)]
pub struct HorizMetric {
    advance_width: Option<u16>,
    left_bearing: i16,
}

impl From<i16> for HorizMetric {
    fn from(left_bearing: i16) -> Self {
        HorizMetric {
            advance_width: None,
            left_bearing,
        }
    }
}
impl From<LongHorizMetric> for HorizMetric {
    fn from(LongHorizMetric {  left_bearing, advance_width }: LongHorizMetric) -> Self {
        HorizMetric {
            advance_width: Some(advance_width),
            left_bearing,
        }
    }
}
