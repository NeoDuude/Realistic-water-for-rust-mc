# Real Physics

Native mod for Horizon (Minecraft 26.3 in Rust). Server side, single player included.

- **Water** holds its volume. A bucket spreads thin, falls, fills holes, levels out. Seas (256+ joined full
  blocks) refill, so an ocean stays level. Lava is vanilla.
- **Sand** and gravel slide when a step is 2 high, so piles settle near 45 degrees. Wet sand holds a step of 2.
- **Trees** fall away from you when you cut the last grounded log, and land as a felled trunk. Logs hurt what
  they hit. Log walls, and trees with no natural leaves, stay up.

Limits per tick are constants at the top of `engine/src/lib.rs`.

## Install

    horizon-launcher native-build <this folder> --instance <instance> --trust-native-code

Pinned to Horizon commit f2f95dad8a1a701424a08bd989168297a7172b36. Native mods run with your full user rights.

## Tests

    cd simtest && cargo test --release
