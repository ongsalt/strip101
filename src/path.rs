use std::{
    ops::{Add, Mul, Sub},
    vec,
};

use usvg::FillRule;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

pub const fn point(x: f32, y: f32) -> Point {
    Point { x, y }
}

impl Point {
    pub fn lerp(self, to: Self, t: f32) -> Self {
        self + (to - self) * t
    }

    /// z-component of the 3D cross product; signed area of the parallelogram
    pub fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }
}

impl Add for Point {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        point(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Point {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        point(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Point {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        point(self.x * rhs, self.y * rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadraticBezier {
    pub start: Point,
    pub control: Point,
    pub end: Point,
}

impl QuadraticBezier {
    pub fn sample(&self, t: f32) -> Point {
        let a = self.start.lerp(self.control, t);
        let b = self.control.lerp(self.end, t);
        a.lerp(b, t)
    }

    /// de Casteljau split into two curves meeting at `sample(t)`
    pub fn split(&self, t: f32) -> (Self, Self) {
        let a = self.start.lerp(self.control, t);
        let b = self.control.lerp(self.end, t);
        let mid = a.lerp(b, t);

        (
            Self {
                start: self.start,
                control: a,
                end: mid,
            },
            Self {
                start: mid,
                control: b,
                end: self.end,
            },
        )
    }

    /// distance of the control point to the chord start..end is within `tolerance`
    pub fn is_flat(&self, tolerance: f32) -> bool {
        let chord = self.end - self.start;
        let arm = self.control - self.start;

        let cross = chord.cross(arm);
        tolerance * tolerance * chord.length_squared() >= cross * cross
    }

    /// polyline of `segments + 1` points sampled at even `t`
    pub fn flatten_uniform(self, segments: u16) -> impl Iterator<Item = Point> {
        let segments = segments.max(1);
        (0..=segments).map(move |i| self.sample(f32::from(i) / f32::from(segments)))
    }

    /// polyline that stays within `tolerance` of the curve
    pub fn flatten_recursive_subdivision(self, tolerance: f32) -> Vec<Point> {
        fn walk(curve: QuadraticBezier, tolerance: f32, out: &mut Vec<Point>) {
            if curve.is_flat(tolerance) {
                out.push(curve.end);
                return;
            }

            let (left, right) = curve.split(0.5);
            walk(left, tolerance, out);
            walk(right, tolerance, out);
        }

        let mut points = vec![self.start];
        walk(self, tolerance, &mut points);
        points
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezier {
    pub start: Point,
    pub control1: Point,
    pub control2: Point,
    pub end: Point,
}

impl CubicBezier {
    pub fn sample(&self, t: f32) -> Point {
        let a = self.start.lerp(self.control1, t);
        let b = self.control1.lerp(self.control2, t);
        let c = self.control2.lerp(self.end, t);

        let d = a.lerp(b, t);
        let e = b.lerp(c, t);
        d.lerp(e, t)
    }

    pub fn split(&self, t: f32) -> (Self, Self) {
        let ab = self.start.lerp(self.control1, t);
        let bc = self.control1.lerp(self.control2, t);
        let cd = self.control2.lerp(self.end, t);
        let abc = ab.lerp(bc, t);
        let bcd = bc.lerp(cd, t);
        let mid = abc.lerp(bcd, t); // = B(t), on the curve

        (
            Self {
                start: self.start,
                control1: ab,
                control2: abc,
                end: mid,
            },
            Self {
                start: mid,
                control1: bcd,
                control2: cd,
                end: self.end,
            },
        )
    }

    pub fn is_flat(&self, tolerance: f32) -> bool {
        let chord = self.end - self.start;

        // if chord.dot(chord) < 1e-12 {
        //     // chord is a point: flat only if controls are also at that point
        //     return (self.control1 - self.start)
        //         .dot(self.control1 - self.start)
        //         .max((self.control2 - self.start).dot(self.control2 - self.start))
        //         <= tolerance * tolerance;
        // }

        let arm1 = (self.control1 - self.start).cross(chord).abs();
        let arm2 = (self.control2 - self.start).cross(chord).abs();

        let cross = arm1.max(arm2);
        tolerance * tolerance * chord.length_squared() >= cross * cross
    }

    /// polyline that stays within `tolerance` of the curve
    pub fn flatten_recursive_subdivision(self, tolerance: f32) -> Vec<Point> {
        fn walk(curve: CubicBezier, tolerance: f32, out: &mut Vec<Point>, depth: u32) {
            if curve.is_flat(tolerance) {
                out.push(curve.end);
                return;
            }

            let (left, right) = curve.split(0.5);
            walk(left, tolerance, out, depth + 1);
            walk(right, tolerance, out, depth + 1);
        }

        let mut points = vec![self.start];
        walk(self, tolerance, &mut points, 0);
        points
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    // (target, control)
    QuadTo(Point, Point),
    CubicTo(Point, Point, Point),
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathSegment {
    Line(Point, Point),
    Quadratic(QuadraticBezier),
    Cubic(CubicBezier),
}

impl PathSegment {
    fn start(&self) -> Point {
        match self {
            PathSegment::Line(start, _) => *start,
            PathSegment::Quadratic(curve) => curve.start,
            PathSegment::Cubic(curve) => curve.start,
        }
    }

    fn end(&self) -> Point {
        match self {
            PathSegment::Line(_, end) => *end,
            PathSegment::Quadratic(curve) => curve.end,
            PathSegment::Cubic(curve) => curve.end,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Path {
    pub commands: Vec<PathCommand>,
    pub fill_rule: FillRule,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SubPath {
    pub segments: Vec<PathSegment>,
}

impl SubPath {
    // to determine if its ccw or not
    fn shoelace(&self) -> f32 {
        let mut area = 0.0;

        for segment in &self.segments {
            area += segment.start().x * segment.end().y;
            area -= segment.end().x * segment.start().y;
        }

        area
    }

    fn reverse(&mut self) {
        let mut out = vec![];
        for segment in self.segments.iter().rev() {
            match segment {
                PathSegment::Line(p1, p2) => {
                    out.push(PathSegment::Line(*p2, *p1));
                }
                PathSegment::Quadratic(QuadraticBezier {
                    start,
                    control,
                    end,
                }) => out.push(PathSegment::Quadratic(QuadraticBezier {
                    start: *end,
                    control: *control,
                    end: *start,
                })),
                PathSegment::Cubic(CubicBezier {
                    start,
                    control1,
                    control2,
                    end,
                }) => {
                    out.push(PathSegment::Cubic(CubicBezier {
                        start: *end,
                        control1: *control2,
                        control2: *control1,
                        end: *start,
                    }));
                }
            }
        }

        self.segments = out
    }

    fn write_lines(&self, output: &mut Vec<Line>) {
        for segment in &self.segments {
            match segment {
                PathSegment::Line(p0, p1) => output.push(Line(*p0, *p1)),
                PathSegment::Quadratic(curve) => {
                    let points = curve.flatten_recursive_subdivision(0.5);
                    let mut current = points[0];
                    for p in points.into_iter().skip(1) {
                        output.push(Line(current, p));
                        current = p
                    }
                }
                PathSegment::Cubic(curve) => {
                    let points = curve.flatten_recursive_subdivision(0.5);
                    let mut current = points[0];
                    for p in points.into_iter().skip(1) {
                        output.push(Line(current, p));
                        current = p
                    }
                }
            };
        }
    }
}

impl Path {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn set_fill_rule(&mut self, rule: FillRule) {
        self.fill_rule = rule;
    }

    pub fn move_to(&mut self, to: Point) -> &mut Self {
        self.commands.push(PathCommand::MoveTo(to));
        self
    }

    pub fn line_to(&mut self, to: Point) -> &mut Self {
        self.commands.push(PathCommand::LineTo(to));
        self
    }

    pub fn quad_to(&mut self, to: Point, control: Point) -> &mut Self {
        self.commands.push(PathCommand::QuadTo(to, control));
        self
    }

    pub fn cubic_to(&mut self, to: Point, control1: Point, control2: Point) -> &mut Self {
        self.commands
            .push(PathCommand::CubicTo(to, control1, control2));
        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.commands.push(PathCommand::Close);
        self
    }

    pub fn break_into_subpath(&self) -> SubPathIter<'_> {
        SubPathIter {
            commands: self.commands.iter(),
        }
    }

    pub fn break_into_lines(&self) -> Vec<Line> {
        let mut out = vec![];
        out.reserve(256);

        for subpath in self.break_into_subpath() {
            // let area = subpath.shoelace();
            // if area.is_sign_positive() {
            //     subpath.reverse();
            // }
            subpath.write_lines(&mut out);
        }

        out
    }
}

pub struct SubPathIter<'a> {
    commands: std::slice::Iter<'a, PathCommand>,
}

impl<'a> Iterator for SubPathIter<'a> {
    type Item = SubPath;

    fn next(&mut self) -> Option<SubPath> {
        let mut current: Option<SubPath> = None;
        let mut current_starting_point = point(0.0, 0.0);
        let mut curren_pos: Point = Point { x: 0.0, y: 0.0 };

        for command in &mut self.commands {
            match command {
                PathCommand::MoveTo(point) => {
                    current = Some(SubPath { segments: vec![] });
                    curren_pos = *point;
                    current_starting_point = *point;
                }
                PathCommand::LineTo(point) => {
                    current
                        .as_mut()
                        .expect("Invalid path operation")
                        .segments
                        .push(PathSegment::Line(curren_pos, *point));
                    curren_pos = *point
                }
                PathCommand::QuadTo(target, c1) => {
                    current
                        .as_mut()
                        .expect("Invalid path operation")
                        .segments
                        .push(PathSegment::Quadratic(QuadraticBezier {
                            start: curren_pos,
                            control: *c1,
                            end: *target,
                        }));
                    curren_pos = *target;
                }
                PathCommand::CubicTo(target, c1, c2) => {
                    current
                        .as_mut()
                        .expect("Invalid path operation")
                        .segments
                        .push(PathSegment::Cubic(CubicBezier {
                            start: curren_pos,
                            control1: *c1,
                            control2: *c2,
                            end: *target,
                        }));
                    curren_pos = *target;
                }

                PathCommand::Close => {
                    if current_starting_point != curren_pos {
                        current
                            .as_mut()
                            .expect("Invalid path operation")
                            .segments
                            .push(PathSegment::Line(curren_pos, current_starting_point));
                    }

                    return Some(current.expect("Invalid path operation"));
                }
            }
        }

        // TODO: auto close
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line(pub Point, pub Point);

impl Line {
    pub fn sample(&self, t: f32) -> Point {
        self.0.lerp(self.1, t)
    }

    pub fn split_at_y(&self, y: f32) -> Option<(Line, Line)> {
        let (a, b) = (self.0, self.1);
        let dy = b.y - a.y;
        if dy == 0.0 {
            return None;
        }

        let t = (y - a.y) / dy;
        if !(0.0..=1.0).contains(&t) {
            return None;
        }

        let p = point(a.x + t * (b.x - a.x), y);
        Some((Line(a, p), Line(p, b)))
    }

    pub fn y_bounds(&self) -> (u32, u32) {
        if self.0.y < self.1.y {
            (self.0.y.floor() as u32, self.1.y.floor() as u32)
        } else {
            (self.1.y.floor() as u32, self.0.y.floor() as u32)
        }
    }

    // cell bounds, so its floor
    pub fn x_bounds(&self) -> (u32, u32) {
        if self.0.x < self.1.x {
            (self.0.x.floor() as u32, self.1.x.floor() as u32)
        } else {
            (self.1.x.floor() as u32, self.0.x.floor() as u32)
        }
    }

    pub fn min_x(&self) -> u32 {
        self.0.x.min(self.1.x).floor() as u32
    }

    // -1 up, 1 down
    pub fn dir(&self) -> f32 {
        if self.1.y > self.0.y { 1.0 } else { -1.0 }
    }

    /// portion of the line inside the strip `start_y..end_y`, or `None` if disjoint.
    /// keeps the original start-to-end direction, so winding is preserved.
    /// `start_y` must be less than `end_y`
    pub fn clip_y(&self, start_y: u32, end_y: u32) -> Option<Line> {
        let (a, b) = (self.0, self.1);
        let y1 = start_y as f32;
        let y2 = end_y as f32;

        let dy = b.y - a.y;
        if dy == 0.0 {
            // horizontal: wholly inside or wholly outside
            return (y1..=y2).contains(&a.y).then_some(*self);
        }

        let ta = ((y1 - a.y) / dy).clamp(0.0, 1.0);
        let tb = ((y2 - a.y) / dy).clamp(0.0, 1.0);
        let (enter, exit) = if ta <= tb { (ta, tb) } else { (tb, ta) };

        if exit <= enter {
            // both ends clamped to the same side: no overlap
            return None;
        }

        Some(Line(self.sample(enter), self.sample(exit)))
    }

    /// `start_x` must be less than `end_x`
    pub fn clip_x(&self, start_x: u32, end_x: u32) -> Option<Line> {
        let (a, b) = (self.0, self.1);
        let x1 = start_x as f32;
        let x2 = end_x as f32;

        let dx = b.x - a.x;
        if dx == 0.0 {
            // vertical: wholly inside or wholly outside
            return (x1..=x2).contains(&a.x).then_some(*self);
        }

        let ta = ((x1 - a.x) / dx).clamp(0.0, 1.0);
        let tb = ((x2 - a.x) / dx).clamp(0.0, 1.0);
        let (enter, exit) = if ta <= tb { (ta, tb) } else { (tb, ta) };

        if exit <= enter {
            // both ends clamped to the same side: no overlap
            return None;
        }

        Some(Line(self.sample(enter), self.sample(exit)))
    }

    pub fn bounds(&self) -> Rect {
        let (x1, x2) = self.x_bounds();
        let (y1, y2) = self.y_bounds();
        Rect { x1, y1, x2, y2 }
    }
}

pub struct Rect {
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
}

impl Rect {
    /// start must be less that ends
    pub fn new(x1: u32, y1: u32, x2: u32, y2: u32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    pub fn from_points(a: Point, b: Point) -> Self {
        let x1 = a.x.min(b.x) as u32;
        let y1 = a.y.min(b.y) as u32;
        let x2 = a.x.max(b.x) as u32;
        let y2 = a.y.max(b.y) as u32;
        Self { x1, y1, x2, y2 }
    }

    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let x1 = self.x1.max(other.x1);
        let y1 = self.y1.max(other.y1);
        let x2 = self.x2.min(other.x2);
        let y2 = self.y2.min(other.y2);
        if x1 > x2 || y1 > y2 {
            return None;
        }
        Some(Self { x1, x2, y1, y2 })
    }
}
