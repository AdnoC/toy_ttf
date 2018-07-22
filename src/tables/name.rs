use widestring::WideString;
use std::str::Utf8Error;
#[allow(unused_imports)]
use byte_slice_cast::{AsSliceOf, Error};

// TODO: Handle format 1 name tables

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
    // pub format: u16, // We only handle format 0
    // pub count: u16, // implied by `records.len()`
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

#[derive(Debug)]
pub enum NameIdentifier {
    Copyright = 0,
    FontFamily = 1,
    FontSubfamily = 2,
    SubfamilyIdentifier = 3,
    FullName = 4,
    Version = 5,
    PostscriptName = 6,
    Trademark = 7,
    Manufacturer = 8,
    Designer = 9,
    Description = 10,
    VenderUrl = 11,
    DesignerUrl = 12,
    License = 13,
    LicenseUrl = 14,
    Reserved1 = 15,
    PreferredFontFamily = 16,
    PreferredFontSubfamily  = 17,
    CompatibleFull = 18, // Mac only
    SampleText = 19,
    PostscriptCid = 20,
    WwsFamily = 21,
    WwsSubfamily = 22,
    LightBackgroundPalette = 23,
    DarkBackgroundPalette = 24,
    VariationsPostScriptNamePrefix = 25,
}

// When building a Unicode font for Windows, the platform ID should be 3 and the encoding ID should be 1, and the referenced string data must be encoded in UTF-16BE. When building a symbol font for Windows, the platform ID should be 3 and the encoding ID should be 0, and the referenced string data must be encoded in UTF-16BE. When building a font that will be used on the Macintosh, the platform ID should be 1 and the encoding ID should be 0.
#[derive(Debug)]
pub enum PlatformIdentifier {
    Unicode = 0,
    Macintosh = 1,
    /// Deprecated
    ISO = 2,
    Microsoft = 3,
    // Custom = 4, // Unsupported
}

#[derive(Debug)]
pub enum MacEncodingIdentifier {
    Roman = 0,
    Japanese = 1,
    TraditionalChinese = 2,
    Korean = 3,
    Arabic = 4,
    Hebrew = 5,
    Greek = 6,
    Russian = 7,
    RSymbol = 8,
    Devanagari = 9,
    Gurmukhi = 10,
    Gujarati = 11,
    Oriya = 12,
    Bengali = 13,
    Tamil = 14,
    Telugu = 15,
    Kannada = 16,
    Malayalam = 17,
    Sinhalese = 18,
    Burmese = 19,
    Khmer = 20,
    Thai = 21,
    Laotian = 22,
    Georgian = 23,
    Armenian = 24,
    SimplifiedChinese = 25,
    Tibetan = 26,
    Mongolian = 27,
    Geez = 28,
    Slavic = 29,
    Vietnamese = 30,
    Sindhi = 31,
    Uninterpreted = 32,
}

#[derive(Debug)]
pub enum UnicodeEncodingIdentifier {
    Version1_0 = 0,
    Version1_1 = 1,
    Iso10646 = 2,
    BmpVersion2_0 = 3,
    NonBmpVersion2_0 = 4,
    VariationSequences = 5,
    Full = 6,
}

#[derive(Debug)]
pub enum WindowsEncodingIdentifier {
    Symbol = 0,
    Ucs2 = 1,
    ShiftJis = 2,
    Prc = 3,
    Big5 = 4,
    Wansung = 5,
    Johab = 6,
    Reserved1 = 7,
    Reserved2 = 8,
    Reserved3 = 9,
    Ucs4 = 10,
}

#[derive(Debug)]
pub enum MacLanguageIdentifier {
    English = 0,
    French = 1,
    German = 2,
    Italian = 3,
    Dutch = 4,
    Swedish = 5,
    Spanish = 6,
    Danish = 7,
    Portuguese = 8,
    Norwegian = 9,
    Hebrew = 10,
    Japanese = 11,
    Arabic = 12,
    Finnish = 13,
    Greek = 14,
    Icelandic = 15,
    Maltese = 16,
    Turkish = 17,
    Croatian = 18,
    ChineseTraditional = 19,
    Urdu = 20,
    Hindi = 21,
    Thai = 22,
    Korean = 23,
    Lithuanian = 24,
    Polish = 25,
    Hungarian = 26,
    Estonian = 27,
    Latvian = 28,
    Sami = 29,
    Faroese = 30,
    FarsiPersian = 31,
    Russian = 32,
    ChineseSimplified = 33,
    Flemish = 34,
    IrishGaelic = 35,
    Albanian = 36,
    Romanian = 37,
    Czech = 38,
    Slovak = 39,
    Slovenian = 40,
    Yiddish = 41,
    Serbian = 42,
    Macedonian = 43,
    Bulgarian = 44,
    Ukrainian = 45,
    Byelorussian = 46,
    Uzbek = 47,
    Kazakh = 48,
    AzerbaijaniCyrillicScript = 49,
    AzerbaijaniArabicScript = 50,
    Armenian = 51,
    Georgian = 52,
    Moldavian = 53,
    Kirghiz = 54,
    Tajiki = 55,
    Turkmen = 56,
    MongolianMongolianScript = 57,
    MongolianCryllicScript = 58,
    Pashto = 59,
    Kurdish = 60,
    Kashmiri = 61,
    Sindhi = 62,
    Tibetan = 63,
    Nepali = 64,
    Sanskrit = 65,
    Marathi = 66,
    Bengali = 67,
    Assamese = 68,
    Gujarati = 69,
    Punjabi = 70,
    Oriya = 71,
    Malayalam = 72,
    Kannada = 73,
    Tamil = 74,
    Telugu = 75,
    Sinhalese = 76,
    Burmese = 77,
    Khmer = 78,
    Lao = 79,
    Vietnamese = 80,
    Indonesian = 81,
    Tagalog = 82,
    MalayRomanScript = 83,
    MalayArabicScript = 84,
    Amharic = 85,
    Tigrinya = 86,
    Galla = 87,
    Somali = 88,
    Swahili = 89,
    KinyarwandaRuanda = 90,
    Rundi = 91,
    NyanjaChewa = 92,
    Malagasy = 93,
    Esperanto = 94,
    Welsh = 128,
    Basque = 129,
    Catalan = 130,
    Latin = 131,
    Quechua = 132,
    Guarani = 133,
    Aymara = 134,
    Tatar = 135,
    Uighur = 136,
    Dzongkha = 137,
    JavaneseRomanScrit = 138,
    SundaneseRomanScrit = 139,
    Galician = 140,
    Afrikaans = 141,
    Breton = 142,
    Inuktitut = 143,
    ScottishGaelic = 144,
    ManxGaelic = 145,
    IrishGaelicDot = 146, // With dot above
    Tongan = 147,
    GreekPolytonic = 148,
    Greenlandic = 149,
    AzerbaijaniRomanScript = 150,
}

pub enum WindowsLanguageIdentifier {
    // TODO: Copy in the list (https://docs.microsoft.com/en-us/typography/opentype/spec/name)
}

pub enum IsoEncodingIdentifier {
    // TODO: Copy in the list (https://docs.microsoft.com/en-us/typography/opentype/spec/name)
}
