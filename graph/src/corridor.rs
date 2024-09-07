//! A link between two buildings.

use bevy::app::{self, App};
use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::ecs::world::World;
use bevy::hierarchy::BuildWorldChildren;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use traffloat_base::save;
use typed_builder::TypedBuilder;

use crate::building;

mod endpoint;
pub use endpoint::{Binary, Endpoint};

pub mod duct;

/// Maintain corridors.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) { save::add_def::<Save>(app); }
}

/// Components for a corridor.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    endpoints: Endpoints,
    duct_list: DuctList,
    #[builder(default, setter(skip))]
    _marker:   Marker,
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
    pub endpoints: Binary<save::Id<building::Save>>,
}

impl save::Def for Save {
    const TYPE: &'static str = "traffloat.save.Corridor";

    type Runtime = Entity;

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(
            mut writer: save::Writer<Save>,
            (building_dep,): (save::StoreDepend<building::Save>,),
            query: Query<(Entity, &Endpoints), With<Marker>>,
        ) {
            writer.write_all(query.iter().map(|(entity, endpoints)| {
                (
                    entity,
                    Save {
                        endpoints: endpoints
                            .endpoints
                            .map(|endpoint| building_dep.must_get(endpoint)),
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

            let mut corridor = world.spawn(
                Bundle::builder()
                    .endpoints(Endpoints {
                        endpoints: def.endpoints.try_map(|endpoint| building_dep.get(endpoint))?,
                    })
                    .duct_list(DuctList { duct_list: Vec::new(), ambient })
                    .build(),
            );
            corridor.add_child(ambient);

            // TODO validate that ambient duct is going to be populated

            Ok(corridor.id())
        }

        save::LoadFn::new(loader)
    }
}
