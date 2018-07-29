use image::GrayImage;

pub struct Raster(pub GrayImage);

impl Raster {
    pub fn new(width: u32, height: u32) -> Raster {
        use image::Luma;
        Raster(GrayImage::from_pixel(width, height, Luma { data: [0xFF] }))
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, brightness: u8) {
        use image::Luma;
        self.0.put_pixel(x, y, Luma { data: [brightness] });
    }

    // TODO: Anti-aliasing
    pub fn draw_line(&mut self, start: Point, end: Point) {
        // TODO: Handle points outside of the image

        if start.y == end.y {
            self.draw_horizontal_line(start, end);
            return;
        }

        // Reorient to always draw up
        let (start, end) = if start.y < end.y {
            (start, end)
        } else {
            (end, start)
        };

        let dxdy = (end.x - start.x) / (end.y - start.y);
        let mut x = start.x;

        for y_pix in (start.y as usize)..(end.y.ceil() as usize) {
            let y = y_pix as f32;
            let y_pix = y_pix as u32;
            let dy = end.y.min(y + 1.) - y.max(start.y);
            let next_x = x + dxdy * dy;

            // Reorient to always draw right
            let (x0, x1) = if x < next_x {
                (x, next_x)
            } else {
                (next_x, x)
            };

            let x0_pix = x0 as u32;
            let x1_pix = x1 as u32;

            self.put_pixel(x0_pix, y_pix, 0);
            self.put_pixel(x1_pix, y_pix, 0);


            x = next_x;
        }
    }

    pub fn draw_horizontal_line(&mut self, start: Point, end: Point) {
        let (start, end) = if start.x < end.x {
            (start, end)
        } else {
            (end, start)
        };
        let y = start.y as u32;
        for x in (start.x as u32)..(end.x as u32) {
            self.put_pixel(x, y, 0);
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Point {
    pub x: f32,
    pub y: f32
}
