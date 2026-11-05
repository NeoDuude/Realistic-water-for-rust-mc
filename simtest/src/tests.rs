use crate::sim::P;
use crate::sim::sand::{Sand, Spot};
use crate::sim::tree::{self, Part};
use crate::sim::water::{Cell, Grid, Water};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct Box3 {
    water: HashMap<P, u8>,
    solid: HashSet<P>,
    writes: usize,
}
impl Grid for Box3 {
    fn read(&mut self, p: P) -> Cell {
        if self.solid.contains(&p) || p[1] < 0 {
            Cell::Solid
        } else if let Some(u) = self.water.get(&p) {
            Cell::Water(*u)
        } else {
            Cell::Open
        }
    }
    fn write(&mut self, p: P, c: Cell) -> bool {
        self.writes += 1;
        match c {
            Cell::Water(u) => {
                self.water.insert(p, u);
            }
            _ => {
                self.water.remove(&p);
            }
        }
        true
    }
}
impl Box3 {
    fn total(&self) -> u32 {
        self.water.values().map(|u| *u as u32).sum()
    }
    /// Stone basin, inside x,z in 0..n, walls `h` high, floor y=0.
    fn basin(n: i32, h: i32) -> Self {
        let mut b = Self::default();
        for x in -1..=n {
            for z in -1..=n {
                b.solid.insert([x, 0, z]);
                if x < 0 || z < 0 || x == n || z == n {
                    for y in 1..=h {
                        b.solid.insert([x, y, z]);
                    }
                }
            }
        }
        b
    }
}
fn run(w: &mut Water, b: &mut Box3, ticks: usize) -> usize {
    for t in 0..ticks {
        w.step(b, 100_000, || true);
        if w.awake() == 0 {
            return t;
        }
    }
    panic!("did not settle in {ticks} ticks, awake {}", w.awake());
}

#[test]
fn bucket_keeps_its_volume_and_spreads_thin() {
    let mut b = Box3::basin(9, 3);
    b.water.insert([4, 1, 4], 8);
    let mut w = Water::default();
    w.wake([4, 1, 4]);
    run(&mut w, &mut b, 2000);
    assert_eq!(b.total(), 8);
    assert!(b.water.values().all(|u| *u <= 2), "{:?}", b.water);
    assert!(b.water.len() >= 4);
}

#[test]
fn water_falls_and_stacks_without_loss() {
    let mut b = Box3::basin(1, 12);
    for y in 6..=9 {
        b.water.insert([0, y, 0], 8);
    }
    let mut w = Water::default();
    for y in 6..=9 {
        w.wake([0, y, 0]);
    }
    run(&mut w, &mut b, 2000);
    assert_eq!(b.total(), 32);
    for y in 1..=4 {
        assert_eq!(b.water.get(&[0, y, 0]), Some(&8), "y={y}");
    }
}

#[test]
fn big_pool_settles_level() {
    let mut b = Box3::basin(12, 6);
    for x in 0..4 {
        for z in 0..12 {
            for y in 1..=4 {
                b.water.insert([x, y, z], 8);
            }
        }
    }
    let start = b.total();
    let mut w = Water::default();
    let keys: Vec<P> = b.water.keys().copied().collect();
    for k in keys {
        w.wake(k);
    }
    run(&mut w, &mut b, 20000);
    // Under the reservoir size, so it is a real pond and keeps its volume.
    assert!(b.total() >= start, "lost water {} < {start}", b.total());
    let tops: Vec<u8> = (0..12)
        .flat_map(|x| (0..12).map(move |z| (x, z)))
        .map(|(x, z)| *b.water.get(&[x, 1, z]).unwrap_or(&0))
        .collect();
    let (lo, hi) = (*tops.iter().min().unwrap(), *tops.iter().max().unwrap());
    assert!(hi - lo <= 1, "floor units {lo}..{hi}");
}

#[test]
fn sea_stays_level_when_a_canal_opens() {
    let mut b = Box3::basin(40, 3);
    for x in 0..30 {
        for z in 0..40 {
            b.water.insert([x, 1, z], 8);
        }
    }
    let mut w = Water::default();
    for z in 0..40 {
        w.wake([29, 1, z]);
    }
    run(&mut w, &mut b, 20000);
    assert_eq!(b.water.get(&[0, 1, 0]), Some(&8));
    let far = b.water.get(&[39, 1, 20]).copied().unwrap_or(0);
    assert!(far >= 7, "canal filled to {far}");
}

#[test]
fn unknown_chunk_is_a_wall() {
    struct Edge(Box3);
    impl Grid for Edge {
        fn read(&mut self, p: P) -> Cell {
            if p[0] > 2 { Cell::Unknown } else { self.0.read(p) }
        }
        fn write(&mut self, p: P, c: Cell) -> bool {
            assert!(p[0] <= 2);
            self.0.write(p, c)
        }
    }
    let mut e = Edge(Box3::basin(9, 3));
    e.0.water.insert([2, 1, 4], 8);
    let mut w = Water::default();
    w.wake([2, 1, 4]);
    for _ in 0..500 {
        w.step(&mut e, 1000, || true);
    }
    assert_eq!(e.0.total(), 8);
}

#[test]
fn budget_is_kept_and_nothing_is_lost_when_out_of_time() {
    let mut b = Box3::basin(30, 3);
    for x in 0..30 {
        for z in 0..3 {
            b.water.insert([x, 1, z], 8);
        }
    }
    let start = b.total();
    let mut w = Water::default();
    let keys: Vec<P> = b.water.keys().copied().collect();
    for k in keys {
        w.wake(k);
    }
    let mut n = 0;
    for _ in 0..50 {
        n = 0;
        let r = w.step(&mut b, 40, || {
            n += 1;
            n < 25
        });
        assert!(r.cells <= 40);
    }
    assert_eq!(b.total(), start);
    let _ = n;
}

#[test]
fn settled_water_goes_to_sleep() {
    let mut b = Box3::basin(5, 3);
    b.water.insert([2, 1, 2], 8);
    let mut w = Water::default();
    w.wake([2, 1, 2]);
    run(&mut w, &mut b, 2000);
    let before = b.writes;
    for _ in 0..100 {
        w.step(&mut b, 1000, || true);
    }
    assert_eq!(b.writes, before);
}

/// Plays slides out with instant gravity.
fn settle_sand(grains: &mut HashSet<P>, floor: i32) {
    let mut s = Sand::default();
    for g in grains.iter() {
        s.wake(*g);
    }
    for _ in 0..200_000 {
        if s.awake() == 0 {
            return;
        }
        let snapshot = grains.clone();
        let mut read = |p: P| {
            if snapshot.contains(&p) {
                Spot::Grain
            } else if p[1] <= floor {
                Spot::Solid
            } else {
                Spot::Open
            }
        };
        for m in s.step(&mut read, 64, 4096) {
            if !grains.remove(&m.from) {
                continue; // Moved by an earlier slide this tick.
            }
            // The game's gravity: what stood on it drops one.
            let mut y = m.from[1] + 1;
            while grains.remove(&[m.from[0], y, m.from[2]]) {
                grains.insert([m.from[0], y - 1, m.from[2]]);
                s.wake_around([m.from[0], y - 1, m.from[2]]);
                y += 1;
            }
            let mut y = m.to[1];
            while y - 1 > floor && !grains.contains(&[m.to[0], y - 1, m.to[2]]) {
                y -= 1;
            }
            while grains.contains(&[m.to[0], y, m.to[2]]) {
                y += 1; // Another slide got there first this tick.
            }
            grains.insert([m.to[0], y, m.to[2]]);
            s.wake_around(m.from);
            s.wake_around([m.to[0], y, m.to[2]]);
        }
    }
    panic!("sand never settled");
}

#[test]
fn tower_of_sand_slumps_to_a_cone() {
    let mut grains: HashSet<P> = (1..=20).map(|y| [0, y, 0]).collect();
    settle_sand(&mut grains, 0);
    assert_eq!(grains.len(), 20);
    let top = grains.iter().map(|g| g[1]).max().unwrap();
    assert!(top <= 5, "top {top}");
    // No step of 2 between neighbouring columns.
    let height = |x, z| grains.iter().filter(|g| g[0] == x && g[2] == z).map(|g| g[1]).max().unwrap_or(0);
    for g in &grains {
        for d in crate::sim::SIDES {
            assert!(height(g[0], g[2]) - height(g[0] + d[0], g[2] + d[2]) <= 1);
        }
    }
}

#[test]
fn wet_sand_holds_a_steeper_face() {
    let mut read = |p: P| match p {
        [0, 1..=2, 0] => Spot::Grain,
        [1, 2, 0] => Spot::Water,
        [_, y, _] if y <= 0 => Spot::Solid,
        _ => Spot::Open,
    };
    let mut s = Sand::default();
    s.wake([0, 2, 0]);
    // The west, north and south faces drop only 2, so wet sand needs 3 and stays.
    assert!(s.step(&mut read, 8, 100).is_empty());
}

struct Oak {
    logs: HashSet<P>,
    leaves: HashSet<P>,
}
impl Oak {
    fn new(height: i32) -> Self {
        let logs = (1..=height).map(|y| [0, y, 0]).collect();
        let mut leaves = HashSet::new();
        for y in height - 1..=height + 1 {
            for x in -2i32..=2 {
                for z in -2i32..=2 {
                    if (x, z) != (0, 0) && x.abs() + z.abs() <= 3 {
                        leaves.insert([x, y, z]);
                    }
                }
            }
        }
        leaves.insert([0, height + 1, 0]);
        Oak { logs, leaves }
    }
    fn read(&self, p: P) -> Part {
        if self.logs.contains(&p) {
            Part::Log
        } else if self.leaves.contains(&p) {
            Part::NaturalLeaf
        } else if p[1] <= 0 {
            Part::Ground
        } else {
            Part::Other
        }
    }
}

/// The game's falling block: gravity, move, drag.
fn landing(p: P, v0: [f64; 3], floor: f64) -> [f64; 2] {
    let (mut x, mut y, mut z) = (p[0] as f64, p[1] as f64, p[2] as f64);
    let mut v = v0;
    for _ in 0..600 {
        v[1] -= 0.04;
        x += v[0];
        y += v[1];
        z += v[2];
        v = [v[0] * 0.98, v[1] * 0.98, v[2] * 0.98];
        if y <= floor {
            return [x, z];
        }
    }
    panic!("never landed");
}

#[test]
fn oak_falls_away_from_the_player_along_a_line() {
    let mut oak = Oak::new(7);
    oak.logs.remove(&[0, 1, 0]);
    let dir = tree::fall_dir([0, 1, 0], [-3.0, 1.0, 0.5]);
    assert_eq!(dir, [1, 0]);
    let fall = tree::plan([0, 1, 0], dir, &mut |p| oak.read(p)).expect("falls");
    assert_eq!(fall.logs, 6);
    assert!(fall.leaves > 10);
    for piece in fall.pieces.iter().filter(|p| p.log) {
        let h = piece.pos[1] - 1;
        let [x, z] = landing(piece.pos, piece.velocity, 1.0);
        assert!((x - h as f64).abs() < 1.5, "log h={h} landed x={x}");
        assert!(z.abs() < 0.01);
    }
}

#[test]
fn log_wall_and_two_trunks_do_not_fall() {
    // Cabin wall: logs, no leaves.
    let wall = Oak { logs: (1..=6).map(|y| [0, y, 0]).collect(), leaves: HashSet::new() };
    assert!(tree::plan([0, 1, 0], [1, 0], &mut |p| if p == [0, 1, 0] { Part::Other } else { wall.read(p) }).is_none());
    // Wide trunk: another log still on the ground.
    let mut oak = Oak::new(7);
    oak.logs.remove(&[0, 1, 0]);
    oak.logs.insert([1, 1, 0]);
    oak.logs.insert([1, 2, 0]);
    assert!(tree::plan([0, 1, 0], [1, 0], &mut |p| oak.read(p)).is_none());
}

/// Whole-engine run through the public hooks, with an in-memory world.
#[derive(Default)]
struct Fake {
    water: HashMap<P, u8>,
    grains: HashSet<P>,
    logs: HashSet<P>,
    leaves: HashSet<P>,
    thrown: Vec<(P, [f64; 3], bool, bool)>,
    slid: Vec<(P, P)>,
    sounds: usize,
}
impl realphysics_engine::World for Fake {
    fn water_cell(&mut self, p: P) -> Cell {
        if p[1] <= 0 {
            Cell::Solid
        } else if let Some(u) = self.water.get(&p) {
            Cell::Water(*u)
        } else {
            Cell::Open
        }
    }
    fn set_water(&mut self, p: P, c: Cell) -> bool {
        match c {
            Cell::Water(u) => {
                self.water.insert(p, u);
            }
            _ => {
                self.water.remove(&p);
            }
        }
        // As the game does: our own write fires the change hook.
        realphysics_engine::block_changed("test:water", p, true);
        true
    }
    fn sand_spot(&mut self, p: P) -> Spot {
        if self.grains.contains(&p) {
            Spot::Grain
        } else if p[1] <= 0 {
            Spot::Solid
        } else {
            Spot::Open
        }
    }
    fn slide_grain(&mut self, from: P, to: P) {
        self.grains.remove(&from);
        self.slid.push((from, to));
    }
    fn tree_part(&mut self, p: P) -> Part {
        if self.logs.contains(&p) {
            Part::Log
        } else if self.leaves.contains(&p) {
            Part::NaturalLeaf
        } else if p[1] <= 0 {
            Part::Ground
        } else {
            Part::Other
        }
    }
    fn lift_piece(&mut self, p: P) {
        self.logs.remove(&p);
        self.leaves.remove(&p);
    }
    fn throw_piece(&mut self, p: P, v: [f64; 3], log: bool, x: bool) {
        self.thrown.push((p, v, log, x));
    }
    fn tree_sound(&mut self, _: P) {
        self.sounds += 1;
    }
}

#[test]
fn engine_runs_water_sand_and_trees_through_its_hooks() {
    let dim = "minecraft:the_end";
    realphysics_engine::reset();
    let mut w = Fake::default();
    // A bucket of water.
    w.water.insert([0, 3, 0], 8);
    realphysics_engine::block_changed(dim, [0, 3, 0], true);
    // A sand spike.
    for y in 1..=4 {
        w.grains.insert([10, y, 0]);
    }
    realphysics_engine::block_changed(dim, [10, 4, 0], false);
    // A tree with its stump cut.
    let oak = Oak::new(6);
    w.logs = oak.logs.clone();
    w.leaves = oak.leaves.clone();
    w.logs.remove(&[20, 1, 0]);
    w.logs.remove(&[0, 1, 0]);
    realphysics_engine::log_cut(dim, [0, 1, 0], Some([-3.0, 1.0, 0.5]));
    for _ in 0..400 {
        realphysics_engine::tick(dim, &mut w);
    }
    assert_eq!(w.water.values().map(|u| u32::from(*u)).sum::<u32>(), 8);
    assert!(w.water.keys().all(|p| p[1] == 1), "{:?}", w.water);
    assert!(!w.slid.is_empty(), "sand slid");
    assert_eq!(w.sounds, 1);
    assert!(w.logs.is_empty() && w.leaves.is_empty());
    assert!(w.thrown.iter().filter(|t| t.2).count() == 5);
    assert!(w.thrown.iter().all(|t| !t.2 || t.3), "logs lie along x");
    realphysics_engine::reset();
}

#[test]
fn speed_report() {
    let mut b = Box3::basin(64, 12);
    for x in 0..16 {
        for z in 0..64 {
            for y in 1..=8 {
                b.water.insert([x, y, z], 7);
            }
        }
    }
    let mut w = Water::default();
    let keys: Vec<P> = b.water.keys().copied().collect();
    for k in keys {
        w.wake(k);
    }
    let start = std::time::Instant::now();
    let (mut cells, mut worst) = (0usize, std::time::Duration::ZERO);
    for _ in 0..200 {
        let t = std::time::Instant::now();
        let end = t + realphysics_engine::WATER_TIME_PER_TICK;
        cells += w.step(&mut b, realphysics_engine::WATER_CELLS_PER_TICK, || std::time::Instant::now() < end).cells;
        worst = worst.max(t.elapsed());
    }
    println!("SPEED cells={cells} total={:?} worst_tick={worst:?}", start.elapsed());
}
