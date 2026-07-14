use std::{fs, ops::Deref, time::Instant};

use image::{ImageFormat, Rgba32FImage, RgbaImage};
use usvg::{Color, FillRule};

use crate::{
    path::{Path, Point, Transform, point}, raster::Canvas,
};

use usvg::tiny_skia_path::PathSegment as UPathSegment;

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 1200;

fn render_svg(tree: &usvg::Tree) -> Canvas {
    let mut canvas = Canvas::new(WIDTH, HEIGHT);
    let (scale, offset) = fit(tree);
    canvas.transform = Transform { offset, scale };
    walk(tree.root(), &mut canvas);
    canvas
}

/// fit the svg into the canvas, centered
fn fit(tree: &usvg::Tree) -> (f32, Point) {
    let size = tree.size();
    let scale = (WIDTH as f32 / size.width()).min(HEIGHT as f32 / size.height());
    let offset = point(
        (WIDTH as f32 - size.width() * scale) / 2.0,
        (HEIGHT as f32 - size.height() * scale) / 2.0,
    );
    (scale, offset)
}

pub fn draw_svg_file(filename: &str) {
    let svg = fs::read_to_string(filename).unwrap();
    let name = filename.split(".").next().unwrap();

    let opt = usvg::Options {
        ..usvg::Options::default()
    };

    let tree = usvg::Tree::from_str(&svg, &opt).unwrap();

    let now = Instant::now();
    let (scale, _) = fit(&tree);
    println!("Drawing {filename} at {WIDTH}x{HEIGHT} scale={scale:.3}");

    let canvas = render_svg(&tree);

    let done = Instant::now();
    let duration = done - now;
    println!("Finished in {duration:.?}");

    canvas.save(&format!("{name}.png"));
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

    for _ in 0..iterations {
        let start = std::time::Instant::now();
        let _ = render_svg(&tree);
        let elapsed = start.elapsed();

        total += elapsed;
        min = min.min(elapsed);
        max = max.max(elapsed);
    }

    println!(
        "bench {filename} x{iterations}: total={total:?}, avg={:?}, min={min:?}, max={max:?}",
        total / iterations as u32,
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
                            canvas.fill(&path, color, fill.opacity().to_u8());
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
        // path data is in local space, so bake in the accumulated group transforms
        let t = value.abs_transform();
        let map = |p: usvg::tiny_skia_path::Point| {
            let mut p = p;
            t.map_point(&mut p);
            Point::from(p)
        };

        let mut path = Path::new();
        for segment in value.data().segments() {
            match segment {
                UPathSegment::MoveTo(p) => path.move_to(map(p)),
                UPathSegment::LineTo(p) => path.line_to(map(p)),
                UPathSegment::QuadTo(c1, p) => path.quad_to(map(p), map(c1)),
                UPathSegment::CubicTo(c1, c2, p) => path.cubic_to(map(p), map(c1), map(c2)),
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
