extern crate toy_ttf;
extern crate image;
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
    use toy_ttf::tables::maxp::MaxP;
    use toy_ttf::tables::loca::Loca;
    use toy_ttf::tables::glyf::Glyf;
    // let font_buf = load_file(ROBOTO);
    let font_buf = load_file(SERIF);
    // toy_ttf::parse::load_font(&font_buf);

    let font = Font::from_buffer(&font_buf).unwrap();
    {
        // let glyph = font.get_glyph(' ').unwrap();
        // // let glyph = font.get_glyph('S').unwrap();
        // // let glyph = font.get_glyph('¼').unwrap();
        // // let glyph = font.get_glyph('✌').unwrap();
        // // let glyph = font.get_glyph('𝕚').unwrap(); // Codepoint: 0x1d55a
        // // let glyph = font.get_glyph('²').unwrap(); // Has instructions
        // //
        // let raster = font.render_glyph(glyph, 64);
        //
        // const img_file: &str = "RASTER_RESULT.bmp";
        // raster.into_dynamic().save(img_file).unwrap();
        //
    }

    // draw_str(&font, "Hello,_World!");
    // draw_str_renderedtext(&font, "Hello,_World!");
    // draw_str_renderedtext(&font, "Hello");

    {
        use toy_ttf::render::compositor::*;
        use toy_ttf::tables::hhea::HHEA;
        use toy_ttf::tables::head::Head;

        let rend_met = font.text_render_metrics().unwrap();
        let head: Head = font.get_table().unwrap();

        let size = 64;

        let mut rend_txt = RenderedText::new_left_to_right(rend_met, size, head.units_per_em);

        let first_msg = "Hello,";
        let second_msg = "World!";

        for ch in first_msg.chars() {
            let glyph = font.get_glyph(ch).unwrap();

            let ch_bitmap = font.render_glyph(glyph, size);
            // let ch_bitmap = flip_vertical(&ch_bitmap);

            let placement_metrics = font.placement_metrics(ch, size).expect("Couldn't get placement metrics");

            rend_txt.add_glyph(ch_bitmap, placement_metrics);
        }

        rend_txt.newline();

        for ch in second_msg.chars() {
            let glyph = font.get_glyph(ch).unwrap();

            let ch_bitmap = font.render_glyph(glyph, size);
            // let ch_bitmap = flip_vertical(&ch_bitmap);

            let placement_metrics = font.placement_metrics(ch, size).expect("Couldn't get placement metrics");

            rend_txt.add_glyph(ch_bitmap, placement_metrics);
        }

        const img_file: &str = "RASTER_RESULT.bmp";
        rend_txt.img.save(img_file).unwrap();
    }
}

fn draw_str_renderedtext<'a>(font: &Font<'a>, text: &str) {
    use toy_ttf::render::compositor::*;
    use toy_ttf::tables::hhea::HHEA;
    use toy_ttf::tables::head::Head;

    let head: Head = font.get_table().unwrap();

    let rend_met = font.text_render_metrics().unwrap();
    let size = 64;

    let mut rend_txt = RenderedText::new_left_to_right(rend_met, size, head.units_per_em);

    for ch in text.chars() {
        let glyph = font.get_glyph(ch).unwrap();

        let ch_bitmap = font.render_glyph(glyph, size);
        // let ch_bitmap = flip_vertical(&ch_bitmap);

        let placement_metrics = font.placement_metrics(ch, size).expect("Couldn't get placement metrics");

        rend_txt.add_glyph(ch_bitmap, placement_metrics);
    }

    const img_file: &str = "RASTER_RESULT.bmp";
    rend_txt.img.save(img_file).unwrap();
}

fn draw_str<'a>(font: &Font<'a>, text: &str) {
    use image::{GrayImage, GenericImage, imageops::flip_vertical};

    let mut img = GrayImage::new(0, 0);
    let size = 64;

    // How to actually lay out each char
    // http://freetype.sourceforge.net/freetype2/docs/glyphs/Image3.png
    for ch in text.chars() {
        let glyph = font.get_glyph(ch).unwrap();

        // let ch_dyn = raster.into_dynamic();
        // let ch_bitmap = ch_dyn.to_luma();
        let ch_bitmap = font.render_glyph(glyph, size);

        let ch_bitmap = flip_vertical(&ch_bitmap);

        let (ch_w, ch_h) = ch_bitmap.dimensions();
        let (img_w, img_h) = img.dimensions();

        let old_img = img;

        let width = img_w + ch_w;
        let height = img_h.max(ch_h);

        img = GrayImage::new(width, height);

        img.copy_from(&old_img, 0, 0);
        img.copy_from(&ch_bitmap, img_w, height - ch_h);
    }

    const img_file: &str = "RASTER_RESULT.bmp";
    img.save(img_file).unwrap();
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
