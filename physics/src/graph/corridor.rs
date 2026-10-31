use std::f32::consts::PI;

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::{Entity, EntityHashSet};
use bevy::ecs::message::MessageWriter;
use bevy::ecs::name::Name;
use bevy::ecs::query::{QueryData, With};
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{EntityCommand, Query, Res, SystemParam};
use bevy::ecs::world::EntityWorldMut;
use bevy::math::{Rect, Vec2};
use bevy::reflect::Reflect;
use traffloat_proto::proto;
use traffloat_util::{
    Alpha, AlphaBeta, Beta, EntityWorldMutExt, QueryExt, Which, WorldExt, loc,
    run_stateless_closure,
};

use crate::graph::conduit::{self, ListInCorridor};
use crate::graph::{Building, Conduit, ViewInitSystemSets, building, edge};
use crate::persist::AppExt;
use crate::{Vector, cargo, fluid, resident, view};

mod persist;
pub use persist::Persist;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<NextCorridorId>();
        app.register_type::<Corridor>();

        app.register_persistable(Persist);
        app.init_resource::<NextCorridorId>();
        app.add_systems(
            app::Update,
            init_viewer_system
                .in_set(view::SendUpdatesSystemSet::Init)
                .in_set(ViewInitSystemSets::Corridor),
        );
        app.add_systems(
            app::Update,
            incr_viewer_system
                .in_set(super::ViewIncrSystemSets::Corridor)
                .in_set(view::SendUpdatesSystemSet::Incr),
        );

        app.add_systems(app::Update, recompute_ambient_volume_system);
    }
}

#[derive(Component, Reflect)]
pub struct Corridor {
    pub length:             f32,
    pub radius:             f32,
    pub wall_thickness:     f32,
    /// Mean cross-sectional area of the ambient space of the corridor.
    ///
    /// This takes into account the residents and cargo blocking ambient space,
    /// primarily used to compute the resistance of fluids advecting through the corridor.
    pub ambient_area:       f32,
    pub endpoint_positions: AlphaBeta<Vector>,
}

#[derive(Resource, Default, Reflect)]
struct NextCorridorId(u64);

pub struct SpawnCommand {
    pub name:               Option<String>,
    pub endpoint_positions: AlphaBeta<Vector>,
    pub radius:             f32,
    pub wall_thickness:     f32,
}

impl EntityCommand for SpawnCommand {
    type Out = ();
    fn apply(self, mut entity: EntityWorldMut) {
        let name = self.name.unwrap_or_else(|| {
            entity.world_scope(|world| {
                let mut next = world.resource_mut::<NextCorridorId>();
                let out = next.0;
                next.0 += 1;
                format!("#{out}")
            })
        });
        let ambient_area = self.radius * self.radius * PI;
        let length = self.endpoint_positions.net_diff().length();
        entity.insert((
            Name::new(format!("Corridor {name}")),
            view::Named { name },
            Corridor {
                length,
                radius: self.radius,
                wall_thickness: self.wall_thickness,
                ambient_area,
                endpoint_positions: self.endpoint_positions,
            },
        ));
        entity.reborrow_scope(|entity| view::AddViewableCommand.apply(entity));

        // ambient conduit
        entity.reborrow_scope(|entity| {
            fluid::AddStorageCommand {
                volume:         ambient_area * length,
                optical_length: self.radius,
            }
            .apply(entity);
        });

        recompute_culling_rect(entity);
    }
}

fn recompute_culling_rect(mut entity: EntityWorldMut) {
    fn get_endpoint_rect<Ab: Which>(
        entity: &EntityWorldMut,
        corridor: &Corridor,
        which: Ab,
    ) -> (Rect, Option<Entity>) {
        let pos = which.select(corridor.endpoint_positions);
        let half_size = corridor.radius + corridor.wall_thickness;
        let mut rect = Rect::from_center_half_size(pos, Vec2::splat(half_size));

        let building = if let Some(edge_entity) = entity.get::<edge::CorridorEdge<Ab>>()
            && let Some(of_building) =
                entity.world().log_get::<edge::OfBuilding<Ab>>(edge_entity.edge(), loc!())
            && let Some(building) = entity.world().log_get::<Building>(of_building.building, loc!())
        {
            let building_rect = building.base_rect();
            rect = rect.union(building_rect);
            Some(of_building.building)
        } else {
            None
        };

        (rect, building)
    }

    let Some(corridor) = entity.log_get::<Corridor>(loc!()) else { return };
    let (rect_alpha, building_alpha) = get_endpoint_rect(&entity, corridor, Alpha);
    let (rect_beta, building_beta) = get_endpoint_rect(&entity, corridor, Beta);
    let rect = rect_alpha.union(rect_beta);
    entity.insert(view::CullingRect(rect));

    for building in [building_alpha, building_beta].into_iter().flatten() {
        entity.world_scope(|world| {
            run_stateless_closure(world, move |params: building::RecomputeCullingRectParams| {
                building::recompute_culling_rect(params, building);
            });
        });
    }

    let conduits: EntityHashSet =
        entity.get::<conduit::ListInCorridor>().iter().flat_map(|list| list.iter()).collect();
    entity.world_scope(|world| {
        for conduit in conduits {
            if let Some(mut culling_rect) = world.log_get_mut::<view::CullingRect>(conduit, loc!())
            {
                culling_rect.0 = rect;
            }
        }
    });
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
    conduit_query:   Query<'w, 's, &'static Conduit>,
    resident_niches: Res<'w, resident::attr::Niches>,
    resident_query:  Query<'w, 's, &'static resident::Attributes>,
    cargo_types:     Res<'w, cargo::Types>,
}

#[derive(QueryData)]
struct RecomputeAmbientVolumeData {
    conduits:  Option<&'static conduit::ListInCorridor>,
    residents: Option<&'static resident::ListInAmbientFixture>,
    cargo:     Option<&'static cargo::Storage>,
}

fn recompute_ambient_volume_system(
    corridor_query: Query<(&mut Corridor, RecomputeAmbientVolumeData, Option<&mut fluid::Storage>)>,
    ctx: RecomputeAmbientVolumeContextParams,
) {
    fn area_used_by_conduits_before_pi(
        data: &RecomputeAmbientVolumeDataItem,
        ctx: &RecomputeAmbientVolumeContextParams,
    ) -> f32 {
        let Some(list) = data.conduits else { return 0.0 };
        list.iter()
            .filter_map(|c| ctx.conduit_query.log_get(c, loc!()))
            .map(|c| c.radius * c.radius)
            .sum()
    }

    fn used_by_residents(
        data: &RecomputeAmbientVolumeDataItem,
        ctx: &RecomputeAmbientVolumeContextParams,
    ) -> f32 {
        let Some(list) = data.residents else { return 0.0 };
        let Some(volume_attr) = ctx.resident_niches.niches[resident::attr::Niche::Volume] else {
            return 0.0;
        };
        list.iter()
            .filter_map(|r| ctx.resident_query.log_get(r, loc!()))
            .map(|attrs| attrs.get(volume_attr))
            .sum()
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "precision loss is acceptable after conversion to volume"
    )]
    fn used_by_cargo(
        data: &RecomputeAmbientVolumeDataItem,
        ctx: &RecomputeAmbientVolumeContextParams,
    ) -> f32 {
        let Some(storage) = data.cargo else { return 0.0 };
        storage
            .types()
            .map(|t| ctx.cargo_types.get(t.ty).granule_volume * (t.granules.0 as f32))
            .sum()
    }

    // This loop is perfectly parallelizable, but the task is largely memory-bound,
    // so parallelization would only be more likely to increase L3 cache misses.
    for (mut corridor, data, fluid_storage) in corridor_query {
        let ambient_area_before_pi =
            corridor.radius * corridor.radius - area_used_by_conduits_before_pi(&data, &ctx);
        let mut ambient_volume = ambient_area_before_pi * PI * corridor.length;
        ambient_volume -= used_by_residents(&data, &ctx);
        ambient_volume -= used_by_cargo(&data, &ctx);

        corridor.ambient_area = ambient_volume / corridor.length;

        if let Some(mut fluid) = fluid_storage {
            fluid.volume = ambient_volume;
        }
    }
}

fn init_viewer_system(
    corridor_query: Query<(&Corridor, &view::Named, &view::Viewable)>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    for (corridor, named, viewable) in corridor_query {
        messages.write_batch(viewable.broadcast_new(|| {
            [proto::Update::NewCorridor(proto::NewCorridor {
                id:             viewable.id,
                name:           named.name.clone(),
                alpha_position: corridor.endpoint_positions.alpha,
                beta_position:  corridor.endpoint_positions.beta,
                radius:         corridor.radius,
                wall_thickness: corridor.wall_thickness,
            })]
        }));
    }
}

fn incr_viewer_system(
    mut throttle: view::BroadcastThrottle,
    corridor_query: Query<(&view::Viewable, &fluid::Storage, &fluid::Sensor), With<Corridor>>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    if !throttle.should_run() {
        return;
    }

    for (viewable, storage, sensor) in corridor_query.iter() {
        messages.write_batch(viewable.broadcast_update(|level| {
            let mut update = proto::UpdateCorridor {
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
