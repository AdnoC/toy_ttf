use tables::TableTag;
use widestring::WideString;
use std::str::Utf8Error;
use byte_slice_cast::{AsSliceOf, Error};

// FIXME: Should be put somewhere more sensible
// TODO: Should hold reference to the string
pub enum NameString {
    Unicode(String),
    Microsoft(WideString),
    Other(Vec<u8>)
}
impl ::std::fmt::Debug for NameString {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use self::NameString::*;
        match self {
            Unicode(name) => f.debug_tuple("Unicode").field(name).finish(),
            Microsoft(name) => f.debug_tuple("Microsoft").field(&name.to_string_lossy()).finish(),
            Other(raw_name) => f.debug_tuple("Other").field(raw_name).finish()
        }
    }
}

impl NameString {
    pub(crate) fn new_unicode_from_raw(s: &[u8]) -> Result<NameString, Utf8Error> {
        // FIXME: UNICODE IS IN UTF-16 ENCODING, NOT UTF-8
        use std::str;
        str::from_utf8(s)
            .map(|s| s.to_string())
            .map(|s| NameString::Unicode(s))
    }

    pub(crate) fn new_microsoft_from_raw(s: &[u8]) -> Result<NameString, Error> { // TODO: Make a better error

        let manual_convert = |s: &[u8]| {
            let wchar_buf: Vec<u16> = s.chunks(2)
                .map(|bytes| (bytes[0], bytes[1]))
                .map(|(hi, lo)| (hi as u16) << 8 | lo as u16)
                .collect();
            let ms_string = WideString::from_vec(wchar_buf);
            Ok(NameString::Microsoft(ms_string))
        };

        // If we can get away with it, try just casting the slice first
        #[cfg(target_endian = "big")]
        {
            if let Ok(wchar_slice) = s.as_slice_of::<u16>() {
                let ms_string = WideString::from_vec(wchar_slice);
                Ok(NameString::Microsoft(ms_string))
            } else {
                manual_convert(s)
            }
        }
        // If the endians don't match we will always have to manually construct
        // the string
        #[cfg(target_endian = "little")]
        manual_convert(s)
    }

    pub(crate) fn new_other_from_raw(s: &[u8]) -> NameString {
        let buf = s.iter().cloned().collect();
        NameString::Other(buf)
    }
}

#[derive(Debug)]
pub struct NameTable {
    pub format: u16, // Constant `0`
    pub count: u16,
    pub string_offset: u16,
    pub records: Vec<NameRecord>,
}
#[derive(Debug)]
pub struct NameRecord {
    pub platform_id: u16,
    pub platform_specific_id: u16,
    pub language_id: u16,
    pub name_id: u16,
    pub length: u16,
    pub offset: u16,
    pub name: NameString,
}
