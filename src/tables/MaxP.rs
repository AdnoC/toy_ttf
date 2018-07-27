use parse::{Parse, BufView};
use parse::primitives::Fixed;
use tables::{PrimaryTable, TableTag};

#[derive(Debug, Parse)]
pub struct MaxP<'a> {
    version: Fixed,
    num_glyphs: u16,
    ext_start: BufView<'a, MaxPV1Ext>,
}
impl<'a> PrimaryTable for MaxP<'a> {
    fn tag() -> TableTag {
        TableTag::MaximumProfile
    }
}

impl<'a> MaxP<'a> {
    pub fn is_version_0_5(&self) -> bool {
        self.version.0 == 0 && self.version.1 == 0x5000
    }

    pub fn is_version_1(&self) -> bool {
        self.version.0 == 1 && self.version.1 == 0
    }

    pub fn version_1_ext(&self) -> Option<MaxPV1Ext> {
        if self.is_version_1() {
            Some(self.ext_start.at(0))
        } else { None }
    }
}

#[derive(Debug, Parse, PartialEq)]
pub struct MaxPV1Ext {
    max_points: u16,
    max_contours: u16,
    max_composite_points: u16,
    max_composite_contours: u16,
    max_zones: u16,
    max_twilight_points: u16,
    max_storage: u16,
    max_function_defs: u16,
    max_instruction_defs: u16,
    max_stack_elements: u16,
    max_size_Of_instructions: u16,
    max_component_elements: u16,
    max_component_depth: u16,
}

#[cfg(test)]
mod test {
    use font::Font;
    use super::{MaxP, MaxPV1Ext};
    use test_utils::font_buf;

    #[test]
    fn maxp() {
        use parse::primitives::Fixed;

        let buf = font_buf();
        let font = Font::from_buffer(&buf).unwrap();
        let maxp: MaxP = font.get_table().unwrap();

        assert_eq!(maxp.version, Fixed(1, 0));
        assert_eq!(maxp.num_glyphs, 3377);

        let expect_v1_ext = MaxPV1Ext {
            max_points: 524,
            max_contours: 43,
            max_composite_points: 107,
            max_composite_contours: 6,
            max_zones: 2,
            max_twilight_points: 16,
            max_storage: 153,
            max_function_defs: 8,
            max_instruction_defs: 0,
            max_stack_elements: 1367,
            max_size_Of_instructions: 273,
            max_component_elements: 5,
            max_component_depth: 4
        };

        assert_eq!(maxp.version_1_ext(), Some(expect_v1_ext));
    }

}
