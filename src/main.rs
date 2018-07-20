extern crate toy_ttf;


#[allow(dead_code)]
const SERIF: &'static str = "fonts/DejaVuSerif.ttf";
#[allow(dead_code)]
const SANS: &'static str = "fonts/DejaVuSans.ttf";
#[allow(dead_code)]
const SANS_MONO: &'static str = "fonts/DejaVuSansMono.ttf";

fn main() {
    let font_buf = load_file(SANS_MONO);
    toy_ttf::parse::load_font(&font_buf);
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
