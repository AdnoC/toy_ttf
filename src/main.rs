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
    use toy_ttf::render::*;

    use toy_ttf::font::{GetTable, Font};
    use toy_ttf::tables::cmap::CMap;
    use toy_ttf::tables::head::Head;
    use toy_ttf::tables::maxp::MaxP;
    use toy_ttf::tables::loca::Loca;
    use toy_ttf::tables::glyf::Glyf;
    // let font_buf = load_file(ROBOTO);
    let font_buf = load_file(SANS_MONO);
    // toy_ttf::parse::load_font(&font_buf);

    let font = Font::from_buffer(&font_buf).unwrap();

    let loca: Loca = font.get_table().unwrap();
    // println!("{:#?}", loca);

    let cmap: CMap = font.get_table().unwrap();
    // println!("{:#?}", cmap);
    // for rec in cmap.encoding_records() {
    //     println!("{:#?}", rec);
    // }
    let format4 = cmap.format4().unwrap();
    // println!("{:#?}", format4);

    let glyf: Glyf = font.get_table().unwrap();

    let glyph_id = format4.lookup_glyph_id('S' as u8 as u16).unwrap();
    let glyph_offset = loca.at(glyph_id as usize);
    println!("glyph_id = {}\tglyph_offset = {}", glyph_id, glyph_offset);
    let glyph = glyf.at_offset(glyph_offset as usize).unwrap();
    for coord in glyph.coordinates() {
        println!("{:?}", coord);
    }
    //
    // let mut x_max = 0;
    // let mut y_max = 0;
    // for coord in glyph.coordinates() {
    //     if coord.x > x_max {
    //         x_max = coord.x;
    //     }
    //     if coord.y > y_max {
    //         y_max = coord.y;
    //     }
    // }
    // println!("img dims = ({}, {})", x_max, y_max);
    //
    // let mut raster = Raster::new(x_max as u32 + 200, y_max as u32 + 200);
    //
    // let first_coord = glyph.coordinates().next().unwrap();
    // let mut last_coord = first_coord;
    // for (c1, c2) in glyph.coordinates().zip(glyph.coordinates().skip(1)) {
    //     last_coord = c2;
    //     let p1 = Point {
    //         x: c1.x as f32,
    //         y: c1.y as f32,
    //     };
    //     let p2 = Point {
    //         x: c2.x as f32,
    //         y: c2.y as f32,
    //     };
    //
    //     raster.draw_line(p1, p2);
    // }
    //
    // let p1 = Point {
    //     x: first_coord.x as f32,
    //     y: first_coord.y as f32,
    // };
    // let p2 = Point {
    //     x: last_coord.x as f32,
    //     y: last_coord.y as f32,
    // };
    //
    // raster.draw_line(p1, p2);
    // const img_file: &str = "RASTER_RESULT.bmp";
    // raster.0.save(img_file).unwrap();
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
