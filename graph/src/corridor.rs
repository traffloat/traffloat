//! A link between two buildings.

use bevy::app::{self, App};
use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::ecs::world::World;
use bevy::hierarchy::BuildWorldChildren;
use bevy::math::{Quat, Vec3};
use bevy::transform::components::Transform;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use traffloat_base::{debug, save};
use traffloat_view::{viewable, Appearance};
use typed_builder::TypedBuilder;

use crate::building;

mod endpoint;
pub use endpoint::{Binary, Endpoint};

pub mod duct;

/// Maintain corridors.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        save::add_def::<Save>(app);
        save::add_def::<duct::Save>(app);
    }
}

/// Components for a corridor.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    endpoints: Endpoints,
    radius:    Radius,
    duct_list: DuctList,
    viewable:  viewable::StationaryBundle,
    #[builder(default, setter(skip))]
    _marker:   Marker,
    #[builder(default = debug::Bundle::new("Corridor"))]
    _debug:    debug::Bundle,
}

/// Marks an entity as a as a corridor.
#[derive(Component, Default)]
pub struct Marker;

/// The endpoint buildings of a corridor.
#[derive(Component)]
pub struct Endpoints {
    /// Endpoint buildings.
    pub endpoints: Binary<Entity>,
}

/// The radius of a corridor.
#[derive(Component)]
pub struct Radius {
    radius: f32,
}

/// List of ducts in a corridor.
#[derive(Component)]
pub struct DuctList {
    /// Non-ambient ducts in this corridor.
    /// The order of entities in this list has no significance.
    pub duct_list: Vec<Entity>,

    /// The ambient duct for this corridor.
    pub ambient: Entity,
}

/// Save schema.
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Save {
    /// Endpoint buildings of the corridor.
    pub endpoints:  Binary<save::Id<building::Save>>,
    /// Radius of the corridor.
    pub radius:     f32,
    /// Appearance of the corridor.
    pub appearance: Appearance,
}

impl save::Def for Save {
    const TYPE: &'static str = "traffloat.save.Corridor";

    type Runtime = Entity;

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(
            mut writer: save::Writer<Save>,
            (building_dep,): (save::StoreDepend<building::Save>,),
            query: Query<(Entity, &Endpoints, &Appearance, &Radius), With<Marker>>,
        ) {
            writer.write_all(query.iter().map(|(entity, endpoints, appearance, radius)| {
                (
                    entity,
                    Save {
                        endpoints:  endpoints
                            .endpoints
                            .map(|endpoint| building_dep.must_get(endpoint)),
                        radius:     radius.radius,
                        appearance: appearance.clone(),
                    },
                )
            }));
        }

        save::StoreSystemFn::new(store_system)
    }

    fn loader() -> impl save::LoadOnce<Def = Self> {
        #[allow(clippy::trivially_copy_pass_by_ref, clippy::unnecessary_wraps)]
        fn loader(
            world: &mut World,
            def: Save,
            (building_dep,): &(save::LoadDepend<building::Save>,),
        ) -> anyhow::Result<Entity> {
            let ambient = world.spawn_empty().id();
            let sid = viewable::next_sid(world);

            let endpoints = def.endpoints.try_map(|endpoint| building_dep.get(endpoint))?;
            let (endpoint_positions, endpoint_scales) = endpoints
                .query_world::<&Transform>(world)
                .expect("entities exist as buildings in dep cache")
                .map(|transform| (transform.translation, transform.scale))
                .unzip();
            let ab_vector = endpoint_positions.beta - endpoint_positions.alpha;
            let ab_direction = ab_vector.normalize();

            // TODO come up with a better algorithm for detecting the scale
            let alpha_edge = endpoint_positions.alpha + ab_direction * endpoint_scales.alpha.x;
            let beta_edge = endpoint_positions.beta - ab_direction * endpoint_scales.beta.x;

            let mut corridor = world.spawn(
                Bundle::builder()
                    .endpoints(Endpoints { endpoints })
                    .radius(Radius { radius: def.radius })
                    .duct_list(DuctList { duct_list: Vec::new(), ambient })
                    .viewable(
                        viewable::StationaryBundle::builder()
                            .base(
                                viewable::BaseBundle::builder()
                                    .sid(sid)
                                    .appearance(def.appearance)
                                    .build(),
                            )
                            .transform(Transform {
                                translation: (alpha_edge + beta_edge) / 2.,
                                rotation:    Quat::from_rotation_arc(Vec3::Z, ab_direction),
                                scale:       Vec3 {
                                    x: def.radius,
                                    y: def.radius,
                                    z: alpha_edge.distance(beta_edge) / 2.,
                                },
                            })
                            .build(),
                    )
                    .build(),
            );
            corridor.add_child(ambient);

            // TODO validate that ambient duct is going to be populated

            Ok(corridor.id())
        }

        save::LoadFn::new(loader)
    }
}
