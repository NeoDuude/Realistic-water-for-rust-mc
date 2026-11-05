//! Game-free physics. No Horizon types here, so `simtest` runs it with plain `cargo test`.
pub mod fast;
pub mod sand;
pub mod tree;
pub mod water;

pub type P = [i32; 3];
pub const SIDES: [P; 4] = [[1, 0, 0], [-1, 0, 0], [0, 0, 1], [0, 0, -1]];

#[inline]
pub fn add(a: P, b: P) -> P {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
#[inline]
pub fn below(p: P) -> P {
    [p[0], p[1] - 1, p[2]]
}
#[inline]
pub fn above(p: P) -> P {
    [p[0], p[1] + 1, p[2]]
}
/// Small position hash, so choices vary by place without a random stream.
#[inline]
pub fn mix(p: P, t: u64) -> u32 {
    let h = (p[0] as u32).wrapping_mul(73_856_093)
        ^ (p[1] as u32).wrapping_mul(19_349_663)
        ^ (p[2] as u32).wrapping_mul(83_492_791)
        ^ (t as u32).wrapping_mul(2_654_435_761);
    h ^ (h >> 13)
}
