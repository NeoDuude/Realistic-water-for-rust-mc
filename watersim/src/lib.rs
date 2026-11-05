//! Real Physics water: a FLIP/PIC fluid on a MAC grid.
//!
//! This is my own code, from the standard method (Zhu & Bridson, "Animating Sand as a Fluid",
//! and Bridson's "Fluid Simulation for Computer Graphics"). None of it is copied from any mod.
//!
//! The GPU side (WGSL in `bundle/render/wgsl`) runs the same steps. This CPU version is the
//! reference the shaders follow and the one the tests check, so the method is proven even where
//! no GPU runs.
//!
//! A MAC grid holds velocity on cell faces: `u` on the x faces, `v` on the y faces, `w` on the
//! z faces. Pressure sits at cell centres. Particles carry the fluid; the grid moves it without
//! losing volume, because the pressure solve makes the velocity field divergence free.

pub mod grid;
pub mod solver;

pub use grid::{Cell, Mac};
pub use solver::{Fluid, Params, Particle};

#[cfg(test)]
mod tests;
