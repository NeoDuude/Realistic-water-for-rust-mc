//! Sand that slides. A grain with a drop of 2 or more beside it slips sideways, so piles
//! settle near 45 degrees. Wet sand holds a steeper face. The game's own gravity does the
//! rest: this only hands over a falling block.
use super::{P, SIDES, above, add, below, mix};
use std::collections::{HashSet, VecDeque};

const DRY_DROP: i32 = 2;
const WET_DROP: i32 = 3;
const MAX_QUEUE: usize = 8192;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Spot {
    Grain,
    Open,
    Water,
    Solid,
}

#[derive(Default)]
pub struct Sand {
    queue: VecDeque<P>,
    queued: HashSet<P>,
    tick: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Slide {
    pub from: P,
    pub to: P,
}

impl Sand {
    pub fn wake(&mut self, p: P) {
        if self.queue.len() < MAX_QUEUE && self.queued.insert(p) {
            self.queue.push_back(p);
        }
    }
    /// What may slip after a block changed at `p`: the ring around it and the ring above.
    pub fn wake_around(&mut self, p: P) {
        for dy in 0..=1 {
            for dx in -1..=1 {
                for dz in -1..=1 {
                    self.wake([p[0] + dx, p[1] + dy, p[2] + dz]);
                }
            }
        }
    }
    pub fn awake(&self) -> usize {
        self.queue.len()
    }
    /// Slides for this tick, at most `max`. The caller moves the blocks.
    pub fn step(&mut self, read: &mut impl FnMut(P) -> Spot, max: usize, max_checks: usize) -> Vec<Slide> {
        self.tick += 1;
        let mut out = Vec::new();
        for _ in 0..max_checks {
            if out.len() >= max {
                break;
            }
            let Some(p) = self.queue.pop_front() else { break };
            self.queued.remove(&p);
            if let Some(to) = slip(p, self.tick, read) {
                out.push(Slide { from: p, to });
                self.wake(above(p));
            }
        }
        out
    }
}

fn slip(p: P, tick: u64, read: &mut impl FnMut(P) -> Spot) -> Option<P> {
    if read(p) != Spot::Grain || matches!(read(below(p)), Spot::Open | Spot::Water) {
        return None; // Not sand, or already falling by the game's own rules.
    }
    let wet = read(above(p)) == Spot::Water
        || SIDES.iter().any(|d| read(add(p, *d)) == Spot::Water);
    let need = if wet { WET_DROP } else { DRY_DROP };
    let spin = mix(p, tick) as usize % 4;
    for i in 0..4 {
        let n = add(p, SIDES[(i + spin) % 4]);
        if !matches!(read(n), Spot::Open | Spot::Water) {
            continue;
        }
        let mut drop = 0;
        let mut at = n;
        while drop < need && matches!(read(at), Spot::Open | Spot::Water) {
            drop += 1;
            at = below(at);
        }
        if drop >= need {
            return Some(n);
        }
    }
    None
}
