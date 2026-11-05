# Real Physics for Horizon

Native mod for **Horizon** (Minecraft: Java Edition 26.3, rewritten in Rust).
Pinned to Horizon commit `f2f95dad8a1a701424a08bd989168297a7172b36`.

All code here is original. Nothing is copied from any other mod.

## What it does

- **Falling trees** — cut the last grounded log and the tree topples away from you,
  landing as a felled trunk. Logs hurt what they land on. Log walls stay up.
- **Sliding sand** — sand and gravel slump to about 45°. Wet sand holds a steeper face.
- **Volume water (blocks)** — water that keeps its volume: a bucket spreads thin, falls,
  fills holes and levels out. Seas stay level.
- **FLIP water (GPU sim)** — a MAC-grid FLIP/PIC fluid for smooth, particle-based water.

## Status — read before you expect too much

| Part | State |
|---|---|
| Falling trees | built, compiles against Horizon, CPU-tested |
| Sliding sand | built, compiles, CPU-tested |
| Block water | built, compiles, CPU-tested |
| FLIP solver core (`watersim/`) | **done, CPU-tested** (volume, settling, flow, no clumping) |
| FLIP GPU shaders (`shaders/`) | **done, all 20 validate with `naga`** |
| GPU driver Rust + in-game rendering | **not written** — no on-screen water yet |

No machine here has a GPU, and no Minecraft 26.3 copy, so **nothing has run inside the
game**. First real launch will find bugs. The FLIP GPU water is a work in progress: the
simulation method is proven on CPU, but the WGSL shaders and the screen-space rendering
(refraction, absorption, foam) are not written yet.

## Layout

- `bundle/` — the native mod (trees, sand, block water). Build target.
- `watersim/` — the FLIP/PIC fluid: CPU reference + tests.
- `shaders/` — the GPU sim and draw shaders (WGSL). See `ARCHITECTURE.md`.
- `shadercheck/` — validates every shader with `naga`.
- `simtest/` — runs the block physics with no game.

## Build the mod

    horizon-launcher native-build bundle --instance <instance> --trust-native-code

Native mods run with your full user rights.

## Tests

    cd simtest     && cargo test --release
    cd watersim    && cargo test --release
    cd shadercheck && cargo run        # validate all WGSL
