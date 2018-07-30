extern crate toy_ttf;
use toy_ttf::tables::glyf::Glyph;

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
    use toy_ttf::tables::glyf::Glyf;
    // let font_buf = load_file(ROBOTO);
    let font_buf = load_file(SANS_MONO);
    // toy_ttf::parse::load_font(&font_buf);

    let font = Font::from_buffer(&font_buf).unwrap();
    let glyph = font.get_glyph('¼').unwrap(); // Codepoint is 188
    draw_glyph(glyph);

}

fn draw_glyph<'a>(glyph: Glyph<'a>) {
    use toy_ttf::render::*;

    const PADDING: u32 = 50;
    let width = glyph.header.x_max - glyph.header.x_min;
    let height = glyph.header.y_max - glyph.header.y_min;
    let x_shift = (PADDING as i16 / 2) - glyph.header.x_min;
    let y_shift = (PADDING as i16 / 2) - glyph.header.y_min;

    fn shift_point(p: Point, x_shift: i16, y_shift: i16) -> Point {
        Point {
            x: p.x + x_shift as f32,
            y: p.y + y_shift as f32,
        }
    }

    let mut raster = Raster::new(width as u32 + PADDING, height as u32 + PADDING);

    let first_coord = glyph.coordinates().next().unwrap();
    let mut last_coord = first_coord;
    for (c1, c2) in glyph.coordinates().zip(glyph.coordinates().skip(1)) {
        last_coord = c2;
        let p1 = Point {
            x: c1.x as f32,
            y: c1.y as f32,
        };
        let p1 = shift_point(p1, x_shift, y_shift);
        let p2 = Point {
            x: c2.x as f32,
            y: c2.y as f32,
        };
        let p2 = shift_point(p2, x_shift, y_shift);

        raster.draw_line(p1, p2);
    }

    let p1 = Point {
        x: first_coord.x as f32,
        y: first_coord.y as f32,
    };
    let p1 = shift_point(p1, x_shift, y_shift);
    let p2 = Point {
        x: last_coord.x as f32,
        y: last_coord.y as f32,
    };
    let p2 = shift_point(p2, x_shift, y_shift);

    raster.draw_line(p1, p2);
    const img_file: &str = "RASTER_RESULT.bmp";
    raster.0.save(img_file).unwrap();

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
