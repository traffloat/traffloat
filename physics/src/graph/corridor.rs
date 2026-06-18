use std::f32::consts::PI;

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::{Entity, EntityHashSet};
use bevy::ecs::message::MessageWriter;
use bevy::ecs::name::Name;
use bevy::ecs::query::With;
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{EntityCommand, Query, SystemState};
use bevy::ecs::world::{EntityWorldMut, World};
use bevy::math::{Rect, Vec2};
use bevy::reflect::Reflect;
use traffloat_proto::proto;

use crate::graph::conduit::{self, ListOnCorridor};
use crate::graph::{Building, Conduit, ViewInitSystemSets, building, edge};
use crate::util::{Alpha, AlphaBeta, Beta, EntityWorldMutExt, Which, WorldExt};
use crate::{Vector, fluid, view};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<NextCorridorId>();
        app.register_type::<Corridor>();

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
    }
}

#[derive(Component, Reflect)]
pub struct Corridor {
    pub name:               String,
    pub length:             f32,
    pub radius:             f32,
    pub wall_thickness:     f32,
    pub ambient_area:       f32,
    pub endpoint_positions: AlphaBeta<Vector>,
}

#[derive(Resource, Default, Reflect)]
struct NextCorridorId(u64);

pub struct SpawnCommand {
    pub name:               Option<String>,
    pub endpoint_positions: AlphaBeta<Vector>,
    pub length:             f32,
    pub radius:             f32,
    pub wall_thickness:     f32,
}

impl EntityCommand for SpawnCommand {
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
        entity.insert((
            Name::new(format!("Corridor {name}")),
            Corridor {
                name,
                length: self.length,
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
                volume:         ambient_area * self.length,
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
            && let Some(building_entity) =
                entity.world().log_get::<edge::OfBuilding<Ab>>(edge_entity.edge())
            && let Some(building) = entity.world().log_get::<Building>(building_entity.0)
        {
            let building_half_size = building.radius + building.wall_thickness;
            let building_rect = building.base_rect();
            rect = rect.union(building_rect);
            Some(building_entity.0)
        } else {
            None
        };

        (rect, building)
    }

    let Some(corridor) = entity.log_get::<Corridor>() else { return };
    let (rect_alpha, building_alpha) = get_endpoint_rect(&entity, corridor, Alpha);
    let (rect_beta, building_beta) = get_endpoint_rect(&entity, corridor, Beta);
    let rect = rect_alpha.union(rect_beta);
    entity.insert(view::CullingRect(rect));

    for building in [building_alpha, building_beta].into_iter().flatten() {
        entity.world_scope(|world| {
            let mut state = SystemState::<building::RecomputeCullingRectParams>::new(world);
            let params = state.get_mut(world);
            building::recompute_culling_rect(params, building);
        });
    }

    let conduits: EntityHashSet =
        entity.get::<conduit::ListOnCorridor>().iter().flat_map(|list| list.iter()).collect();
    entity.world_scope(|world| {
        for conduit in conduits {
            if let Some(mut culling_rect) = world.log_get_mut::<view::CullingRect>(conduit) {
                culling_rect.0 = rect;
            }
        }
    });
}

pub struct DespawnCommand;

impl EntityCommand for DespawnCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        view::on_viewable_despawn(&mut entity);
        entity.despawn();
    }
}

pub struct RecomputeAmbientVolume;

impl EntityCommand for RecomputeAmbientVolume {
    fn apply(self, mut entity: EntityWorldMut) {
        let used_by_conduits: f32 = if let Some(conduit_list) = entity.get::<ListOnCorridor>() {
            let conduits: Vec<_> = conduit_list.iter().collect();
            conduits
                .iter()
                .filter_map(|&f| entity.world().log_get::<Conduit>(f))
                .map(|f| f.radius * f.radius)
                .sum()
        } else {
            0.0
        };
        let Some(mut corridor) = entity.log_get_mut::<Corridor>() else { return };

        let ambient_base = corridor.radius * corridor.radius - used_by_conduits;
        let ambient_area = ambient_base * PI;
        corridor.ambient_area = ambient_area;
        let ambient_volume = ambient_area * corridor.length;

        if let Some(mut fluid) = entity.log_get_mut::<fluid::Storage>() {
            fluid.volume = ambient_volume;
        }
    }
}

fn init_viewer_system(
    corridor_query: Query<(&Corridor, &view::Viewable)>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    for (corridor, viewable) in corridor_query {
        messages.write_batch(viewable.broadcast_new(|| {
            [proto::Update::NewCorridor(proto::NewCorridor {
                id:             viewable.id,
                name:           corridor.name.clone(),
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
