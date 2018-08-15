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

        HMTX {
            horiz_metrics,
            left_bearings,
        }
    }
}

#[derive(Debug, Parse)]
pub struct LongHorizMetric {
    advance_width: u16,
    left_bearing: i16,
}
