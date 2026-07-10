use std::env;

use usvg::Color;

use crate::{
    path::{Path, point},
    raster::Canvas,
    svg::draw_svg_file,
};

mod path;
mod raster;
mod svg;

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(last) = args.last() {
        if last.ends_with(".svg") {
            println!("Drawing {last}");
            draw_svg_file(last);
        }
    } else {
        draw_stuff();
    }
}

fn draw_stuff() {
    let mut canvas = Canvas::new(800, 600);
    let mut path = Path::new();

    // counter clockwise
    // we need to ensure that eveverything are counter clockwise
    path.move_to(point(50.0, 0.0))
        .line_to(point(30.0, 330.0))
        .line_to(point(100.0, 300.0))
        .quad_to(point(50.0, 0.0), point(100.0, 0.0))
        .close();

    path.move_to(point(250.0, 0.0))
        .quad_to(point(300.0, 300.0), point(300.0, 0.0))
        .line_to(point(230.0, 330.0))
        // .line_to(point(300.0, 300.0))
        .close();

    let color = Color::new_rgb(255, 0, 0);
    canvas.fill_scanline(&path, &color, 128);
    // fill_scanline(&path, &mut img, &color);

    canvas.save("scanline.png");
}
