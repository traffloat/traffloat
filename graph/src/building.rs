//! A building in which facilities can be installed.

use bevy::app::{self, App};
use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::ecs::world::World;
use bevy::hierarchy::BuildWorldChildren;
use bevy::transform::components::Transform;
use serde::{Deserialize, Serialize};
use traffloat_base::{proto, save};
use typed_builder::TypedBuilder;

pub mod facility;

/// Maintain buildings.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) { save::add_def::<Save>(app); }
}

/// Components for a building.
#[derive(bundle::Bundle, TypedBuilder)]
#[allow(missing_docs)]
pub struct Bundle {
    position:      Transform,
    facility_list: FacilityList,
}

/// List of facilities in a building.
#[derive(Component)]
pub struct FacilityList {
    /// Non-ambient facilities in this building.
    /// The order of entities in this list has no significance.
    pub facilities: Vec<Entity>, // entities with facility components

    /// The ambient space for this building.
    pub ambient: Entity,
}

/// Protobuf structure for saves.
#[derive(Serialize, Deserialize)]
pub struct Save {
    /// Position of a building.
    pub position: proto::Position,
}

impl save::Def for Save {
    const TYPE: &'static str = "traffloat.save.Building";

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(
            mut writer: save::Writer<Save>,
            (): (),
            query: Query<(Entity, &Transform), With<FacilityList>>,
        ) {
            writer.write_all(query.iter().map(|(entity, transform)| {
                (entity, Save { position: transform.translation.into() })
            }));
        }

        save::StoreSystemFn::new(store_system)
    }

    fn loader() -> impl save::LoadOnce<Def = Self> {
        #[allow(clippy::trivially_copy_pass_by_ref, clippy::unnecessary_wraps)]
        fn loader(world: &mut World, def: Save, (): &()) -> anyhow::Result<Entity> {
            let ambient = world.spawn_empty().id();

            let mut building = world.spawn(
                Bundle::builder()
                    .position(Transform::from_translation(def.position.into()))
                    .facility_list(FacilityList { facilities: Vec::new(), ambient })
                    .build(),
            );
            building.add_child(ambient);

            // TODO validate that ambient facility is going to be populated

            Ok(building.id())
        }

        save::LoadFn::new(loader)
    }
}
