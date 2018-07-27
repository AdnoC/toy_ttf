extern crate toy_ttf;


#[allow(dead_code)]
const SERIF: &'static str = "fonts/DejaVuSerif.ttf";
#[allow(dead_code)]
const SANS: &'static str = "fonts/DejaVuSans.ttf";
#[allow(dead_code)]
const SANS_MONO: &'static str = "fonts/DejaVuSansMono.ttf";

fn main() {
    use toy_ttf::font::Font;
    use toy_ttf::tables::cmap::CMap;
    use toy_ttf::tables::maxp::MaxP;
    let font_buf = load_file(SANS_MONO);
    // toy_ttf::parse::load_font(&font_buf);

    let font = Font::from_buffer(&font_buf).unwrap();
    let cmap = font.get_table::<CMap>().unwrap();
    println!("{:#?}", cmap);
    let format4 = cmap.format4().unwrap();
    println!("{:#?}", format4);
    let maxp = font.get_table::<MaxP>().unwrap();
    println!("{:#?}", maxp);
    println!("{:#?}", maxp.version_1_ext());
}

fn load_file(name: &str) -> Vec<u8> {
    use std::fs::File;
    use std::io::BufReader;
    use std::io::prelude::*;

    let file = File::open(name).expect("unable to open file");

    let mut reader = BufReader::new(file);
    let mut data = Vec::new();
    reader.read_to_end(&mut data).expect("error reading file");

    data
}
