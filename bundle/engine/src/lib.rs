//! Real Physics: water that keeps its volume, sand that slides, trees that fall.
//! All game access goes through `World`, so this crate tests with no game running.
pub mod sim;

use sim::sand::{Sand, Spot};
use sim::tree::{self, Part};
use sim::water::{Cell, Grid, Water};
use sim::{P, SIDES, above, add, below};
use std::cell::Cell as Flag;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const WATER_CELLS_PER_TICK: usize = 1500;
pub const WATER_TIME_PER_TICK: Duration = Duration::from_millis(4);
pub const SAND_SLIDES_PER_TICK: usize = 32;
pub const SAND_CHECKS_PER_TICK: usize = 1024;
/// Thrown leaves per tree. The rest vanish: nobody counts them mid-crash.
pub const LEAVES_THROWN: usize = 160;
pub const TREES_PER_TICK: usize = 2;

/// What the game gives the engine, for the level in place.
pub trait World {
    fn water_cell(&mut self, p: P) -> Cell;
    fn set_water(&mut self, p: P, cell: Cell) -> bool;
    fn sand_spot(&mut self, p: P) -> Spot;
    /// Take the grain away and throw it at `to`.
    fn slide_grain(&mut self, from: P, to: P);
    fn tree_part(&mut self, p: P) -> Part;
    /// Read the block's state, clear it, and remember it for `throw_piece`.
    fn lift_piece(&mut self, p: P);
    /// Throw the lifted block. A log turns to lie along the fall.
    fn throw_piece(&mut self, p: P, velocity: [f64; 3], log: bool, axis_x: bool);
    fn tree_sound(&mut self, at: P);
}

enum Note {
    /// A block changed. `only_water` means air and water, so no sand can care.
    Changed { p: P, only_water: bool },
    /// The game wanted to run its own water here.
    Flow(P),
    Cut { p: P, player: Option<[f64; 3]> },
}

static INBOX: Mutex<Vec<(u8, Note)>> = Mutex::new(Vec::new());
static LEVELS: Mutex<Option<HashMap<u8, Level>>> = Mutex::new(None);

thread_local! {
    /// Set while the engine itself writes water, so it does not wake itself.
    static WRITING: Flag<bool> = const { Flag::new(false) };
}

#[derive(Default)]
struct Level {
    water: Water,
    sand: Sand,
    cuts: Vec<(P, Option<[f64; 3]>)>,
}

fn dim_key(dimension: &str) -> u8 {
    match dimension {
        "minecraft:overworld" => 0,
        "minecraft:the_nether" => 1,
        "minecraft:the_end" => 2,
        _ => 3,
    }
}

fn post(dimension: &str, note: Note) {
    if let Ok(mut inbox) = INBOX.lock() {
        inbox.push((dim_key(dimension), note));
    }
}

/// A committed block change.
pub fn block_changed(dimension: &str, p: P, only_water: bool) {
    if only_water && WRITING.with(|w| w.get()) {
        return;
    }
    post(dimension, Note::Changed { p, only_water });
}
/// The game's own water tried to move here: blocked by the hook, handed to the sim.
pub fn vanilla_flow(dimension: &str, p: P) {
    post(dimension, Note::Flow(p));
}
/// A player broke a log.
pub fn log_cut(dimension: &str, p: P, player: Option<[f64; 3]>) {
    post(dimension, Note::Cut { p, player });
}
/// A world closed: nothing carries into the next one.
pub fn reset() {
    if let Ok(mut levels) = LEVELS.lock() {
        *levels = None;
    }
    if let Ok(mut inbox) = INBOX.lock() {
        inbox.clear();
    }
}

struct Writes<'a, W: World>(&'a mut W);
impl<W: World> Grid for Writes<'_, W> {
    fn read(&mut self, p: P) -> Cell {
        self.0.water_cell(p)
    }
    fn write(&mut self, p: P, cell: Cell) -> bool {
        WRITING.with(|w| w.set(true));
        let ok = self.0.set_water(p, cell);
        WRITING.with(|w| w.set(false));
        ok
    }
}

/// Run one tick of the level in place. Call once per level per server tick.
pub fn tick<W: World>(dimension: &str, world: &mut W) {
    let key = dim_key(dimension);
    let notes: Vec<(u8, Note)> = {
        let Ok(mut inbox) = INBOX.lock() else { return };
        if inbox.iter().all(|(k, _)| *k != key) {
            Vec::new()
        } else {
            let (mine, rest): (Vec<_>, Vec<_>) = inbox.drain(..).partition(|(k, _)| *k == key);
            *inbox = rest;
            mine
        }
    };
    let Ok(mut levels) = LEVELS.lock() else { return };
    let level = levels.get_or_insert_with(HashMap::new).entry(key).or_default();
    for (_, note) in notes {
        match note {
            Note::Changed { p, only_water } => {
                level.water.wake_around(p);
                if !only_water {
                    level.sand.wake_around(p);
                }
            }
            Note::Flow(p) => level.water.wake(p),
            Note::Cut { p, player } => level.cuts.push((p, player)),
        }
    }
    run_water(level, world);
    run_sand(level, world);
    run_trees(level, world);
}

fn run_water<W: World>(level: &mut Level, world: &mut W) {
    if level.water.awake() == 0 {
        return;
    }
    let end = Instant::now() + WATER_TIME_PER_TICK;
    level
        .water
        .step(&mut Writes(world), WATER_CELLS_PER_TICK, || Instant::now() < end);
}

fn run_sand<W: World>(level: &mut Level, world: &mut W) {
    if level.sand.awake() == 0 {
        return;
    }
    let slides = level.sand.step(
        &mut |p| world.sand_spot(p),
        SAND_SLIDES_PER_TICK,
        SAND_CHECKS_PER_TICK,
    );
    for s in slides {
        world.slide_grain(s.from, s.to);
    }
}

fn run_trees<W: World>(level: &mut Level, world: &mut W) {
    let take = level.cuts.len().min(TREES_PER_TICK);
    for (cut, player) in level.cuts.drain(..take).collect::<Vec<_>>() {
        let from = player.unwrap_or([cut[0] as f64 - 1.0, cut[1] as f64, cut[2] as f64 + 0.5]);
        let dir = tree::fall_dir(cut, from);
        let Some(fall) = tree::plan(cut, dir, &mut |p| world.tree_part(p)) else {
            continue;
        };
        world.tree_sound(cut);
        // Clear everything first, so no piece lands on a block about to vanish.
        for piece in &fall.pieces {
            world.lift_piece(piece.pos);
        }
        let every = (fall.leaves / LEAVES_THROWN).max(1);
        let mut leaf = 0;
        for piece in &fall.pieces {
            if !piece.log {
                leaf += 1;
                if leaf % every != 0 {
                    continue;
                }
            }
            world.throw_piece(piece.pos, piece.velocity, piece.log, dir[0] != 0);
        }
    }
}

pub fn is_grain(name: &str) -> bool {
    matches!(name, "minecraft:sand" | "minecraft:red_sand" | "minecraft:gravel")
}
pub fn is_washable(name: &str) -> bool {
    matches!(
        name,
        "minecraft:air"
            | "minecraft:cave_air"
            | "minecraft:void_air"
            | "minecraft:short_grass"
            | "minecraft:fern"
            | "minecraft:dandelion"
            | "minecraft:poppy"
            | "minecraft:oxeye_daisy"
            | "minecraft:cornflower"
            | "minecraft:azure_bluet"
            | "minecraft:dead_bush"
    )
}
pub fn is_log(name: &str) -> bool {
    name.ends_with("_log") || name.ends_with("_stem")
}
pub fn is_leaves(name: &str) -> bool {
    name.ends_with("_leaves")
}
pub fn tree_part(name: &str, persistent: Option<&str>) -> Part {
    if is_log(name) {
        Part::Log
    } else if is_leaves(name) {
        if persistent == Some("true") { Part::PlacedLeaf } else { Part::NaturalLeaf }
    } else if is_washable(name) || name == "minecraft:water" {
        Part::Other
    } else {
        Part::Ground
    }
}
/// A water block's `level` to units, and back. Falling states (8+) count as full.
pub fn level_to_units(level: i32) -> u8 {
    if (1..=7).contains(&level) { (8 - level) as u8 } else { 8 }
}
pub fn units_to_level(units: u8) -> i32 {
    8 - i32::from(units.clamp(1, 8))
}
