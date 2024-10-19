//! A building in which facilities can be installed.

use std::iter;

use bevy::app::{self, App};
use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::ecs::world::World;
use bevy::hierarchy::BuildWorldChildren;
use bevy::transform::components::Transform;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use traffloat_base::{debug, proto, save};
use traffloat_view::{appearance, viewable};
use typed_builder::TypedBuilder;

pub mod facility;

/// Maintain buildings.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        save::add_def::<Save>(app);
        save::add_def::<facility::Save>(app);
    }
}

/// Components for a building.
#[derive(bundle::Bundle, TypedBuilder)]
#[allow(missing_docs)]
pub struct Bundle {
    viewable:      viewable::StationaryBundle,
    facility_list: FacilityList,
    #[builder(default, setter(skip))]
    _marker:       Marker,
    #[builder(default = debug::Bundle::new("Building"))]
    _debug:        debug::Bundle,
}

/// Marks an entity as a building.
#[derive(Component, Default)]
pub struct Marker;

/// List of facilities in a building.
#[derive(Component)]
pub struct FacilityList {
    /// The ambient space for this building.
    pub ambient: Entity,

    /// Non-ambient facilities in this building.
    /// The order of entities in this list has no significance.
    pub non_ambient: Vec<Entity>, // entities with facility components
}

impl FacilityList {
    /// Iterates through all facilities of the building, including the ambient facility.
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        iter::once(&self.ambient).chain(&self.non_ambient).copied()
    }
}

/// Save schema.
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Save {
    /// Position of the building.
    pub transform:  proto::Transform,
    /// Appearance of the building.
    pub appearance: appearance::Appearance,
}

impl save::Def for Save {
    const TYPE: &'static str = "traffloat.save.Building";

    type Runtime = Entity;

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(
            mut writer: save::Writer<Save>,
            (): (),
            query: Query<(Entity, &Transform, &appearance::Appearance), With<Marker>>,
        ) {
            writer.write_all(query.iter().map(|(entity, &transform, appearance)| {
                (entity, Save { transform: transform.into(), appearance: appearance.clone() })
            }));
        }

        save::StoreSystemFn::new(store_system)
    }

    fn loader() -> impl save::LoadOnce<Def = Self> {
        #[allow(clippy::trivially_copy_pass_by_ref, clippy::unnecessary_wraps)]
        fn loader(world: &mut World, def: Save, (): &()) -> anyhow::Result<Entity> {
            let ambient = world.spawn_empty().id();

            let sid = viewable::next_sid(world);
            let mut building = world.spawn(
                Bundle::builder()
                    .viewable(
                        viewable::StationaryBundle::builder()
                            .base(
                                viewable::BaseBundle::builder()
                                    .sid(sid)
                                    .appearance(def.appearance)
                                    .build(),
                            )
                            .transform(def.transform.into())
                            .build(),
                    )
                    .facility_list(FacilityList { non_ambient: Vec::new(), ambient })
                    .build(),
            );
            building.add_child(ambient);

            // TODO validate that ambient facility is going to be populated

            Ok(building.id())
        }

        save::LoadFn::new(loader)
    }
}
