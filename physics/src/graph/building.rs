use std::f32::consts::PI;

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::name::Name;
use bevy::ecs::query::{QueryData, With, };
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{EntityCommand, Res,Query, SystemParam};
use bevy::ecs::world::EntityWorldMut;
use bevy::math::{Rect, Vec2};
use bevy::reflect::Reflect;
use traffloat_proto::proto;
use traffloat_util::{Alpha, Beta, EntityWorldMutExt, QueryExt, Which, WorldExt, loc};

use crate::graph::{Facility, ViewInitSystemSets, edge, facility};
use crate::persist::AppExt;
use crate::{Vector, cargo, fluid, resident, vehicle, view};

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

        app.add_systems(app::Update, recompute_ambient_volume_system);
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
            let Some(edge) = edge_query.log_get(edge, loc!()) else { continue };
            let Some(corridor) = corridor_query.log_get(edge.corridor.0, loc!()) else { continue };
            *rect = rect.union(corridor.rect.0);
        }
    }

    let Some(RecomputeCullingRectBuildingDataItem { building, alpha_edges, beta_edges, mut rect }) =
        params.building_query.log_get_mut(building, loc!())
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

#[derive(SystemParam)]
struct RecomputeAmbientVolumeContextParams<'w, 's> {
    facility_query: Query<'w, 's, &'static Facility>,
    vehicle_types: Res<'w, vehicle::Types>,
    vehicle_query: Query<'w, 's, &'static vehicle::Vehicle>,
    cargo_types: Res<'w, cargo::Types>,
}

#[derive(QueryData)]
struct RecomputeAmbientVolumeData {
    facilities: Option<&'static facility::ListOnBuilding>,
    // residents:  Option<&'static resident::ListInBuilding>,
    vehicles:   Option<&'static vehicle::ListInBuilding>,
    cargo:     Option<&'static cargo::Storage>,
}

fn recompute_ambient_volume_system(
    building_query: Query<(&mut Building, RecomputeAmbientVolumeData, Option<&mut fluid::Storage>)>,
    ctx: RecomputeAmbientVolumeContextParams,
) {
    fn used_by_facilities(data: &RecomputeAmbientVolumeDataItem, ctx: &RecomputeAmbientVolumeContextParams) -> f32 {
        let Some(list) = data.facilities else { return 0.0 };
        list
            .iter()
            .filter_map(|f| ctx.facility_query.log_get(f, loc!()))
            .map(|f| f.volume)
            .sum()
    }

    fn used_by_residents(data: &RecomputeAmbientVolumeDataItem, ctx: &RecomputeAmbientVolumeContextParams) -> f32 {
        // TODO implement resident::ListInBuilding
        0.0
    }

    fn used_by_vehicles(data: &RecomputeAmbientVolumeDataItem, ctx: &RecomputeAmbientVolumeContextParams) -> f32 {
        let Some(list) = data.vehicles else { return 0.0 };
        list.0.iter().filter_map(|&v| ctx.vehicle_query.log_get(v, loc!()))
            .map(|vehicle| ctx.vehicle_types.get(vehicle.ty).physical.volume)
            .sum()
    }

    #[expect(clippy::cast_precision_loss, reason = "precision loss is acceptable after conversion to volume")]
    fn used_by_cargo(data: &RecomputeAmbientVolumeDataItem, ctx: &RecomputeAmbientVolumeContextParams) -> f32 {
        let Some(storage) = data.cargo else { return 0.0 };
        storage.types().map(|t| ctx.cargo_types.get(t.ty).granule_volume * (t.granules.0 as f32)).sum()
    }

    // This loop is perfectly parallelizable, but the task is largely memory-bound,
    // so parallelization would only be more likely to increase cache misses.
    for (mut building, data, fluid_storage) in building_query {
        let used = used_by_facilities(&data, &ctx) + used_by_residents(&data, &ctx)
            + used_by_vehicles(&data, &ctx) + used_by_cargo(&data, &ctx);

        let ambient_volume = sphere_volume(building.radius) - used;
        building.ambient_volume = ambient_volume;

        if let Some(mut fluid) = fluid_storage {
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
                id:      viewable.id,
                color:   proto::Color(storage.rgba),
                ambient: None,
            };
            match level {
                view::SubscriptionLevel::Optical => {}
                view::SubscriptionLevel::Detail => {
                    update.ambient = Some(storage.to_proto_normal(sensor));
                }
                view::SubscriptionLevel::Debug => {
                    update.ambient = Some(storage.to_proto_debug());
                }
            }
            [update.into()]
        }));
    }
}
