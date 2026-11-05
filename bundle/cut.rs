// A player broke a block. A log may bring a tree down next tick, once the block is gone.
fn realphysics_cut(game: &mut crate::game::Game<'_>, pos: [i32; 3]) {
    let state = game.world.block(pos[0], pos[1], pos[2]);
    let is_log = realphysics_engine::is_log(&game.res.facts().get(state).name);
    if !is_log {
        return;
    }
    let player = game.native_acting_player().map(|p| p.position);
    let dim = game.dim.id().to_owned();
    realphysics_engine::log_cut(&dim, pos, player);
}
