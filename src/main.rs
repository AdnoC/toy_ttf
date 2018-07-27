extern crate toy_ttf;

#[allow(dead_code)]
const SERIF: &'static str = "fonts/DejaVuSerif.ttf";
#[allow(dead_code)]
const SANS: &'static str = "fonts/DejaVuSans.ttf";
#[allow(dead_code)]
const SANS_MONO: &'static str = "fonts/DejaVuSansMono.ttf";
#[allow(dead_code)]
const ROBOTO: &'static str = "fonts/Roboto-Regular.ttf";

fn main() {
    use toy_ttf::font::{GetTable, Font};
    use toy_ttf::tables::cmap::CMap;
    use toy_ttf::tables::head::Head;
    use toy_ttf::tables::maxp::MaxP;
    use toy_ttf::tables::loca::Loca;
    // let font_buf = load_file(ROBOTO);
    let font_buf = load_file(SANS_MONO);
    // toy_ttf::parse::load_font(&font_buf);

    let font = Font::from_buffer(&font_buf).unwrap();

    let loca: Loca = font.get_table().unwrap();
    println!("{:#?}", loca);

    let cmap: CMap = font.get_table().unwrap();
    println!("{:#?}", cmap);
    // for rec in cmap.encoding_records() {
    //     println!("{:#?}", rec);
    // }
    // let format4 = cmap.format4().unwrap();
    // println!("{:#?}", format4);
    // let maxp = font.get_table::<MaxP>().unwrap();
    // println!("{:#?}", maxp);
    // println!("{:#?}", maxp.version_1_ext());
    //
}

fn load_file(name: &str) -> Vec<u8> {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    let file = File::open(name).expect("unable to open file");

    let mut reader = BufReader::new(file);
    let mut data = Vec::new();
    reader.read_to_end(&mut data).expect("error reading file");

    data
}
