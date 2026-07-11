use std::{env, fmt::Debug, time::Instant};

use usvg::Color;

use crate::{
    path::{Path, point},
    raster::Canvas,
    svg::{bench_svg_file, draw_svg_file},
};

mod path;
mod raster;
mod svg;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--bench") {
        let svg = args
            .iter()
            .find(|a| a.ends_with(".svg"))
            .map(|s| s.as_str())
            .unwrap_or("tiger.svg");
        bench_svg_file(svg, 100);
    } else if let Some(last) = args.last() {
        if last.ends_with(".svg") {
            let now = Instant::now();
            println!("Drawing {last}. Starting at {now:.?}");

            draw_svg_file(last);

            let done = Instant::now();
            let duration = done - now;
            println!("Finished in {duration:.?}");
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
