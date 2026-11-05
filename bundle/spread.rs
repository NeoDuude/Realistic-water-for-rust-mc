// The game's own water would spread here: stop it and wake the sim. Lava and fire stay vanilla.
fn realphysics_spread(
    level: &crate::game::GameLevel<'_>,
    dimension: LevelDimension,
    kind: &str,
    from: BlockPos,
    to: BlockPos,
    canceled: &mut bool,
) {
    if kind != "fluid" {
        return;
    }
    let state = level.world.block(from.x, from.y, from.z);
    if level.facts.get(state).name.as_ref() as &str == "minecraft:lava" {
        return;
    }
    *canceled = true;
    let dim = match dimension {
        LevelDimension::Overworld => "minecraft:overworld",
        LevelDimension::Nether => "minecraft:the_nether",
        LevelDimension::End => "minecraft:the_end",
    };
    realphysics_engine::vanilla_flow(dim, [from.x, from.y, from.z]);
    realphysics_engine::vanilla_flow(dim, [to.x, to.y, to.z]);
}
