use legion::Entity;

use traffloat::edge;

/// Sets up legion ECS.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    let (def, nodes, edges) = traffloat_vanilla::get();
    let mut setup = setup.resource(def);

    let entities: Vec<Entity> = nodes
        .into_iter()
        .map(|node| setup.world.push(node))
        .collect();

    #[allow(clippy::indexing_slicing)]
    for (from, to, size) in edges {
        setup
            .world
            .push(edge::create_components(entities[from], entities[to], size));
    }

    setup
}
