# How the GPU water works

The water is a **FLIP/PIC fluid** on a MAC grid. The method is the standard one
(Zhu & Bridson, *Animating Sand as a Fluid*; Bridson, *Fluid Simulation for Computer
Graphics*; van der Laan, *Screen Space Fluid Rendering*). Every line is my own; none is
taken from any other mod.

## Two copies of the same method

- `watersim/` — the solver in plain Rust. It runs on the CPU and is what the tests check,
  so the method is proven even with no GPU here. Volume is conserved, a dropped column
  settles flat, water runs downhill, still water stays still, no cell clumps.
- `shaders/*.comp.wgsl` — the same steps as WGSL compute. They run on the GPU through
  Horizon's own wgpu. All validate with `naga` (see `shadercheck/`).

Because the GPU shaders mirror the tested CPU solver step for step, the simulation is
sound. What a GPU is still needed for is to *run* and *draw* it at speed.

## The simulation step (one frame)

1. `clear_atomic` — zero the transfer buffers.
2. `p2g` — splat particle velocity onto the MAC faces (fixed-point atomics).
3. `normalize` — divide accumulated value by weight to get face velocity.
4. `copy` — save the old velocity field (FLIP needs the change, not the value).
5. `gravity` — pull the vertical faces down.
6. `count` + `mark` — mark each cell solid / fluid / air.
7. `solidfaces` — zero velocity into solids (no-slip normal).
8. `divergence` — per fluid cell.
9. `jacobi` ×N — solve the pressure Poisson equation (ping-pong buffers).
10. `gradient` — subtract the pressure gradient; now the field is divergence-free,
    so **volume is held**.
11. `extrapolate` — give air faces a nearby velocity.
12. `g2p_advect` — blend grid velocity back to particles (FLIP ratio ~0.95) and move
    them (RK2), clamped to the domain and pushed out of solids.

## Drawing (screen-space fluid)

1. `water_depth` — billboard each particle as a sphere imposter; write smoothed depth and
   additive thickness.
2. `water_blur` — narrow-range filter: smooth the surface, keep the silhouette.
3. `water_composite` — rebuild the normal from depth, refract the scene behind, darken by
   thickness (Beer-Lambert), add a fresnel sky reflection and a sun highlight.
4. `grain.*` — sand drawn as lit grains.

## Wiring into Horizon

A native mod injects a HEAD hook into `native_world_drawn` (render thread, gives the wgpu
`device`, `queue`, `encoder`, the frame colour target and the world depth). It asks for the
depth with `native_render_needs`. The server side (the `bundle/` native mod) tells the
client where water, sand and cut trees are; the client runs the sim regions and draws them.

## Not done yet

The Rust that drives these pipelines against Horizon's frame is **not written**, and nothing
has run on a GPU or in the game. The pieces that ARE proven: the solver (CPU tests) and that
every shader compiles (`naga`). Finishing needs a real GPU to run and tune.
