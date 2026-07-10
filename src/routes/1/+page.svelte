<script lang="ts">
  import {
      breakCurve1,
      breakCurveRecursiveSubdivision,
      point,
      type Point,
      type QuadraticBezier
  } from "$lib/path";

  let canvas = $state<HTMLCanvasElement>();

  const context = $derived(canvas?.getContext("2d"));

  $effect(() => {
    if (!context) {
      return;
    }

    const curve: QuadraticBezier = {
      start: point(100, 100),
      control: point(100, 300),
      end: point(300, 300),
    };

    // renderActual(curve);
    // const points = breakCurve1(curve, 10)
    const points = breakCurveRecursiveSubdivision(curve, 2)

    renderApproximate(points);
  });

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
    context?.fillRect(start.x - 2, start.y - 2, 4, 4);
    for (const p of points) {
      context?.fillRect(p.x - 2, p.y - 2, 4, 4);
      context!.lineTo(p.x, p.y);
    }
    context!.stroke();
  }
</script>

<h1>Recursive Subdivision</h1>
<canvas bind:this={canvas} width="400" height="400"> </canvas>

<style>
  canvas {
    border: solid 1px black;
  }
</style>
