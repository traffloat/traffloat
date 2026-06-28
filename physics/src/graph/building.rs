use std::f32::consts::PI;

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::name::Name;
use bevy::ecs::query::{QueryData, With};
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{EntityCommand, Query, SystemParam};
use bevy::ecs::world::EntityWorldMut;
use bevy::math::{Rect, Vec2};
use bevy::reflect::Reflect;
use traffloat_proto::proto;

use crate::graph::{Facility, ViewInitSystemSets, edge, facility};
use crate::persist::AppExt;
use crate::util::{Alpha, Beta, EntityWorldMutExt, QueryExt, Which, WorldExt};
use crate::{Vector, fluid, view};

mod persist;
pub use persist::Persist;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Building>();

        app.register_persistable(Persist);

        app.add_systems(
            app::Update,
            init_viewer_system
                .in_set(view::SendUpdatesSystemSet::Init)
                .in_set(ViewInitSystemSets::Building),
        );
        app.add_systems(
            app::Update,
            incr_viewer_system
                .in_set(super::ViewIncrSystemSets::Building)
                .in_set(view::SendUpdatesSystemSet::Incr),
        );
    }
}

#[derive(Component, Reflect)]
pub struct Building {
    pub position:       Vector,
    pub radius:         f32,
    pub wall_thickness: f32,
    pub ambient_volume: f32,
}

impl Building {
    /// The rect enclosing the building wall only.
    pub fn base_rect(&self) -> Rect {
        Rect::from_center_half_size(self.position, Vec2::splat(self.radius + self.wall_thickness))
    }
}

pub struct SpawnCommand {
    pub name:           String,
    pub position:       Vector,
    pub radius:         f32,
    pub wall_thickness: f32,
}

impl EntityCommand for SpawnCommand {
    type Out = ();
    fn apply(self, mut entity: EntityWorldMut) {
        let ambient_volume = sphere_volume(self.radius);

        let building = Building {
            position: self.position,
            radius: self.radius,
            wall_thickness: self.wall_thickness,
            ambient_volume,
        };
        entity.insert((
            Name::new(format!("Building {}", self.name)),
            view::CullingRect(building.base_rect()),
            view::Named { name: self.name },
            building,
        ));
        entity.reborrow_scope(|entity| view::AddViewableCommand.apply(entity));

        // ambient storage
        entity.reborrow_scope(|entity| {
            fluid::AddStorageCommand {
                volume:         ambient_volume,
                optical_length: self.radius,
            }
            .apply(entity);
        });
    }
}

/// Recomputes the culling rect of a building,
/// enclosing its own volume as well as all connected corridors and their peer buildings.
///
/// This function must be called after the culling rect of connected corridors are updated,
/// since the building's culling rect de
#[tracing::instrument(level = "trace", skip(params))]
pub(super) fn recompute_culling_rect(mut params: RecomputeCullingRectParams, building: Entity) {
    fn add_edges<Ab: Which>(
        rect: &mut Rect,
        edge_list: Option<&edge::BuildingEdges<Ab>>,
        edge_query: &Query<RecomputeCullingRectEdgeData<Ab>>,
        corridor_query: &Query<RecomputeCullingRectCorridorData>,
    ) {
        for edge in edge_list.iter().flat_map(|edges| edges.iter()) {
            let Some(edge) = edge_query.log_get(edge) else { continue };
            let Some(corridor) = corridor_query.log_get(edge.corridor.0) else { continue };
            *rect = rect.union(corridor.rect.0);
        }
    }

    let Some(RecomputeCullingRectBuildingDataItem { building, alpha_edges, beta_edges, mut rect }) =
        params.building_query.log_get_mut(building)
    else {
        return;
    };
    let rect = &mut rect.0;

    *rect = building.base_rect();
    add_edges(rect, alpha_edges, &params.alpha_edge_query, &params.corridor_query);
    add_edges(rect, beta_edges, &params.beta_edge_query, &params.corridor_query);
}

#[derive(SystemParam)]
pub(super) struct RecomputeCullingRectParams<'w, 's> {
    building_query:   Query<'w, 's, RecomputeCullingRectBuildingData>,
    alpha_edge_query: Query<'w, 's, RecomputeCullingRectEdgeData<Alpha>>,
    beta_edge_query:  Query<'w, 's, RecomputeCullingRectEdgeData<Beta>>,
    corridor_query:   Query<'w, 's, RecomputeCullingRectCorridorData>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct RecomputeCullingRectBuildingData {
    building:    &'static Building,
    alpha_edges: Option<&'static edge::BuildingEdges<Alpha>>,
    beta_edges:  Option<&'static edge::BuildingEdges<Beta>>,
    rect:        &'static mut view::CullingRect,
}

#[derive(QueryData)]
struct RecomputeCullingRectEdgeData<Ab: Which> {
    corridor: &'static edge::OfCorridor<Ab>,
}

#[derive(QueryData)]
struct RecomputeCullingRectCorridorData {
    rect: &'static view::CullingRect,
}

pub struct DespawnCommand;

impl EntityCommand for DespawnCommand {
    type Out = ();
    fn apply(self, mut entity: EntityWorldMut) {
        view::before_viewable_despawn(&mut entity);
        entity.despawn();
    }
}

pub struct RecomputeAmbientVolume;

impl EntityCommand for RecomputeAmbientVolume {
    type Out = ();
    fn apply(self, mut entity: EntityWorldMut) {
        let used_by_facilities: f32 =
            if let Some(facility_list) = entity.get::<facility::ListOnBuilding>() {
                let facilities: Vec<_> = facility_list.iter().collect();
                facilities
                    .iter()
                    .filter_map(|&f| entity.world().log_get::<Facility>(f))
                    .map(|f| f.volume)
                    .sum()
            } else {
                0.0
            };
        let Some(mut building) = entity.log_get_mut::<Building>() else { return };

        let ambient_volume = sphere_volume(building.radius) - used_by_facilities;
        building.ambient_volume = ambient_volume;

        if let Some(mut fluid) = entity.log_get_mut::<fluid::Storage>() {
            fluid.volume = ambient_volume;
        }
    }
}

fn sphere_volume(radius: f32) -> f32 { radius.powi(3) * PI * 4.0 / 3.0 }

fn init_viewer_system(
    building_query: Query<(&Building, &view::Named, &view::Viewable)>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    for (building, named, viewable) in building_query {
        messages.write_batch(viewable.broadcast_new(|| {
            [proto::Update::NewBuilding(proto::NewBuilding {
                id:             viewable.id,
                name:           named.name.clone(),
                position:       building.position,
                radius:         building.radius,
                wall_thickness: building.wall_thickness,
            })]
        }));
    }
}

fn incr_viewer_system(
    mut throttle: view::BroadcastThrottle,
    building_query: Query<(&view::Viewable, &fluid::Storage, &fluid::Sensor), With<Building>>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    if !throttle.should_run() {
        return;
    }

    for (viewable, storage, sensor) in building_query {
        messages.write_batch(viewable.broadcast_update(|level| {
            let mut update = proto::UpdateBuilding {
                id:            viewable.id,
                color:         proto::Color(storage.rgba),
                ambient_fluid: None,
            };
            match level {
                view::SubscriptionLevel::Optical => {}
                view::SubscriptionLevel::Detail => {
                    update.ambient_fluid = Some(storage.to_proto_normal(sensor));
                }
                view::SubscriptionLevel::Debug => {
                    update.ambient_fluid = Some(storage.to_proto_debug());
                }
            }
            [update.into()]
        }));
    }
}
