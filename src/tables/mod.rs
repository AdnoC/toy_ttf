pub mod name;
pub mod head;
pub mod cmap;
pub mod maxp;
pub mod loca;
pub mod font_directory;

pub enum ParseTableErrorInner {
    TableNotFound
}
pub type ParseTableError<'a> = ::nom::Err<&'a [u8], ParseTableErrorInner>;
pub trait PrimaryTable {
    fn tag() -> TableTag;
}

pub struct RecordIterator<'file, T: ::parse::Parse<'file>> {
    next_record: &'file [u8],
    num_left: u16,
    _marker: ::std::marker::PhantomData<T>,
}
impl<'a, T: ::parse::Parse<'a>> Iterator for RecordIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        use ::parse::Parse;

        if self.num_left < 1 {
            return None;
        }

        let (next_record, record) = T::parse(self.next_record);

        self.num_left -= 1;
        self.next_record = next_record;

        Some(record)
    }
}


// https://stackoverflow.com/questions/42199727/how-to-construct-const-integers-from-literal-byte-expressions
#[cfg(target_endian = "big")]
#[macro_export]
macro_rules! u32_code {
    ($w:expr) => {
        ((($w[3] as u32) <<  0) |
         (($w[2] as u32) <<  8) |
         (($w[1] as u32) << 16) |
         (($w[0] as u32) << 24) |
         ((*$w as [u8; 4])[0] as u32 * 0))
    }
}
#[cfg(target_endian = "little")]
#[macro_export]
macro_rules! u32_code {
    ($w:expr) => {
        ((($w[0] as u32) <<  0) |
         (($w[1] as u32) <<  8) |
         (($w[2] as u32) << 16) |
         (($w[3] as u32) << 24) |
         ((*$w as [u8; 4])[0] as u32 * 0))
    }
}

// Varius tags: http://scripts.sil.org/cms/scripts/page.php?site_id=nrsi&id=IWS-AppendixC
#[repr(u32)]
#[derive(Debug, FromPrimitive, PartialEq, Eq)]
pub enum TableTag {
    // Required
    Name = u32_code!(b"name"),
    GlyphOutline = u32_code!(b"glyf"),
    CharacterCodeMapping = u32_code!(b"cmap"),
    PostScriptGlyphName = u32_code!(b"post"),
    FontHeader = u32_code!(b"head"),
    HorizontalMetrics = u32_code!(b"hmtx"),
    HorizontalHeader = u32_code!(b"hhea"),
    HorizontalDeviceMetrics = u32_code!(b"hdmx"),
    GlyphLocation = u32_code!(b"loca"),
    MaximumProfile = u32_code!(b"maxp"),

    // Windows
    Compatibility = u32_code!(b"OS/2"),

    // Apple
    AccentAttachment = u32_code!(b"acnt"),
    AnchorPoint = u32_code!(b"ankr"),
    AxisVariation = u32_code!(b"avar"),
    BitmapData = u32_code!(b"bdat"),
    BitmapFontHeader = u32_code!(b"bhed"),
    BitmapLocation = u32_code!(b"bloc"),
    Baseline = u32_code!(b"bsln"),
    CVTVariation = u32_code!(b"cvar"),
    FontDescriptor = u32_code!(b"fdsc"),
    LayoutFeature = u32_code!(b"feat"),
    FontMetrics = u32_code!(b"fmtx"),
    FontFamilyCompat = u32_code!(b"fond"),
    FontVariation = u32_code!(b"fvar"),
    GlyphVariation = u32_code!(b"gvar"),
    Justification = u32_code!(b"just"),
    ExtendedKerning = u32_code!(b"kerx"),
    LigatureCaret = u32_code!(b"lcar"),
    LanguageTag = u32_code!(b"ltag"),
    Metadata = u32_code!(b"meta"),
    Metamorphosis = u32_code!(b"mort"),
    ExtendedMetamorphosis = u32_code!(b"morx"),
    OpticalBounds = u32_code!(b"opbd"),
    Properties = u32_code!(b"prop"),
    ExtendedBitmaps = u32_code!(b"sbix"),
    Tracking = u32_code!(b"trak"),
    CrossReference = u32_code!(b"xref"),
    GlyphReference = u32_code!(b"Zapf"),

    // OpenType
    GSUB = u32_code!(b"GSUB"),
    GPOS = u32_code!(b"GPOS"),
    GDEF = u32_code!(b"GDEF"),
    BASE = u32_code!(b"BASE"),
    JSFT = u32_code!(b"JSTF"),

    // Graphite
    Silf = u32_code!(b"Silf"),
    Glat = u32_code!(b"Glat"),
    Gloc = u32_code!(b"Gloc"),
    Feat = u32_code!(b"Feat"),

    // FontForge
    FFTimestamp = u32_code!(b"FFTM"), // FontForge timestamp table

    // Optional
    Kerning = u32_code!(b"kern"),
    LTHS = u32_code!(b"LTHS"),
    VerticalMetrics = u32_code!(b"vmtx"),
    VerticalHeader = u32_code!(b"vhea"),
    VDMX = u32_code!(b"VDMX"),
    DSIG = u32_code!(b"DSIG"),
    PCLT = u32_code!(b"PCLT"),
    GridFitAndScanConv = u32_code!(b"gasp"),
    ControlValueProgram = u32_code!(b"prep"),
    FontProgram = u32_code!(b"fpgm"),
    ControlValue = u32_code!(b"cvt "), // Tags less than 4 chars have trailing spaces
    CFF = u32_code!(b"CFF "),
    VORG = u32_code!(b"VORG"),
    EBDT = u32_code!(b"EBDT"),
    EBLC = u32_code!(b"EBLC"),
    EmbeddedBitmapScalingControl = u32_code!(b"EBSC"),
}
