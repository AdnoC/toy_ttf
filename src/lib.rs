#[cfg(test)]
extern crate byte_conv;

extern crate byte_slice_cast;

#[macro_use]
extern crate nom;

#[macro_use]
extern crate num_derive;
extern crate num_traits;

extern crate widestring;


// NOTE: TrueType is big endian (from https://wiki.osdev.org/TrueType_Fonts)

pub mod parse;
