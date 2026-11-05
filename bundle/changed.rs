fn realphysics_changed(
    level: &crate::game::GameLevel<'_>,
    dimension: LevelDimension,
    pos: BlockPos,
    old: BlockStateId,
    new: BlockStateId,
) {
    let fluidish = |id| {
        let name: &str = &level.facts.get(id).name;
        name == "minecraft:water" || realphysics_engine::is_washable(name)
    };
    realphysics_engine::block_changed(
        realphysics_dim(dimension),
        [pos.x, pos.y, pos.z],
        fluidish(old) && fluidish(new),
    );
    fn realphysics_dim(d: LevelDimension) -> &'static str {
        match d {
            LevelDimension::Overworld => "minecraft:overworld",
            LevelDimension::Nether => "minecraft:the_nether",
            LevelDimension::End => "minecraft:the_end",
        }
    }
}
