use std::ops::{Add, Mul, Sub};

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
    Quadratic(QuadraticBezier),
    Line(Point, Point),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Path {
    pub commands: Vec<PathCommand>,
    pub fill_rule: FillRule,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClosedPath {
    pub segments: Vec<PathSegment>,
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
        self.commands.push(PathCommand::CubicTo(to, control1, control2));
        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.commands.push(PathCommand::Close);
        self
    }

    // TODO: subpath iter, change path direction if needed
    pub fn break_into_lines(&self) -> Vec<Line> {
        let mut lines = vec![];
        let mut current_starting_point = point(0.0, 0.0);
        let mut current = point(0.0, 0.0);
        for segment in &self.commands {
            match segment {
                PathCommand::MoveTo(point) => {
                    current = *point;
                    current_starting_point = *point;
                }
                PathCommand::LineTo(point) => {
                    lines.push(Line(current, *point));
                    current = *point;
                }
                PathCommand::QuadTo(point, point1) => {
                    let quad = QuadraticBezier {
                        start: current,
                        control: *point1,
                        end: *point,
                    };

                    let points = quad.flatten_recursive_subdivision(0.5);
                    for p in points.into_iter().skip(1) {
                        lines.push(Line(current, p));
                        current = p;
                    }
                }
                PathCommand::CubicTo(point, c1, c2) => {
                    let cubic = CubicBezier {
                        start: current,
                        control1: *c1,
                        control2: *c2,
                        end: *point,
                    };

                    let points = cubic.flatten_recursive_subdivision(0.5);
                    for p in points.into_iter().skip(1) {
                        lines.push(Line(current, p));
                        current = p;
                    }
                }
                PathCommand::Close => {
                    if current != current_starting_point {
                        lines.push(Line(current, current_starting_point));
                        current = current_starting_point;
                    }
                }
            };
        }

        lines
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
}
