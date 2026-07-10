#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub fn point(x: f64, y: f64) -> Point {
    Point { x, y }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadraticBezier {
    pub start: Point,
    pub control: Point,
    pub end: Point,
}

pub fn lerp(from: f64, to: f64, t: f64) -> f64 {
    from * (1.0 - t) + to * t
}

pub fn sample_curve(curve: &QuadraticBezier, t: f64) -> Point {
    let x1 = lerp(curve.start.x, curve.control.x, t);
    let y1 = lerp(curve.start.y, curve.control.y, t);
    let x2 = lerp(curve.control.x, curve.end.x, t);
    let y2 = lerp(curve.control.y, curve.end.y, t);
    point(lerp(x1, x2, t), lerp(y1, y2, t))
}

/// into polyline
pub fn break_curve_1(curve: &QuadraticBezier, amount: u32) -> Vec<Point> {
    (0..=amount)
        .map(|i| sample_curve(curve, f64::from(i) / f64::from(amount)))
        .collect()
}

pub fn break_curve_recursive_subdivision(curve: &QuadraticBezier, tolerence: f64) -> Vec<Point> {
    let mut points = vec![curve.start];
    walk(curve, tolerence, &mut points);
    points
}

fn walk(curve: &QuadraticBezier, tolerence: f64, points: &mut Vec<Point>) {
    if is_flat(curve, tolerence) {
        points.push(curve.end);
        return;
    }

    let center = sample_curve(curve, 0.5);
    let c1 = point(
        lerp(curve.start.x, curve.control.x, 0.5),
        lerp(curve.start.y, curve.control.y, 0.5),
    );
    let c2 = point(
        lerp(curve.control.x, curve.end.x, 0.5),
        lerp(curve.control.y, curve.end.y, 0.5),
    );

    walk(
        &QuadraticBezier {
            start: curve.start,
            control: c1,
            end: center,
        },
        tolerence,
        points,
    );

    walk(
        &QuadraticBezier {
            start: center,
            control: c2,
            end: curve.end,
        },
        tolerence,
        points,
    );
}

fn is_flat(curve: &QuadraticBezier, tolerence: f64) -> bool {
    let vx = curve.end.x - curve.start.x;
    let vy = curve.end.y - curve.start.y;

    let wx = curve.control.x - curve.start.x;
    let wy = curve.control.y - curve.start.y;

    // measure a distance of a control point to a line from start..end
    let cross = vx * wy - vy * wx;
    tolerence * tolerence * (vx * vx + vy * vy) >= cross * cross
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCommand {
    Move { to: Point },
    Line { to: Point },
    Quad { to: Point, control: Point },
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathSegment {
    Quadratic(QuadraticBezier),
    Line { start: Point, end: Point },
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

fn break_path(_path: &Path) {}
