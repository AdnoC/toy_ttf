// Apple TrueType Spec
// https://developer.apple.com/fonts/TrueType-Reference-Manual/

// Microsoft OpenType Spec
// https://docs.microsoft.com/en-us/typography/opentype/spec/otff

// TODO: New plan
//
// Don't copy anything with variable length
//
//      All variable-length things should be accessed thgough iterators
//
// Don't eagerly decode strings
//
//      Wait for the user to request it
//
// All table parsers have their own failure codes
//
//      Should be in the form of a #[derive(FromPrimitive, ToPrimitive)] enum
//
// Single method to get a table of some type
//
//      Should return Result<SpecificType, GetTableErr>
//
//      GetTableErr should be enum of all specific table's error enums
//
//          Should also include a `TableNotFound` case
//
// Still use nom for parsing
//
//      Error conditions should be mapped to table-specific errors
//
// All tables should be bound to lifetime of file

extern crate encoding;

#[cfg(test)]
extern crate byte_conv;

extern crate byte_slice_cast;

#[macro_use]
extern crate nom;

#[macro_use]
extern crate num_derive;
extern crate num_traits;

extern crate widestring;

extern crate byteorder;

extern crate itertools;

extern crate image;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate parse_derive;

// NOTE: TrueType is big endian (from https://wiki.osdev.org/TrueType_Fonts)

#[macro_use]
mod parse;
pub mod font;
pub mod tables;
pub mod render;

#[cfg(test)]
pub(crate) mod test_utils {
    pub const SANS_MONO: &'static str = "fonts/DejaVuSansMono.ttf";
    pub const ROBOTO: &'static str = "fonts/Roboto-Regular.ttf";

    pub fn font_buf() -> Vec<u8> {
        load_font_buf(SANS_MONO)
    }
    pub fn load_font_buf(name: &str) -> Vec<u8> {
        use std::fs::File;
        use std::io::prelude::*;
        use std::io::BufReader;

        let file = File::open(name).expect("unable to open file");

        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data).expect("error reading file");

        data
    }
}
