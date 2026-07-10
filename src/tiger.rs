use std::{fs, ops::Deref};

use image::{ImageFormat, Rgba32FImage, RgbaImage};
use usvg::{Color, FillRule};

use crate::{
    path::{Path, Point, point},
    raster::Canvas,
};

use usvg::tiny_skia_path::PathSegment as UPathSegment;


pub fn draw_svg_file(filename: &str) {
    let svg = fs::read_to_string(filename).unwrap();
    let opt = usvg::Options {
        ..usvg::Options::default()
    };

    let tree = usvg::Tree::from_str(&svg, &opt).unwrap();
    let root = tree.root();

    let mut canvas = Canvas::new(1000, 1000);
    canvas.offset = point(300.0, 300.0);
    canvas.scale = 1.5;
    walk(root, &mut canvas);

    canvas.save("tiger.png");
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
