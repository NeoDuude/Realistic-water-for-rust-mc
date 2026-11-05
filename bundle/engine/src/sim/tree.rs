//! Felling. Cut the last grounded log and the tree tips over away from you. Each piece is
//! thrown so it lands where a stiff tree turning on its stump would lay it.
use super::{P, above, add, below};
use std::collections::HashSet;

pub const MAX_LOGS: usize = 128;
pub const MAX_LEAVES: usize = 400;
const MIN_NATURAL_LEAVES: usize = 4;
const LEAF_REACH: u8 = 4;
const GRAVITY: f64 = 0.04;
const DRAG: f64 = 0.98;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Part {
    Log,
    /// Grown on the tree, not placed by a player.
    NaturalLeaf,
    PlacedLeaf,
    /// Ground a trunk can stand on.
    Ground,
    Other,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub pos: P,
    pub log: bool,
    pub velocity: [f64; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fall {
    pub dir: [i32; 2],
    pub pieces: Vec<Piece>,
    pub logs: usize,
    pub leaves: usize,
}

/// Which way it goes: straight away from the player, on the stronger axis.
pub fn fall_dir(cut: P, player: [f64; 3]) -> [i32; 2] {
    let dx = cut[0] as f64 + 0.5 - player[0];
    let dz = cut[2] as f64 + 0.5 - player[2];
    if dx.abs() >= dz.abs() {
        [if dx >= 0.0 { 1 } else { -1 }, 0]
    } else {
        [0, if dz >= 0.0 { 1 } else { -1 }]
    }
}

/// `cut` is the block just removed. None when no whole natural tree hangs over it: a log
/// wall, a house, or a tree still standing on other logs.
pub fn plan(cut: P, dir: [i32; 2], read: &mut impl FnMut(P) -> Part) -> Option<Fall> {
    let mut logs = Vec::new();
    let mut seen = HashSet::from([cut]);
    let mut todo = Vec::new();
    for dx in -1..=1 {
        for dz in -1..=1 {
            let n = [cut[0] + dx, cut[1] + 1, cut[2] + dz];
            if read(n) == Part::Log && seen.insert(n) {
                todo.push(n);
            }
        }
    }
    while let Some(p) = todo.pop() {
        logs.push(p);
        if logs.len() > MAX_LOGS {
            return None;
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                for dz in -1..=1 {
                    let n = [p[0] + dx, p[1] + dy, p[2] + dz];
                    if !seen.contains(&n) && read(n) == Part::Log {
                        seen.insert(n);
                        todo.push(n);
                    }
                }
            }
        }
    }
    if logs.len() < 2 {
        return None;
    }
    // Still held up by another trunk?
    if logs.iter().any(|p| read(below(*p)) == Part::Ground) {
        return None;
    }
    let (mut leaves, mut natural) = (Vec::new(), 0usize);
    let mut leaf_seen = HashSet::new();
    let mut ring: Vec<P> = logs.clone();
    for _ in 0..LEAF_REACH {
        let mut next = Vec::new();
        for p in &ring {
            for d in [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]] {
                let n = add(*p, d);
                if leaf_seen.contains(&n) || leaves.len() >= MAX_LEAVES {
                    continue;
                }
                if read(n) == Part::NaturalLeaf {
                    leaf_seen.insert(n);
                    natural += 1;
                    leaves.push(n);
                    next.push(n);
                }
            }
        }
        ring = next;
    }
    if natural < MIN_NATURAL_LEAVES {
        return None;
    }
    let mut pieces = Vec::with_capacity(logs.len() + leaves.len());
    logs.sort_by_key(|p| p[1]);
    for &p in &logs {
        pieces.push(Piece { pos: p, log: true, velocity: throw(cut, p, dir, read) });
    }
    for &p in &leaves {
        pieces.push(Piece { pos: p, log: false, velocity: throw(cut, p, dir, read) });
    }
    Some(Fall { dir, pieces, logs: logs.len(), leaves: leaves.len() })
}

/// Turn the point 90 degrees about the stump: height becomes distance along the fall, and
/// its offset along the fall becomes height. Solve the throw with the game's own drag.
fn throw(cut: P, p: P, dir: [i32; 2], _read: &mut impl FnMut(P) -> Part) -> [f64; 3] {
    let h = (p[1] - cut[1]) as f64;
    let along_now = ((p[0] - cut[0]) * dir[0] + (p[2] - cut[2]) * dir[1]) as f64;
    let up_after = along_now.max(0.0);
    let t = ticks_to_drop((h - up_after).max(1.0));
    let along_move = h - along_now;
    let v = along_move * (1.0 - DRAG) / (1.0 - DRAG.powi(t));
    [v * dir[0] as f64, 0.0, v * dir[1] as f64]
}

/// Ticks for a falling block let go at rest to drop `height`.
pub fn ticks_to_drop(height: f64) -> i32 {
    let (mut y, mut vy, mut t) = (0.0, 0.0, 0);
    while y < height && t < 400 {
        vy = (vy - GRAVITY) * 1.0;
        y -= vy;
        vy *= DRAG;
        t += 1;
    }
    t.max(1)
}
