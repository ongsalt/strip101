<script lang="ts">
  import {
    breakCurve1,
    breakCurveRecursiveSubdivision,
    point,
    type Point,
    type QuadraticBezier,
  } from "$lib/path";

  let canvas = $state<HTMLCanvasElement>();

  const context = $derived(canvas?.getContext("2d"));

  let tolerence = $state(1);
  let shouldRenderSplitPoint = $state(true);
  let curve = $state<QuadraticBezier>({
    start: point(50, 50),
    control: point(50, 150),
    end: point(450, 300),
  });

  const points = $derived(breakCurveRecursiveSubdivision(curve, tolerence));

  const HANDLE_RADIUS = 6;
  let dragging = $state<keyof QuadraticBezier | null>(null);


  $effect(() => {
    if (!context) {
      return;
    }

    context.clearRect(0, 0, 1000, 1000);

    // renderActual(curve);
    // const points = breakCurve1(curve, 10)

    renderApproximate(points);
    renderControlPoints(curve);
  });

  function renderControlPoints(curve: QuadraticBezier) {
    context!.fillStyle = "red";
    for (const p of [curve.start, curve.control, curve.end]) {
      context!.beginPath();
      context!.arc(p.x, p.y, 4, 0, Math.PI * 2);
      context!.fill();
    }
    context!.fillStyle = "black";
  }

  function renderActual(curve: QuadraticBezier) {
    context!.beginPath();
    context!.strokeStyle = "green";
    context!.moveTo(curve.start.x, curve.start.y);
    context!.quadraticCurveTo(
      curve.control.x,
      curve.control.y,
      curve.end.x,
      curve.end.y,
    );
    context!.stroke();
  }

  function renderApproximate(line: Point[]) {
    context!.strokeStyle = "blue";
    const [start, ...points] = line;
    context!.beginPath();
    context!.moveTo(start.x, start.y);
    if (shouldRenderSplitPoint) {
      context?.fillRect(start.x - 2, start.y - 2, 4, 4);
    }
    for (const p of points) {
      if (shouldRenderSplitPoint) {
        context?.fillRect(p.x - 2, p.y - 2, 4, 4);
      }
      context!.lineTo(p.x, p.y);
    }
    context!.stroke();
  }

  // MARKER: control point dragging

  function toCanvas(event: PointerEvent): Point {
    const rect = canvas!.getBoundingClientRect();
    return point(
      ((event.clientX - rect.left) / rect.width) * canvas!.width,
      ((event.clientY - rect.top) / rect.height) * canvas!.height,
    );
  }

  function hit(p: Point): keyof QuadraticBezier | null {
    for (const key of ["control", "start", "end"] as const) {
      const dx = curve[key].x - p.x;
      const dy = curve[key].y - p.y;
      if (dx * dx + dy * dy <= HANDLE_RADIUS * HANDLE_RADIUS) {
        return key;
      }
    }
    return null;
  }

  function onpointerdown(event: PointerEvent) {
    dragging = hit(toCanvas(event));
    if (dragging) {
      canvas!.setPointerCapture(event.pointerId);
    }
  }

  function onpointermove(event: PointerEvent) {
    if (!dragging) {
      return;
    }
    curve[dragging] = toCanvas(event);
  }

  function onpointerup(event: PointerEvent) {
    if (dragging) {
      canvas!.releasePointerCapture(event.pointerId);
      dragging = null;
    }
  }
</script>

<h1>Recursive Subdivision</h1>
<div>
  <label>
    tolerence
    <input type="range" min="0.1" max="10" step="0.01" bind:value={tolerence} />
    <input
      type="number"
      min="0.1"
      bind:value={
        () => tolerence,
        (it) => {
          if (it > 0) {
            tolerence = it;
          }
        }
      }
    />
  </label>

  <div>
    <label>
      <input type="checkbox" bind:checked={shouldRenderSplitPoint} />
      render split points
    </label>
  </div>

  <p>section: {points.length - 1}</p>
</div>
<canvas
  bind:this={canvas}
  width="500"
  height="500"
  {onpointerdown}
  {onpointermove}
  {onpointerup}
></canvas>

<p>there are better way to do this like <a href="https://raphlinus.github.io/graphics/curves/2019/12/23/flatten-quadbez.html">this one</a></p>

<style>
  canvas {
    border: solid 1px black;
    touch-action: none;
  }
</style>
