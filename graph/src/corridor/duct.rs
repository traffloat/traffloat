//! An internal structure of a corridor.

use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::ecs::world::World;
use bevy::hierarchy::{self, BuildWorldChildren};
use serde::{Deserialize, Serialize};
use traffloat_base::save;
use typed_builder::TypedBuilder;

/// Components for a facility.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    #[builder(default, setter(skip))]
    _marker: Marker,
}

/// Marks an entity as a building.
#[derive(Component, Default)]
pub struct Marker;

/// Save schema.
#[derive(Serialize, Deserialize)]
pub struct Save {
    /// Reference to parent building.
    pub parent:     save::Id<super::Save>,
    /// Whether the facility is the ambient facility of its parent building.
    pub is_ambient: bool,
}

impl save::Def for Save {
    const TYPE: &'static str = "traffloat.save.Duct";

    type Runtime = Entity;

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(
            mut writer: save::Writer<Save>,
            (corridor_dep,): (save::StoreDepend<super::Save>,),
            (query, corridor_query): (
                Query<(Entity, &hierarchy::Parent), With<Marker>>,
                Query<&super::DuctList, With<super::Marker>>,
            ),
        ) {
            writer.write_all(query.iter().map(|(entity, parent)| {
                (
                    entity,
                    Save {
                        parent:     corridor_dep.must_get(parent.get()),
                        is_ambient: corridor_query
                            .get(parent.get())
                            .expect("dangling parent building reference")
                            .ambient
                            == entity,
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
            (building_dep,): &(save::LoadDepend<super::Save>,),
        ) -> anyhow::Result<Entity> {
            let parent = building_dep.get(def.parent)?;
            let facility_bundle = Bundle::builder().build();

            let id = if def.is_ambient {
                let id = {
                    let list = world
                        .get::<super::DuctList>(parent)
                        .expect("parent building was created in the previous load step");
                    list.ambient
                };
                let mut facility = world.entity_mut(id);
                facility.insert(facility_bundle);

                id
            } else {
                let id = {
                    let mut facility = world.spawn(facility_bundle);
                    facility.set_parent(parent);
                    facility.id()
                };

                let mut list = world
                    .get_mut::<super::DuctList>(parent)
                    .expect("parent building was created in the previous load step");
                list.duct_list.push(id);

                id
            };

            Ok(id)
        }

        save::LoadFn::new(loader)
    }
}
