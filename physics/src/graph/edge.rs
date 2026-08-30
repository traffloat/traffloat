//! An edge entity represents a connection between a corridor and a building.
//!
//! A corridor may have 1 or 2 endpoints.
//! When a corridor is constructed, the base building is assigned as alpha.
//! It is allowed to build the corridor towards either a new building or a fixed position.
//! In case of the former, the new building becomes the beta endpoint.
//! In case of the latter, the beta endpoint is not set.
//!
//! When a building is destroyed,
//! the corresponding relationship component is removed from its connected corridors.
//! A corridor is removed only when both buildings are removed.
//!
//! An edge needs to be its own entity because it needs to hold
//! building-corridor components like [`crate::fluid::Edge`],
//! which are independent on both sides of an edge.
//!
//! # Viewable
//! The edge entity itself is not a viewable.
//! Viewable updates would be sent as part of corridor updates instead.

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::name::Name;
use bevy::ecs::query::{Changed, QueryFilter};
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{EntityCommand, Query};
use bevy::ecs::world::{EntityWorldMut, World};
use bevy::math::Vec3;
use bevy::reflect::Reflect;
use traffloat_proto::proto;
use traffloat_util::{Alpha, Beta, EntityWorldMutExt, QueryExt, Which, WorldExt};

use crate::graph::{Building, Corridor};
use crate::persist::AppExt;
use crate::{fluid, view};

mod persist;
pub use persist::Persist;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        fn register_ab<Ab: Which>(app: &mut App) {
            app.register_type::<OfBuilding<Ab>>();
            app.register_type::<BuildingEdges<Ab>>();
            app.register_type::<OfCorridor<Ab>>();
            app.register_type::<CorridorEdge<Ab>>();
        }

        app.register_type::<Edge>();

        register_ab::<Alpha>(app);
        register_ab::<Beta>(app);

        app.register_persistable(Persist);

        app.add_systems(
            app::Update,
            (
                broadcast_edge_change_system::<Alpha, BroadcastToNewViewers>,
                broadcast_edge_change_system::<Beta, BroadcastToNewViewers>,
                broadcast_edge_change_system::<Alpha, BroadcastDataChange>,
                broadcast_edge_change_system::<Beta, BroadcastDataChange>,
            )
                .chain()
                .in_set(view::SendUpdatesSystemSet::Incr)
                .in_set(super::ViewIncrSystemSets::Edge),
        );
    }
}

#[derive(Component, Reflect)]
pub struct Edge {
    pub open: bool,
}

/// Component on edge referencing building.
#[derive(Component, Reflect)]
#[relationship(relationship_target = BuildingEdges<Ab>)]
pub struct OfBuilding<Ab: Which> {
    #[relationship]
    pub building:     Entity,
    /// Position of the edge relative to the building center.
    pub interior_pos: Vec3,
    which:            Ab,
}

/// Component on building listing edges.
#[derive(Component, Reflect)]
#[relationship_target(relationship = OfBuilding<Ab>, linked_spawn)]
pub struct BuildingEdges<Ab: Which>(#[relationship] Vec<Entity>, Ab);

/// Component on edge referencing corridor.
#[derive(Component, Reflect)]
#[relationship(relationship_target = CorridorEdge<Ab>)]
pub struct OfCorridor<Ab: Which>(#[relationship] pub Entity, Ab);

/// Component on corridor referencing edge.
#[derive(Component, Reflect)]
#[relationship_target(relationship = OfCorridor<Ab>, linked_spawn)]
pub struct CorridorEdge<Ab: Which>(#[relationship] Entity, Ab);

impl<Ab: Which> CorridorEdge<Ab> {
    pub fn edge(&self) -> Entity { self.0 }
}

pub struct SpawnCommand<Ab: Which> {
    pub building: Entity,
    pub corridor: Entity,
    pub which:    Ab,
    pub open:     bool,
}

impl<Ab: Which> EntityCommand for SpawnCommand<Ab> {
    type Out = ();

    fn apply(self, mut entity: EntityWorldMut) {
        let Some(building) = entity.world().log_get::<Building>(self.building) else { return };
        let Some(endpoint_pos) = entity
            .world()
            .log_get::<Corridor>(self.corridor)
            .map(|c| self.which.select(c.endpoint_positions))
        else {
            return;
        };
        let interior_pos = (endpoint_pos - building.position).extend(0.0);
        entity.insert((
            Name::new(format!("Edge {:?} -> {:?}", self.building, self.corridor)),
            Edge { open: self.open },
            OfBuilding { building: self.building, interior_pos, which: self.which },
            OfCorridor(self.corridor, self.which),
        ));

        if self.open {
            let entity_id = entity.id();
            entity.world_scope(|world| {
                insert_fluid_edge(world, entity_id, self.building, self.corridor);
            });
        }
    }
}

// Convention: fluid edge alpha = building, fluid edge beta = corridor
fn insert_fluid_edge(world: &mut World, edge: Entity, building: Entity, corridor: Entity) {
    let (resistance_recip, area) = {
        let Some(building) = world.log_get::<Building>(building) else { return };
        let Some(corridor) = world.log_get::<Corridor>(corridor) else { return };
        (1.0 / (building.radius + corridor.length * 0.5), corridor.ambient_area)
    };

    fluid::AddEdgeCommand { resistance_recip, area, alpha: building, beta: corridor }
        .apply(world.entity_mut(edge));
}

trait BroadcastChange {
    type Filter: QueryFilter;
    const NEW_SUB_ONLY: bool;
}

enum BroadcastToNewViewers {}

impl BroadcastChange for BroadcastToNewViewers {
    type Filter = ();
    const NEW_SUB_ONLY: bool = true;
}

enum BroadcastDataChange {}

impl BroadcastChange for BroadcastDataChange {
    type Filter = Changed<Edge>;
    const NEW_SUB_ONLY: bool = false;
}

fn broadcast_edge_change_system<Ab: Which, Chg: BroadcastChange>(
    edge_query: Query<(&OfBuilding<Ab>, &OfCorridor<Ab>, &Edge), Chg::Filter>,
    opposite_edge_query: Query<&OfBuilding<Ab::Other>>,
    corridor_query: Query<(&view::Viewable, Option<&CorridorEdge<Ab::Other>>)>,
    building_query: Query<&view::Viewable>,
    mut writer: MessageWriter<view::SentUpdate>,
) {
    for (building, corridor, edge) in edge_query {
        let Some((corridor_viewable, opposite_edge)) = corridor_query.log_get(corridor.0) else {
            continue;
        };
        let Some(building_viewable) = building_query.log_get(building.building) else { continue };
        let opposite_building_viewable = opposite_edge
            .and_then(|opposite| opposite_edge_query.log_get(opposite.0))
            .and_then(|opposite_building| building_query.log_get(opposite_building.building));
        writer.write_batch(view::Viewable::broadcast_update_if_all_optical_and_any_detail(
            [building_viewable, corridor_viewable].into_iter().chain(opposite_building_viewable),
            |level| match level {
                view::SubscriptionLevel::Optical => None,
                view::SubscriptionLevel::Detail | view::SubscriptionLevel::Debug => Some(
                    proto::UpdateCorridorEndpoint {
                        corridor: corridor_viewable.id,
                        which:    Ab::default().proto(),
                        value:    Some(proto::CorridorEndpoint {
                            building: building_viewable.id,
                            open:     edge.open,
                        }),
                    }
                    .into(),
                ),
            },
            Chg::NEW_SUB_ONLY,
        ));
    }
}

pub struct DespawnCommand;

impl EntityCommand for DespawnCommand {
    type Out = ();
    fn apply(self, mut entity: EntityWorldMut) {
        fn cleanup<Ab: Which>(which: Ab, entity: &mut EntityWorldMut) {
            let Some(&OfCorridor::<Ab>(corridor_entity, ..)) = entity.log_get() else { return };

            entity.world_scope(|world| {
                let Some(corridor_viewable) = world.log_get::<view::Viewable>(corridor_entity)
                else {
                    return;
                };
                let update = proto::Update::from(proto::UpdateCorridorEndpoint {
                    corridor: corridor_viewable.id,
                    which:    which.proto(),
                    value:    None,
                });
                let messages: Vec<_> =
                    corridor_viewable.broadcast_update(|_| [update.clone()]).collect();
                world.write_message_batch(messages);
            });
        }

        // Either Alpha or Beta, whichever is absent will return on first line.
        cleanup(Alpha, &mut entity);
        cleanup(Beta, &mut entity);
        entity.despawn();
    }
}
