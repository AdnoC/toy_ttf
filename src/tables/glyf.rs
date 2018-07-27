use parse::{Parse, BufView};

// Total # of glyphs is `num_glyphs` in MaxP table
// Loca table provides index of glyph by glyph_id

#[derive(Debug, Parse)]
pub struct Glyf<'a>(BufView<'a, u8>);
