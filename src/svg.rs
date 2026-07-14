use std::{fs, ops::Deref, time::Instant};

use image::{ImageFormat, Rgba32FImage, RgbaImage};
use usvg::{Color, FillRule};

use crate::{
    path::{Path, Point, Transform, point}, raster::Canvas,
};

use usvg::tiny_skia_path::PathSegment as UPathSegment;

fn render_svg(tree: &usvg::Tree, scale: f32) -> Canvas {
    let (width, height) = scaled_size(tree, scale);
    let mut canvas = Canvas::new(width, height);
    canvas.transform = Transform {
        offset: point(0.0, 0.0),
        scale,
    };
    walk(tree.root(), &mut canvas);
    canvas
}

/// viewbox size scaled, rounded up to whole pixels
fn scaled_size(tree: &usvg::Tree, scale: f32) -> (u32, u32) {
    let size = tree.size();
    (
        (size.width() * scale).ceil() as u32,
        (size.height() * scale).ceil() as u32,
    )
}

pub fn draw_svg_file(filename: &str, scale: f32) {
    let svg = fs::read_to_string(filename).unwrap();
    let name = filename.split(".").next().unwrap();

    let opt = usvg::Options {
        ..usvg::Options::default()
    };

    let tree = usvg::Tree::from_str(&svg, &opt).unwrap();

    let now = Instant::now();
    let (width, height) = scaled_size(&tree, scale);
    println!("Drawing {filename} at {width}x{height} scale={scale:.3}");

    let canvas = render_svg(&tree, scale);

    let done = Instant::now();
    let duration = done - now;
    println!("Finished in {duration:.?}");

    canvas.save(&format!("{name}.png"));
}

pub fn bench_svg_file(filename: &str, iterations: usize, scale: f32) {
    let svg = fs::read_to_string(filename).unwrap();
    let opt = usvg::Options {
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(&svg, &opt).unwrap();

    // warmup
    let _ = render_svg(&tree, scale);

    let mut total = std::time::Duration::ZERO;
    let mut min = std::time::Duration::MAX;
    let mut max = std::time::Duration::ZERO;

    for _ in 0..iterations {
        let start = std::time::Instant::now();
        let _ = render_svg(&tree, scale);
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
