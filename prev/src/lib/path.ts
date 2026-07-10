export type QuadraticBezier = {
  start: Point;
  control: Point;
  end: Point;
};

export type Point = {
  x: number,
  y: number;
};

export function point(x: number, y: number): Point {
  return { x, y };
}

// into polyline
export function breakCurve1(curve: QuadraticBezier, amount: number): Point[] {
  const points: Point[] = [];

  for (let i = 0; i <= amount; i++) {
    points.push(sampleCurve(curve, i / amount));
  }

  return points;
}

export function breakCurveRecursiveSubdivision(curve: QuadraticBezier, tolerence: number): Point[] {
  const points: Point[] = [curve.start];

  function walk(curve: QuadraticBezier) {
    if (isFlat(curve, tolerence)) {
      points.push(curve.end);
    } else {
      const center = sampleCurve(curve, 0.5);
      const cx1 = lerp(curve.start.x, curve.control.x, 0.5);
      const cy1 = lerp(curve.start.y, curve.control.y, 0.5);

      const cx2 = lerp(curve.control.x, curve.end.x, 0.5);
      const cy2 = lerp(curve.control.y, curve.end.y, 0.5);

      walk({
        start: curve.start,
        end: center,
        control: point(cx1, cy1),
      });

      walk({
        start: center,
        end: curve.end,
        control: point(cx2, cy2),
      });
    }
  }

  walk(curve);

  return points;
}

function isFlat(curve: QuadraticBezier, tolerence: number): boolean {
  const vx = curve.end.x - curve.start.x;
  const vy = curve.end.y - curve.start.y;

  const wx = curve.control.x - curve.start.x;
  const wy = curve.control.y - curve.start.y;

  // measure a distance of a control point to a line from start..end
  const cross = vx * wy - vy * wx;
  return tolerence * tolerence * (vx * vx + vy * vy) >= cross * cross;
}

export function sampleCurve(curve: QuadraticBezier, t: number) {
  const x1 = lerp(curve.start.x, curve.control.x, t);
  const y1 = lerp(curve.start.y, curve.control.y, t);
  const x2 = lerp(curve.control.x, curve.end.x, t);
  const y2 = lerp(curve.control.y, curve.end.y, t);
  return point(
    lerp(x1, x2, t),
    lerp(y1, y2, t),
  );
}

export function lerp(from: number, to: number, t: number) {
  return from * (1 - t) + (to * t);
}

export type Path = {
  commands: PathCommand[]
  fillRule?: "non-zero" | "even-odd"
}

export type ClosedPath = {
  segments: PathSegment[]
}

export type PathSegment = ({
  kind: "quadratic",
} & QuadraticBezier) | {
  kind: "line",
  start: Point,
  end: Point;
};

export type PathCommand = {
  kind: "move",
  to: Point
} | {
  kind: "line",
  to: Point,
} | {
  kind: "quad",
  to: Point,
  control: Point
} | {
  kind: "close"
}

function breakPath(path: Path) {

}