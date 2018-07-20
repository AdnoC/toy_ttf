#[cfg(test)]
extern crate byte_conv;

#[macro_use]
extern crate nom;
#[macro_use]
extern crate num_derive;
extern crate num_traits;

// NOTE: TrueType is big endian (from https://wiki.osdev.org/TrueType_Fonts)

pub mod parse;
