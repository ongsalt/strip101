use clap::Parser;
use usvg::Color;

use crate::{
    path::{Path, point},
    raster::Canvas,
    svg::{bench_svg_file, draw_svg_file},
    vello::draw_svg_file_vello,
};

mod path;
mod raster;
mod svg;
mod vello;

#[derive(Parser)]
struct Args {
    /// SVG file to render
    svg: Option<String>,

    /// Render with vello_cpu instead of the scanline rasterizer
    #[arg(long)]
    vello: bool,

    /// Benchmark rendering (1000 iterations)
    #[arg(long)]
    bench: bool,

    /// Scale factor applied to the svg viewbox size
    #[arg(long, default_value_t = 1.0)]
    scale: f32,
}

fn main() {
    let args = Args::parse();

    if args.vello {
        draw_svg_file_vello(args.svg.as_deref().unwrap_or("tiger.svg"), args.scale);
    } else if args.bench {
        bench_svg_file(args.svg.as_deref().unwrap_or("tiger.svg"), 1000, args.scale);
    } else if let Some(svg) = args.svg.as_deref() {
        draw_svg_file(svg, args.scale);
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
    canvas.fill(&path, &color, 128);

    canvas.save("scanline.png");
}
