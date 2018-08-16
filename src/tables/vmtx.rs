use parse::{DynArr, Parse};
use parse::primitives::FontUnit;
use tables::{PrimaryTable, TableTag};

#[derive(Debug)]
pub struct VMTX<'a> {
    vert_metrics: DynArr<'a, LongVertMetric>,
    top_bearings: DynArr<'a, FontUnit<i16>>,
}

impl<'a> PrimaryTable for VMTX<'a> {
    fn tag() -> TableTag {
        TableTag::VerticalMetrics
    }
}

impl<'a> VMTX<'a> {
    pub fn parse_metrics(buf: &'a [u8], num_vert_metrics: u16) -> VMTX<'a> {
        use std::marker::PhantomData;

        let (vert_metric_buf, top_bearings_buf) =
            DynArr::<LongVertMetric>::split_buf_for_len(buf, num_vert_metrics as usize);
        let vert_metrics = DynArr(vert_metric_buf, PhantomData);
        let top_bearings = DynArr(top_bearings_buf, PhantomData);
        // assert_eq: top_bearings.len(), num_glyphs - num_vert_metrics

        VMTX {
            vert_metrics,
            top_bearings,
        }
    }

    pub fn metrics_for_glyph(&self, glyph_id: u32) -> VertMetric {
        let glyph_id = glyph_id as usize;
        if glyph_id < self.vert_metrics.len() {
            self.vert_metrics.at(glyph_id).into()
        } else {
            let idx = glyph_id - self.vert_metrics.len();
            self.top_bearings.at(idx).into()
        }
    }
}

#[derive(Debug, Parse)]
pub struct LongVertMetric {
    advance_height: FontUnit<u16>,
    top_bearing: FontUnit<i16>,
}

#[derive(Debug)]
pub struct VertMetric {
    pub advance_height: Option<FontUnit<u16>>,
    pub top_bearing: FontUnit<i16>,
}

impl From<FontUnit<i16>> for VertMetric {
    fn from(top_bearing: FontUnit<i16>) -> Self {
        VertMetric {
            advance_height: None,
            top_bearing,
        }
    }
}
impl From<LongVertMetric> for VertMetric {
    fn from(LongVertMetric {  top_bearing, advance_height }: LongVertMetric) -> Self {
        VertMetric {
            advance_height: Some(advance_height),
            top_bearing,
        }
    }
}

