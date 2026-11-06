# Real Physics for Horizon

Realistic water, falling trees and sliding sand for **Horizon** (Minecraft: Java Edition 26.3,
rewritten in Rust).

All code here is original. Nothing is copied from any other mod.

## Two parts

**Real Water** (`real_water-<version>.hshader`) — a Horizon shader pack:

- rolling waves that really move the water surface
- real reflections of the world, Fresnel-correct
- refraction: see the bottom, bent by the waves
- deep water turns blue with depth (red light goes first)
- foaming waterfalls, white water on fast flows, foam at the shore
- sun glint and a moon path, rain ripples
- sun shadows, and rippling caustic light on the ground under water
- underwater fog and wobble
- options for all of it (Shader Options...)

**Real Physics** (`realphysics-<version>.hnmod`) — a native mod:

- **Falling trees** — cut the last grounded log and the tree topples away from you.
- **Sliding sand** — sand and gravel slump to about 45°.
- **Volume water** — a bucket spreads thin, falls, fills holes and levels out.

## Install

Get both files from the [Releases](../../releases) page.

- **Water look:** drag `real_water-<version>.hshader` onto the Horizon launcher. In the game:
  Options > Video Settings > Shaders > Shader Packs > **Real Water (Horizon)**.
- **Trees, sand, water physics:** in the launcher, Mods page > **Add from GitHub**, paste
  `https://github.com/NeoDuude/Realistic-water-for-rust-mc`, then **Build**. The first native
  build downloads Rust and takes a while.

The shader pack works on its own. The native mod is preview: not yet run in a live game.

## License

MIT. See `LICENSE`.
