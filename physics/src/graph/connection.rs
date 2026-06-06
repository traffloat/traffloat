//! A fluid connection between a facility and its peer (ambient, another facility, or a conduit).
//!
//! This is exclusively for connection between two [`fluid::Storage`] entities.

use std::f32::consts::PI;
use std::mem;

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::name::Name;
use bevy::ecs::query::QueryData;
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{EntityCommand, Query, SystemParam};
use bevy::ecs::world::EntityWorldMut;
use bevy::reflect::Reflect;
use traffloat_proto::proto;

use crate::graph::{Conduit, Corridor, Facility, conduit, facility};
use crate::util::{QueryExt, WorldExt};
use crate::{fluid, view};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Connection>();
        app.register_type::<MainFacility>();
        app.register_type::<ListOnMainFacility>();
        app.register_type::<AltFacility>();
        app.register_type::<ListOnAltFacility>();
        app.register_type::<ToBuilding>();
        app.register_type::<ListOnBuilding>();
        app.register_type::<ToPipe>();
        app.register_type::<ListOnPipe>();

        app.add_systems(app::Update, init_viewer_system.in_set(view::SendUpdatesSystemSet::Init));
        app.add_systems(
            app::Update,
            broadcast_building_changes_system
                .in_set(super::ViewSystemSets::Connection)
                .in_set(view::SendUpdatesSystemSet::Incr),
        );
    }
}

#[derive(Component, Reflect)]
#[component(immutable)]
pub struct Connection {
    pub max_area: f32,
}

/// The main facility that this connection connects to.
/// Component on connections.
#[derive(Component, Reflect)]
#[relationship(relationship_target = ListOnMainFacility)]
pub struct MainFacility(pub Entity);

/// List of connections on a facility where it is the main facility.
/// Component on facilities.
///
/// In practice, users of this component will almost always want to
/// use [`ListOnAltFacility`] as well.
#[derive(Component, Reflect)]
#[relationship_target(relationship = MainFacility, linked_spawn)]
pub struct ListOnMainFacility(Vec<Entity>);

/// The alternate facility that this connection connects to.
/// Component on connections, when they are facility&ndash;facility connections.
///
/// Must belong to the same building as [`MainFacility`]'s.
#[derive(Component, Reflect)]
#[relationship(relationship_target = ListOnAltFacility)]
pub struct AltFacility(pub Entity);

/// List of connections on a facility where it is the alternate facility.
/// Component on facilities.
///
/// In practice, users of this component will almost always want to
/// use [`ListOnFacility`] as well.
#[derive(Component, Reflect)]
#[relationship_target(relationship = AltFacility, linked_spawn)]
pub struct ListOnAltFacility(Vec<Entity>);

/// Marks the connection as a facility&ndash;building connections.
///
/// The referenced entity is always the parent building of the facility referenced by [`OfFacility`].
#[derive(Component, Reflect)]
#[relationship(relationship_target = ListOnBuilding)]
pub struct ToBuilding(pub Entity);

/// List of connections connecting the ambient storage of the building to its inner facilities.
/// Component on buildings.
#[derive(Component, Reflect)]
#[relationship_target(relationship = ToBuilding, linked_spawn)]
pub struct ListOnBuilding(Vec<Entity>);

/// The pipe that this connection connects to.
/// Component on connections, when they are facility&ndash;pipe connections.
///
/// This component is not inserted on ambient&ndash;corridor connections.
/// Those should use [`super::edge`] instead.
#[derive(Component, Reflect)]
#[relationship(relationship_target = ListOnPipe)]
pub struct ToPipe(pub Entity);

/// List of connections a fluid pipe is connected to.
/// Component on pipe conduits.
///
/// In practice, each endpoint of a pipe will have at most one connection,
/// so this component should not contain more than two entities.
/// However, this restriction is not imposed at the graph level
/// and should be handled by user request validation instead.
#[derive(Component, Reflect)]
#[relationship_target(relationship = ToPipe, linked_spawn)]
pub struct ListOnPipe(Vec<Entity>);

#[derive(Component, Reflect)]
pub struct BuildingHasChanges(pub bool);

pub struct SpawnCommand {
    pub main: Entity,
    pub peer: SpawnPeer,
}

pub enum SpawnPeer {
    Facility(Entity),
    Building(Entity),
    Pipe(Entity),
}

impl EntityCommand for SpawnCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let Some(&Facility { volume: main_volume, .. }) =
            entity.world().log_get::<Facility>(self.main)
        else {
            return;
        };

        let max_area;
        let resistance;
        let peer_storage = match self.peer {
            SpawnPeer::Facility(peer) => {
                let Some(&Facility { volume: peer_volume, .. }) =
                    entity.world().log_get::<Facility>(peer)
                else {
                    return;
                };
                max_area = main_volume.min(peer_volume).powf(2.0 / 3.0);
                resistance = main_volume.cbrt() + peer_volume.cbrt();

                entity.insert(AltFacility(peer));
                peer
            }
            SpawnPeer::Building(peer) => {
                max_area = main_volume.powf(2.0 / 3.0);
                resistance = main_volume.cbrt();

                entity.insert(ToBuilding(peer));
                peer
            }
            SpawnPeer::Pipe(peer) => {
                let Some(conduit) = entity.world().log_get::<Conduit>(peer) else { return };
                let Some(&conduit::OfCorridor(corridor_entity)) = entity.world().log_get(peer)
                else {
                    return;
                };
                let Some(corridor) = entity.world().log_get::<Corridor>(corridor_entity) else {
                    return;
                };
                max_area = conduit.radius.powi(2) * PI;
                resistance = corridor.length * 0.5 + main_volume.cbrt();

                entity.insert(ToPipe(peer));
                peer
            }
        };

        entity.insert((
            Name::new("Facility connection"),
            Connection { max_area },
            MainFacility(self.main),
        ));
        fluid::AddEdgeCommand {
            resistance_recip: resistance.recip(),
            area:             max_area,
            alpha:            self.main,
            beta:             peer_storage,
        }
        .apply(entity);
    }
}

fn init_viewer_system(
    building_query: Query<(&view::Viewable, &facility::ListOnBuilding)>,
    mut params: BroadcastChangesParams,
) {
    for (viewable, facility_list) in building_query {
        broadcast_changes(viewable, facility_list, &mut params, true);
    }
}

fn broadcast_building_changes_system(
    building_query: Query<(&view::Viewable, &mut BuildingHasChanges, &facility::ListOnBuilding)>,
    mut params: BroadcastChangesParams,
) {
    for (viewable, mut has_changes, facility_list) in building_query {
        if mem::take(&mut has_changes.0) {
            broadcast_changes(viewable, facility_list, &mut params, false);
        }
    }
}

#[derive(SystemParam)]
struct BroadcastChangesParams<'w, 's> {
    // we do not include `ListOnAltFacility` here because they must have been mentioned
    // by the main facility in the same building
    facility_query:   Query<'w, 's, &'static ListOnMainFacility>,
    connection_query: Query<'w, 's, BroadcastChangesConnectionData>,
    viewable_query:   Query<'w, 's, &'static view::Viewable>,
    writer:           MessageWriter<'w, view::SentUpdate>,
}

#[derive(QueryData)]
struct BroadcastChangesConnectionData {
    connection:   &'static Connection,
    pipe:         Option<&'static ToPipe>,
    alt_facility: Option<&'static AltFacility>,
    building:     Option<&'static ToBuilding>,
    edge:         &'static fluid::Edge,
}

fn broadcast_changes(
    building_viewable: &view::Viewable,
    facility_list: &facility::ListOnBuilding,
    params: &mut BroadcastChangesParams,
    is_init: bool,
) {
    fn make_proto_pair(
        viewable_query: &Query<&view::Viewable>,
        main_id: proto::Id,
        data: &BroadcastChangesConnectionDataItem,
        connection_entity: Entity,
    ) -> Option<proto::BuildingFluidConnectionPair> {
        if let Some(pipe) = data.pipe {
            Some(proto::BuildingFluidConnectionPair::FacilityPipe {
                facility: main_id,
                pipe:     viewable_query.log_get(pipe.0)?.id,
            })
        } else if let Some(alt_facility) = data.alt_facility {
            Some(proto::BuildingFluidConnectionPair::FacilityFacility(
                main_id,
                viewable_query.log_get(alt_facility.0)?.id,
            ))
        } else if let Some(building) = data.building {
            Some(proto::BuildingFluidConnectionPair::FacilityBuilding {
                facility: main_id,
                building: viewable_query.log_get(building.0)?.id,
            })
        } else {
            tracing::warn!("Connection {connection_entity:?} has no peer component");
            None
        }
    }

    let make_update = || {
        let mut connections = Vec::new();

        for facility in facility_list.iter() {
            let Some(&view::Viewable { id: main_id, .. }) = params.viewable_query.log_get(facility)
            else {
                continue;
            };
            let Ok(connection_list) = params.facility_query.get(facility) else { continue };
            for connection in connection_list.iter() {
                let Some(data) = params.connection_query.log_get(connection) else { continue };
                if let Some(pair) =
                    make_proto_pair(&params.viewable_query, main_id, &data, connection)
                {
                    connections.push(proto::BuildingFluidConnection {
                        current_area: data.edge.area,
                        max_area: data.connection.max_area,
                        pair,
                    });
                }
            }
        }

        proto::Update::SetBuildingFluidConnections(proto::SetBuildingFluidConnections {
            id: building_viewable.id,
            connections,
        })
    };

    if is_init {
        params.writer.write_batch(building_viewable.broadcast_new(|| [make_update()]));
    } else {
        let update = make_update();
        params.writer.write_batch(building_viewable.broadcast_update(|_| [update.clone()]));
    }
}
