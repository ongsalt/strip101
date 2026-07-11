use std::{fs, ops::Deref};

use image::{ImageFormat, Rgba32FImage, RgbaImage};
use usvg::{Color, FillRule};

use crate::{
    path::{Path, Point, point},
    raster::Canvas,
};

use usvg::tiny_skia_path::PathSegment as UPathSegment;


fn render_svg(tree: &usvg::Tree) -> Canvas {
    let mut canvas = Canvas::new(1200, 1200);
    canvas.offset = point(400.0, 400.0);
    canvas.scale = 2.0;
    walk(tree.root(), &mut canvas);
    canvas
}

pub fn draw_svg_file(filename: &str) {
    let svg = fs::read_to_string(filename).unwrap();
    let opt = usvg::Options {
        ..usvg::Options::default()
    };

    let tree = usvg::Tree::from_str(&svg, &opt).unwrap();

    let mut canvas = render_svg(&tree);
    canvas.dump_profile();
    canvas.save("tiger.png");
}

pub fn bench_svg_file(filename: &str, iterations: usize) {
    let svg = fs::read_to_string(filename).unwrap();
    let opt = usvg::Options {
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(&svg, &opt).unwrap();

    // warmup
    let _ = render_svg(&tree);

    let mut total = std::time::Duration::ZERO;
    let mut min = std::time::Duration::MAX;
    let mut max = std::time::Duration::ZERO;
    let mut break_into_lines_total = std::time::Duration::ZERO;

    for _ in 0..iterations {
        let start = std::time::Instant::now();
        let canvas = render_svg(&tree);
        let elapsed = start.elapsed();

        total += elapsed;
        min = min.min(elapsed);
        max = max.max(elapsed);

        if let Some(profile) = canvas.profile() {
            break_into_lines_total += profile.break_into_lines;
        }
    }

    println!(
        "bench {filename} x{iterations}: total={total:?}, avg={:?}, min={min:?}, max={max:?}, break_into_lines_avg={:?}",
        total / iterations as u32,
        break_into_lines_total / iterations as u32,
    );
}

fn walk(parent: &usvg::Group, canvas: &mut Canvas) {
    for node in parent.children() {
        // do stuff...
        match node {
            usvg::Node::Group(group) => walk(group, canvas),
            usvg::Node::Path(p) => {
                let path: Path = p.deref().into();
                if let Some(fill) = p.fill() {
                    match fill.paint() {
                        usvg::Paint::Color(color) => {
                            canvas.fill_scanline(&path, color, fill.opacity().to_u8());
                        }
                        usvg::Paint::LinearGradient(_) => {}
                        usvg::Paint::RadialGradient(_) => {}
                        usvg::Paint::Pattern(_) => {}
                    }
                }
            }
            _ => {}
        }

        // handle subroots as well
        node.subroots(|subroot| walk(subroot, canvas));
    }
}

impl From<&usvg::Path> for Path {
    fn from(value: &usvg::Path) -> Self {
        let mut path = Path::new();
        for segment in value.data().segments() {
            match segment {
                UPathSegment::MoveTo(p) => path.move_to(p.into()),
                UPathSegment::LineTo(p) => path.line_to(p.into()),
                UPathSegment::QuadTo(c1, p) => path.quad_to(p.into(), c1.into()),
                UPathSegment::CubicTo(c1, c2, p) => path.cubic_to(p.into(), c1.into(), c2.into()),
                UPathSegment::Close => path.close(),
            };
        }

        path
    }
}

impl From<usvg::tiny_skia_path::Point> for Point {
    fn from(value: usvg::tiny_skia_path::Point) -> Self {
        point(value.x, value.y)
    }
}
