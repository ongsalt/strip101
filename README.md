# Todo
- stroke
- correct even odd fill rule
- fix bug when some path are offscreen
- rect clip (viewport)
  - still need to calculate winding number of stuff outside of this
  - when its outside of viewport tile size can be much larger, arbitrary h
- think about arbitrary clipping
- nuke HashMap and do proper tiling

# References
- [Spare strips](https://ethz.ch/content/dam/ethz/special-interest/infk/inst-pls/plf-dam/documents/StudentProjects/MasterTheses/2025-Laurenz-Thesis.pdf)
- [Flattening quadratic Béziers](https://raphlinus.github.io/graphics/curves/2019/12/23/flatten-quadbez.html)
- [Parallel vector graphics rasterization on CPU](https://gasiulis.name/parallel-rasterization-on-cpu/)
- [Fast cubic Bézier curve offsetting.
](https://gasiulis.name/cubic-curve-offsetting/)
- [The Scanline Sweeper: A Glyph Rendering Algorithm](https://www.youtube.com/watch?v=B9bztU1sTFA)