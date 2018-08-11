extern crate toy_ttf;
use toy_ttf::tables::glyf::Glyph;
use toy_ttf::math::{Point, Affine};
use toy_ttf::render::*;
use toy_ttf::font::*;

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
    // let glyph = font.get_glyph('S').unwrap();
    // let glyph = font.get_glyph('¼').unwrap();
    // let glyph = font.get_glyph('✌').unwrap();
    let glyph = font.get_glyph('𝕚').unwrap(); // Codepoint: 0x1d55a
    draw_glyph(&font, glyph);

}

fn render_glyph<'a>(font: &Font<'a>, raster: &mut impl Raster, affine: Affine, glyph: Glyph<'a>) {
    use toy_ttf::tables::glyf::{Coordinate, Description};

    fn affine_dc(affine: Affine, dc: DrawCommand) -> DrawCommand {
        dc
        // match dc {
        //     DrawCommand::Line(p1, p2) => DrawCommand::Line(affine * p1, affine *  p2),
        //     DrawCommand::Curve(p1, m, p2) => DrawCommand::Curve(affine*p1, affine*m, affine*p2),
        // }
    }

    match glyph.desc {
        Description::Simple(glyph) => {
            // for contour in glyph.contours() {
            //     for dc in DrawCommands::from_coordinates(contour.into_iter()) {
            //         println!("{:?}", affine_dc(affine, dc));
            //         match dc {
            //             DrawCommand::Line(p1, p2) => raster.draw_line(affine * p1, affine * p2),
            //             DrawCommand::Curve(p1, m, p2) => {
            //                 raster.draw_curve(affine * p1, affine * m, affine * p2);
            //             },
            //         }
            //     }
            // }
            for contour in glyph.contours() {
                for (start, end) in FlattenedDrawCommands::from_coordinates(contour.into_iter()) {
                    // println!("({:?}, {:?})", affine*start, affine*end);
                    raster.add_line(affine * start, affine * end);
                }
            }


            // let first_coord = glyph.coordinates().next().unwrap();
            // let mut last_coord = first_coord;
            // for (c1, c2) in glyph.coordinates().zip(glyph.coordinates().skip(1)) {
            //     last_coord = c2;
            //     draw_coords(raster, c1, c2, affine);
            // }
            // draw_coords(raster, first_coord, last_coord, affine);
            //
            //
            // fn draw_coords(r: &mut Raster, c1: Coordinate, c2: Coordinate, af: Affine) {
            //     let p1 = Point {
            //         x: c1.x as f32,
            //         y: c1.y as f32,
            //     };
            //     let p2 = Point {
            //         x: c2.x as f32,
            //         y: c2.y as f32,
            //     };
            //
            //     r.draw_line(af * p1, af * p2);
            // }
        },
        Description::Composite(glyph) => {
            use toy_ttf::tables::glyf::Glyf;
            use toy_ttf::tables::loca::Loca;
            for (sub_idx, sub_affine) in glyph.coordinates() {
                let glyf: Glyf = font.get_table().unwrap();
                let loca: Loca = font.get_table().unwrap();
                let offset = loca.at(sub_idx);
                let sub_glyph = glyf.at_offset(offset as usize).unwrap();

                // TODO Check affine order
                render_glyph(font, raster, affine * sub_affine, sub_glyph);
            }
        },
    };
}
fn draw_glyph<'a>(font: &Font<'a>, glyph: Glyph<'a>) {

    const PADDING: u32 = 0;
    let width = glyph.header.x_max - glyph.header.x_min;
    let height = glyph.header.y_max - glyph.header.y_min;
    let x_shift = (PADDING as i16 / 2) - glyph.header.x_min;
    let y_shift = (PADDING as i16 / 2) - glyph.header.y_min;
    let affine = Affine::translation(x_shift, y_shift);

    println!("Raster (w, h) = ({}, {})", width as u32 + PADDING, height as u32 + PADDING);
    let mut raster = FillInRaster::new(width as u32 + PADDING, height as u32 + PADDING);
    // let mut raster = OutlineRaster::new(width as u32 + PADDING, height as u32 + PADDING);

    render_glyph(font, &mut raster, affine, glyph);

    const img_file: &str = "RASTER_RESULT.bmp";
    raster.into_dynamic().save(img_file).unwrap();
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
