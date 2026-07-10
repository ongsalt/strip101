use image::{ImageFormat, Rgba32FImage, RgbaImage};
use usvg::{Color, FillRule};

use crate::{
    path::{Path, point},
    raster::{fill_scanline, raster_band},
};

mod path;
mod raster;

fn main() {
    let width = 500;
    let height = 500;
    let mut img = RgbaImage::new(width, height);

    let mut path = Path::new();
    path.set_fill_rule(FillRule::EvenOdd);

    // counter clockwise

    path.move_to(point(0.0, 0.0))
        .line_to(point(0.0, 100.0))
        .line_to(point(100.0, 100.0))
        .line_to(point(100.0, 0.0))
        .close();

    path.move_to(point(150.0, 0.0))
        .quad_to(point(200.0, 200.0), point(0.0, 200.0))
        .close();

    path.move_to(point(250.0, 0.0))
        .line_to(point(230.0, 300.0))
        .line_to(point(300.0, 300.0))
        .close();

    path.move_to(point(50.0, 200.0))
        .line_to(point(50.0, 300.0))
        .line_to(point(150.0, 300.0))
        .close();

    let color = Color::black();
    fill_scanline(&path, &mut img, &color);

    let save_path = std::path::Path::new("scanline.png");
    img.save_with_format(&save_path, ImageFormat::Png).unwrap();
}
