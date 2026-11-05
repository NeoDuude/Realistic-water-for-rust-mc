// Runs the engine once per level, in the level's own tick.
fn realphysics_tick(game: &mut crate::game::Game<'_>, dimension: &str) {
    use realphysics_engine::sim::sand::Spot;
    use realphysics_engine::sim::tree::Part;
    use realphysics_engine::sim::water::Cell;
    use realphysics_engine::sim::P;
    use realphysics_engine::{World, is_grain, is_washable, tree_part};
    use std::collections::HashMap;

    const SKIP_PLACE: i32 = 512;
    const CLIENTS: i32 = 2;
    const KNOWN_SHAPE: i32 = 16;
    const NEIGHBOURS: i32 = 1;

    if !game.native_authoritative() {
        return;
    }

    struct Live<'g, 'a> {
        game: &'g mut crate::game::Game<'a>,
        dim: String,
        water: Option<BlockStateId>,
        air: Option<BlockStateId>,
        lifted: HashMap<P, BlockStateId>,
    }
    impl Live<'_, '_> {
        fn loaded(&self, p: P) -> bool {
            let w = &self.game.world;
            (w.bottom_y()..w.top_y()).contains(&p[1]) && w.chunks.contains_key(&(p[0] >> 4, p[2] >> 4))
        }
        fn state(&self, p: P) -> BlockStateId {
            self.game.world.block(p[0], p[1], p[2])
        }
        fn name(&self, p: P) -> &str {
            &self.game.res.facts().get(self.state(p)).name
        }
    }
    impl World for Live<'_, '_> {
        fn water_cell(&mut self, p: P) -> Cell {
            if !self.loaded(p) {
                return Cell::Unknown;
            }
            let name = self.name(p);
            if name == "minecraft:water" {
                let level = self
                    .game
                    .pack
                    .assets
                    .registry
                    .state(self.state(p))
                    .and_then(|s| s.properties.get("level"))
                    .and_then(|v| v.parse::<i32>().ok())
                    .unwrap_or(0);
                Cell::Water(realphysics_engine::level_to_units(level))
            } else if is_washable(name) {
                Cell::Open
            } else {
                Cell::Solid
            }
        }
        fn set_water(&mut self, p: P, cell: Cell) -> bool {
            let was_water = self.name(p) == "minecraft:water";
            let (state, now_water) = match cell {
                Cell::Water(units) => {
                    let Some(source) = self.water else { return false };
                    let level = realphysics_engine::units_to_level(units).to_string();
                    let reg = self.game.pack.assets.registry;
                    let Some(state) = with_property(reg, source, "level", &level) else {
                        return false;
                    };
                    (state, true)
                }
                _ => {
                    let Some(air) = self.air else { return false };
                    (air, false)
                }
            };
            // Level-only changes tell nobody. Water arriving or leaving does.
            let flags = if was_water == now_water {
                CLIENTS | KNOWN_SHAPE | SKIP_PLACE
            } else {
                CLIENTS | NEIGHBOURS | SKIP_PLACE
            };
            let dim = self.dim.clone();
            self.game.native_set_block(&dim, p, state, flags)
        }
        fn sand_spot(&mut self, p: P) -> Spot {
            if !self.loaded(p) {
                return Spot::Solid;
            }
            let name = self.name(p);
            if is_grain(name) {
                Spot::Grain
            } else if name == "minecraft:water" {
                Spot::Water
            } else if is_washable(name) {
                Spot::Open
            } else {
                Spot::Solid
            }
        }
        fn slide_grain(&mut self, from: P, to: P) {
            let (state, dim) = (self.state(from), self.dim.clone());
            let Some(air) = self.air else { return };
            if !self.game.native_set_block(&dim, from, air, CLIENTS | NEIGHBOURS) {
                return;
            }
            let at = [f64::from(to[0]) + 0.5, f64::from(to[1]), f64::from(to[2]) + 0.5];
            let thrown = self.game.native_spawn_falling_block(
                &dim,
                state,
                at,
                [0.0, 0.0, 0.0],
                crate::game::native_falling::NativeFalling { drop_item: true, hurts: false },
            );
            if !thrown {
                // At the falling block limit: leave the grain where it was.
                self.game.native_set_block(&dim, from, state, CLIENTS | NEIGHBOURS);
            }
        }
        fn tree_part(&mut self, p: P) -> Part {
            if !self.loaded(p) {
                return Part::Ground;
            }
            let state = self.state(p);
            let props = self.game.pack.assets.registry.state(state);
            let persistent = props.and_then(|s| s.properties.get("persistent")).map(|v| v.as_str());
            tree_part(self.name(p), persistent)
        }
        fn lift_piece(&mut self, p: P) {
            let (state, dim) = (self.state(p), self.dim.clone());
            let Some(air) = self.air else { return };
            self.lifted.insert(p, state);
            self.game.native_set_block(&dim, p, air, CLIENTS | KNOWN_SHAPE);
        }
        fn throw_piece(&mut self, p: P, velocity: [f64; 3], log: bool, axis_x: bool) {
            let Some(mut state) = self.lifted.get(&p).copied() else { return };
            if log {
                let reg = self.game.pack.assets.registry;
                let axis = if axis_x { "x" } else { "z" };
                state = with_property(reg, state, "axis", axis).unwrap_or(state);
            }
            let at = [f64::from(p[0]) + 0.5, f64::from(p[1]), f64::from(p[2]) + 0.5];
            let dim = self.dim.clone();
            self.game.native_spawn_falling_block(
                &dim,
                state,
                at,
                velocity,
                crate::game::native_falling::NativeFalling { drop_item: log, hurts: log },
            );
        }
        fn tree_sound(&mut self, at: P) {
            let at = [f64::from(at[0]) + 0.5, f64::from(at[1]) + 1.0, f64::from(at[2]) + 0.5];
            const BLOCKS: usize = 4;
            self.game.native_level_play_sound("minecraft:entity.zombie.break_wooden_door", at, BLOCKS, 1.0, 0.6);
        }
    }

    let registry = game.pack.assets.registry;
    let water = registry.default_state("minecraft:water");
    let air = registry.default_state("minecraft:air");
    let mut live = Live { game, dim: dimension.to_owned(), water, air, lifted: HashMap::new() };
    realphysics_engine::tick(dimension, &mut live);
}
