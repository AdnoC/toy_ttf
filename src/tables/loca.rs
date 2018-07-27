use parse::{DynArr, Parse};
use tables::{PrimaryTable, TableTag};

/// Not `Parse`-able since it requires outside information
#[derive(Debug)]
pub enum Loca<'a> {
    Short(S<'a>),
    Long(L<'a>),
}

#[derive(Debug)]
pub struct S<'a>(pub(crate) DynArr<'a, u16>);
#[derive(Debug)]
pub struct L<'a>(pub(crate) DynArr<'a, u32>);

impl<'a> PrimaryTable for Loca<'a> {
    fn tag() -> TableTag {
        TableTag::GlyphLocation
    }
}

impl<'a> Loca<'a> {
    pub fn at(&self, idx: usize) -> u32 {
        use self::Loca::*;
        match self {
            Short(arr) => arr.0.at(idx) as u32,
            Long(arr) => arr.0.at(idx),
        }
    }
}
