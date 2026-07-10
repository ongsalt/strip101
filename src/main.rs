use std::{fs, ops::Deref};

use image::{ImageFormat, Rgba32FImage, RgbaImage};
use usvg::{Color, FillRule};

use crate::{
    path::{Path, Point, point},
    raster::{fill_scanline, raster_band},
};

mod path;
mod raster;

use usvg::tiny_skia_path::PathSegment as UPathSegment;

fn main() {
    let width = 800;
    let height = 600;
    let mut img = RgbaImage::new(width, height);

    let svg = fs::read_to_string("tiger.svg").unwrap();
    let opt = usvg::Options {
        ..usvg::Options::default()
    };

    let tree = usvg::Tree::from_str(&svg, &opt).unwrap();
    walk(&tree.root(), &mut img);

    img.save_with_format("tiger.png", ImageFormat::Png).unwrap();
}

fn walk(parent: &usvg::Group, img: &mut RgbaImage) {
    for node in parent.children() {
        // do stuff...
        match node {
            usvg::Node::Group(group) => walk(group, img),
            usvg::Node::Path(p) => {
                let path: Path = p.deref().into();
                if let Some(fill) = p.fill() {
                    match fill.paint() {
                        usvg::Paint::Color(color) => {
                            fill_scanline(&path, img, color);
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
        node.subroots(|subroot| walk(subroot, img));
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

fn draw_stuff() {
    let width = 800;
    let height = 600;
    let mut img = RgbaImage::new(width, height);

    let mut path = Path::new();

    // counter clockwise
    // we need to ensure that eveverything are counter clockwise
    path.move_to(point(0.0, 0.0))
        .line_to(point(0.0, 200.0))
        .line_to(point(200.0, 200.0))
        .cubic_to(point(0.0, 0.0), point(200.0, 0.0), point(0.0, 200.0))
        .close();

    path.move_to(point(250.0, 0.0))
        .line_to(point(230.0, 330.0))
        .line_to(point(300.0, 300.0))
        .quad_to(point(250.0, 0.0), point(300.0, 0.0))
        .close();

    path.move_to(point(50.0, 250.0))
        .line_to(point(50.0, 350.0))
        .line_to(point(150.0, 350.0))
        .close();

    let color = Color::new_rgb(255, 0, 0);
    fill_scanline(&path, &mut img, &color);

    let save_path = std::path::Path::new("scanline.png");
    img.save_with_format(&save_path, ImageFormat::Png).unwrap();
}
