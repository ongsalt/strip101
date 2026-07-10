import type { Path } from "$lib/path";

export function raster(path: Path, to: ImageData) {
  const target = to.data;

  // break image into h=16px bands, put each curve into it
  // assign line segments to each band
  // then just do it like normal scanline?

}