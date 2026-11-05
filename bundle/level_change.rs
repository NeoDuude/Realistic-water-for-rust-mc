// The game's own water would change level here: stop it, the sim owns water.
fn realphysics_level_change(dimension: LevelDimension, pos: BlockPos, fluid: &str, cancel: &mut bool) {
    if fluid != "water" {
        return;
    }
    *cancel = true;
    let dim = match dimension {
        LevelDimension::Overworld => "minecraft:overworld",
        LevelDimension::Nether => "minecraft:the_nether",
        LevelDimension::End => "minecraft:the_end",
    };
    realphysics_engine::vanilla_flow(dim, [pos.x, pos.y, pos.z]);
}
