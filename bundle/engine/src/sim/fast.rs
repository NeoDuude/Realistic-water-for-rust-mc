//! A quick hash for block positions. The standard one guards against attacks these keys
//! can't make, and costs a third of the water's time.
use super::P;
use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default, Clone, Copy)]
pub struct Quick(u64);
impl Hasher for Quick {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.write_u8(*b);
        }
    }
    fn write_u8(&mut self, n: u8) {
        self.write_u32(u32::from(n));
    }
    fn write_i32(&mut self, n: i32) {
        self.write_u32(n as u32);
    }
    fn write_u32(&mut self, n: u32) {
        self.0 = (self.0.rotate_left(5) ^ u64::from(n)).wrapping_mul(0x517c_c1b7_2722_0a95);
    }
    fn write_usize(&mut self, n: usize) {
        self.write_u32(n as u32);
    }
}
pub type Build = BuildHasherDefault<Quick>;
pub type HashMap<K, V> = std::collections::HashMap<K, V, Build>;
pub type HashSet<K> = std::collections::HashSet<K, Build>;

pub trait HashMapExt {
    fn new_fast() -> Self;
}
impl<K, V> HashMapExt for HashMap<K, V> {
    fn new_fast() -> Self {
        Self::default()
    }
}
pub fn set_of(p: P) -> HashSet<P> {
    let mut s = HashSet::default();
    s.insert(p);
    s
}
