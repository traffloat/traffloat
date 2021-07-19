use legion::Entity;

use traffloat::graph::{EdgeId, EdgeSize, NodeId};

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
        let from_id: NodeId = *setup
            .world
            .entry(entities[from])
            .expect("Just pushed")
            .into_component()
            .expect("Initial node does not have NodeId");
        let to_id: NodeId = *setup
            .world
            .entry(entities[to])
            .expect("Just pushed")
            .into_component()
            .expect("Initial node does not have NodeId");

        let mut edge = EdgeId::new(from_id, to_id);
        edge.set_from_entity(Some(entities[from]));
        edge.set_to_entity(Some(entities[to]));
        setup.world.push((edge, EdgeSize::new(size)));
    }

    setup
}
