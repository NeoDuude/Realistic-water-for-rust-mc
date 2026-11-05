//! Water that keeps its volume. A block holds 1..=8 units: 8 is a source block, n is
//! flowing `level = 8 - n`, so a bucket spreads thin and runs out instead of copying itself.
//! Only cells near a change are awake. Settled water costs nothing.
use super::{P, SIDES, above, add, below, mix};
use super::fast::{HashMap, HashMapExt, HashSet, set_of};
use std::collections::VecDeque;

pub const FULL: u8 = 8;
const RESERVOIR_CELLS: usize = 256;
const RESERVOIR_TTL: u64 = 200;
const FILM_REACH: i32 = 5;
const FAR_CELLS: usize = 256;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    /// Air, or a plant water washes away.
    Open,
    Water(u8),
    Solid,
    /// Chunk not loaded: treated as a wall, so no water is lost into it.
    Unknown,
}

pub trait Grid {
    fn read(&mut self, p: P) -> Cell;
    /// `Cell::Open` or `Cell::Water`. Answers false when the game refused.
    fn write(&mut self, p: P, cell: Cell) -> bool;
}

#[derive(Default)]
pub struct Water {
    awake: VecDeque<P>,
    queued: HashSet<P>,
    reservoirs: HashMap<P, (u64, bool)>,
    tick: u64,
}

struct Pass<'a, G: Grid> {
    grid: &'a mut G,
    seen: HashMap<P, Cell>,
    first: HashMap<P, Cell>,
}

impl<G: Grid> Pass<'_, G> {
    fn get(&mut self, p: P) -> Cell {
        if let Some(c) = self.seen.get(&p) {
            return *c;
        }
        let c = self.grid.read(p);
        self.seen.insert(p, c);
        c
    }
    fn set(&mut self, p: P, c: Cell) {
        let old = self.get(p);
        self.first.entry(p).or_insert(old);
        self.seen.insert(p, c);
    }
    fn units(&mut self, p: P) -> u8 {
        match self.get(p) {
            Cell::Water(u) => u,
            _ => 0,
        }
    }
    fn put_units(&mut self, p: P, u: u8) {
        self.set(p, if u == 0 { Cell::Open } else { Cell::Water(u.min(FULL)) });
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Report {
    pub cells: usize,
    pub writes: usize,
    pub awake: usize,
}

impl Water {
    pub fn wake(&mut self, p: P) {
        if self.queued.insert(p) {
            self.awake.push_back(p);
        }
    }
    pub fn wake_around(&mut self, p: P) {
        self.wake(p);
        self.wake(above(p));
        self.wake(below(p));
        for d in SIDES {
            self.wake(add(p, d));
        }
    }
    pub fn awake(&self) -> usize {
        self.awake.len()
    }

    /// One game tick. `keep_going` is asked between cells: return false when out of time.
    pub fn step<G: Grid>(
        &mut self,
        grid: &mut G,
        max_cells: usize,
        mut keep_going: impl FnMut() -> bool,
    ) -> Report {
        self.tick += 1;
        if self.tick % RESERVOIR_TTL == 0 {
            let now = self.tick;
            self.reservoirs.retain(|_, (at, _)| now - *at < RESERVOIR_TTL);
        }
        let mut pass = Pass { grid, seen: HashMap::default(), first: HashMap::default() };
        let mut wakes = Vec::new();
        let mut cells = 0;
        // Only what was awake at the start, so a flood moves at a steady speed.
        let batch = self.awake.len().min(max_cells);
        while cells < batch && keep_going() {
            let Some(p) = self.awake.pop_front() else { break };
            self.queued.remove(&p);
            cells += 1;
            self.cell(&mut pass, p, &mut wakes);
        }
        // Volume moved in memory; write once, whatever number of moves touched a block.
        let mut writes = 0;
        let mut changes: Vec<(P, Cell, Cell)> = Vec::with_capacity(pass.first.len());
        for (p, old) in &pass.first {
            changes.push((*p, *old, pass.seen[p]));
        }
        changes.sort_by_key(|(p, _, _)| *p);
        for (p, old, new) in changes {
            if old != new {
                if pass.grid.write(p, new) {
                    writes += 1;
                }
            }
        }
        for w in wakes {
            self.wake_around(w);
        }
        Report { cells, writes, awake: self.awake.len() }
    }

    fn cell<G: Grid>(&mut self, pass: &mut Pass<'_, G>, p: P, wakes: &mut Vec<P>) {
        let Cell::Water(mut u) = pass.get(p) else { return };
        let start = u;
        let wakes_before = wakes.len();
        let source = u == FULL && self.is_reservoir(pass, p);

        // Fall first.
        let down = below(p);
        let room = match pass.get(down) {
            Cell::Open => FULL,
            Cell::Water(v) => FULL - v,
            _ => 0,
        };
        if room > 0 {
            let moved = room.min(u);
            let v = pass.units(down);
            pass.put_units(down, v + moved);
            u -= moved;
            wakes.push(down);
        }

        // Then level out with lower neighbours. Rotate, so no side always wins.
        if u > 0 {
            let spin = mix(p, self.tick) as usize % 4;
            let mut lower = 0;
            for d in SIDES {
                let n = add(p, d);
                if matches!(pass.get(n), Cell::Open | Cell::Water(_)) && u >= pass.units(n) + 2 {
                    lower += 1;
                }
            }
            if lower > 0 {
                for i in 0..4 {
                    let n = add(p, SIDES[(i + spin) % 4]);
                    if !matches!(pass.get(n), Cell::Open | Cell::Water(_)) {
                        continue;
                    }
                    let t = pass.units(n);
                    if u < t + 2 {
                        continue;
                    }
                    let moved = ((u - t) / (lower + 1)).max(1);
                    pass.put_units(n, t + moved);
                    u -= moved;
                    wakes.push(n);
                }
            }
        }

        // Neighbours one unit lower look settled, but a long slope is not. Reach along the
        // body for a low spot, so a pool ends flat. Every move lowers the pool, so it ends.
        if u >= 2 && lower_none(pass, p, u) {
            if let Some(n) = self.far_low_spot(pass, p, u) {
                let t = pass.units(n);
                pass.put_units(n, t + 1);
                u -= 1;
                wakes.push(n);
            }
        }

        // And a low cell pulls from a higher one further along, so a canal fills from the sea.
        if u > 0 && u + 2 <= FULL && lower_none(pass, p, u) {
            if let Some(from) = self.far_high_spot(pass, p, u) {
                let v = pass.units(from);
                if !(v == FULL && self.is_reservoir(pass, from)) {
                    pass.put_units(from, v - 1);
                    wakes.push(from);
                } else {
                    wakes.push(from);
                }
                u += 1;
            }
        }

        // The last thin film runs to the nearest edge.
        if u == 1 && room == 0 {
            if let Some(n) = self.towards_drop(pass, p) {
                let t = pass.units(n);
                pass.put_units(n, t + 1);
                u = 0;
                wakes.push(n);
            }
        }

        if source {
            u = FULL; // A sea does not drop by a bucket's worth.
        }
        if u != start {
            pass.put_units(p, u);
            wakes.push(p);
        } else if source && wakes.len() > wakes_before {
            wakes.push(p);
        }
    }

    /// Nearest cell at this height, joined to `p` by water, holding 2 or more units less.
    fn far_low_spot<G: Grid>(&self, pass: &mut Pass<'_, G>, p: P, u: u8) -> Option<P> {
        let spin = mix(p, self.tick) as usize % 4;
        let mut seen = set_of(p);
        let mut todo = VecDeque::from([p]);
        while let Some(c) = todo.pop_front() {
            if seen.len() > FAR_CELLS {
                return None;
            }
            for i in 0..4 {
                let n = add(c, SIDES[(i + spin) % 4]);
                if !seen.insert(n) {
                    continue;
                }
                match pass.get(n) {
                    Cell::Water(v) if v + 2 <= u => return Some(n),
                    Cell::Water(_) => todo.push_back(n),
                    Cell::Open if u >= 2 && c != p => return Some(n),
                    _ => {}
                }
            }
        }
        None
    }

    /// Nearest cell at this height, joined to `p` by water, holding 2 or more units more.
    fn far_high_spot<G: Grid>(&self, pass: &mut Pass<'_, G>, p: P, u: u8) -> Option<P> {
        let mut seen = set_of(p);
        let mut todo = VecDeque::from([p]);
        while let Some(c) = todo.pop_front() {
            if seen.len() > FAR_CELLS {
                return None;
            }
            for d in SIDES {
                let n = add(c, d);
                if !seen.insert(n) {
                    continue;
                }
                if let Cell::Water(v) = pass.get(n) {
                    if v >= u + 2 {
                        return Some(n);
                    }
                    todo.push_back(n);
                }
            }
        }
        None
    }

    /// Next step along a straight line to a place water can fall, within reach.
    fn towards_drop<G: Grid>(&self, pass: &mut Pass<'_, G>, p: P) -> Option<P> {
        let spin = mix(p, 0) as usize % 4;
        let mut best: Option<(i32, P)> = None;
        for i in 0..4 {
            let d = SIDES[(i + spin) % 4];
            let mut at = p;
            for dist in 1..=FILM_REACH {
                at = add(at, d);
                if pass.get(at) != Cell::Open {
                    break;
                }
                let under = pass.get(below(at));
                let falls = under == Cell::Open || matches!(under, Cell::Water(v) if v < FULL);
                if falls {
                    if best.is_none_or(|(b, _)| dist < b) {
                        best = Some((dist, add(p, d)));
                    }
                    break;
                }
            }
        }
        best.map(|(_, n)| n)
    }

    /// True for a big body of full water: it gives without dropping, so a dug canal fills
    /// and the sea stays level. A pond under the limit really does drain.
    fn is_reservoir<G: Grid>(&mut self, pass: &mut Pass<'_, G>, p: P) -> bool {
        if let Some((at, answer)) = self.reservoirs.get(&p) {
            if self.tick - *at < RESERVOIR_TTL {
                return *answer;
            }
        }
        let mut seen = set_of(p);
        let mut todo = vec![p];
        while let Some(c) = todo.pop() {
            if seen.len() >= RESERVOIR_CELLS {
                break;
            }
            for d in SIDES {
                let n = add(c, d);
                if !seen.contains(&n) && pass.get(n) == Cell::Water(FULL) {
                    seen.insert(n);
                    todo.push(n);
                }
            }
        }
        let answer = seen.len() >= RESERVOIR_CELLS;
        let now = self.tick;
        for c in seen {
            self.reservoirs.insert(c, (now, answer));
        }
        answer
    }
}

fn lower_none<G: Grid>(pass: &mut Pass<'_, G>, p: P, u: u8) -> bool {
    SIDES.iter().all(|d| {
        let n = add(p, *d);
        !matches!(pass.get(n), Cell::Open | Cell::Water(_)) || u < pass.units(n) + 2
    })
}
