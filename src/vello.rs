use std::{fs, time::Instant};

use vello_cpu::{
    Pixmap, RenderContext, Resources,
    color::{AlphaColor, Srgb},
    kurbo::{Affine, BezPath, Cap, Join, Point, Stroke},
    peniko::Fill,
};

use usvg::tiny_skia_path::PathSegment as UPathSegment;

pub fn draw_svg_file_vello(filename: &str) {
    let svg = fs::read_to_string(filename).unwrap();
    let name = filename.split('.').next().unwrap();

    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(&svg, &opt).unwrap();

    let width: u16 = 1200;
    let height: u16 = 1200;

    // fit the svg into the canvas, centered
    let size = tree.size();
    let scale = (width as f64 / size.width() as f64).min(height as f64 / size.height() as f64);
    let tx = (width as f64 - size.width() as f64 * scale) / 2.0;
    let ty = (height as f64 - size.height() as f64 * scale) / 2.0;
    let view = Affine::translate((tx, ty)) * Affine::scale(scale);

    println!("Drawing {filename} with vello_cpu at {width}x{height} scale={scale:.3}");
    let now = Instant::now();

    let mut context = RenderContext::new(width, height);
    let mut resources = Resources::new();

    walk(tree.root(), &mut context, view);

    let mut pixmap = Pixmap::new(width, height);
    context.flush();
    context.render_to_pixmap(&mut resources, &mut pixmap);

    println!("Finished in {:?}", now.elapsed());

    let png = pixmap.into_png().unwrap();
    fs::write(format!("{name}_vello.png"), png).unwrap();
    println!("Saved {name}_vello.png");
}

fn walk(parent: &usvg::Group, context: &mut RenderContext, view: Affine) {
    for node in parent.children() {
        match node {
            usvg::Node::Group(group) => walk(group, context, view),
            usvg::Node::Path(p) => {
                if !p.is_visible() {
                    continue;
                }

                let path = to_bez_path(p);
                context.set_transform(view * to_affine(p.abs_transform()));

                if let Some(fill) = p.fill() {
                    if let usvg::Paint::Color(color) = fill.paint() {
                        context.set_fill_rule(match fill.rule() {
                            usvg::FillRule::NonZero => Fill::NonZero,
                            usvg::FillRule::EvenOdd => Fill::EvenOdd,
                        });
                        context.set_paint(to_color(color, fill.opacity().to_u8()));
                        context.fill_path(&path);
                    }
                }

                if let Some(stroke) = p.stroke() {
                    if let usvg::Paint::Color(color) = stroke.paint() {
                        context.set_stroke(to_stroke(stroke));
                        context.set_paint(to_color(color, stroke.opacity().to_u8()));
                        context.stroke_path(&path);
                    }
                }
            }
            _ => {}
        }

        node.subroots(|subroot| walk(subroot, context, view));
    }
}

fn to_bez_path(p: &usvg::Path) -> BezPath {
    let mut path = BezPath::new();
    for segment in p.data().segments() {
        match segment {
            UPathSegment::MoveTo(a) => path.move_to(pt(a)),
            UPathSegment::LineTo(a) => path.line_to(pt(a)),
            UPathSegment::QuadTo(c, a) => path.quad_to(pt(c), pt(a)),
            UPathSegment::CubicTo(c1, c2, a) => path.curve_to(pt(c1), pt(c2), pt(a)),
            UPathSegment::Close => path.close_path(),
        }
    }
    path
}

fn pt(p: usvg::tiny_skia_path::Point) -> Point {
    Point::new(p.x as f64, p.y as f64)
}

fn to_affine(t: usvg::Transform) -> Affine {
    Affine::new([
        t.sx as f64,
        t.ky as f64,
        t.kx as f64,
        t.sy as f64,
        t.tx as f64,
        t.ty as f64,
    ])
}

fn to_color(c: &usvg::Color, alpha: u8) -> AlphaColor<Srgb> {
    AlphaColor::from_rgba8(c.red, c.green, c.blue, alpha)
}

fn to_stroke(s: &usvg::Stroke) -> Stroke {
    Stroke::new(s.width().get() as f64)
        .with_miter_limit(s.miterlimit().get() as f64)
        .with_caps(match s.linecap() {
            usvg::LineCap::Butt => Cap::Butt,
            usvg::LineCap::Round => Cap::Round,
            usvg::LineCap::Square => Cap::Square,
        })
        .with_join(match s.linejoin() {
            usvg::LineJoin::Miter | usvg::LineJoin::MiterClip => Join::Miter,
            usvg::LineJoin::Round => Join::Round,
            usvg::LineJoin::Bevel => Join::Bevel,
        })
}
