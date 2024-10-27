//! An internal structure of a corridor.

use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::ecs::world::World;
use bevy::hierarchy::{self, BuildWorldChildren};
use bevy::math::{Quat, Vec3};
use bevy::transform::components::Transform;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use traffloat_base::{debug, proto, save};
use traffloat_view::{viewable, Appearance};
use typed_builder::TypedBuilder;

/// Components for a facility.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    viewable: viewable::StationaryChildBundle,
    #[builder(default, setter(skip))]
    _marker:  Marker,
    #[builder(default = debug::Bundle::new("Duct"))]
    _debug:   debug::Bundle,
}

/// Marks an entity as a duct.
#[derive(Component, Default)]
pub struct Marker;

/// Radius of the duct in absolute scale.
#[derive(Component)]
pub struct Radius {
    radius: f32,
}

/// Save schema.
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Save {
    /// Reference to parent building.
    pub parent:     save::Id<super::Save>,
    /// Appearance of the duct.
    pub appearance: Appearance,
    /// Settings specific to the ambientness of the duct.
    pub ambient:    SaveAmbient,
}

/// Settings specific to the ambientness of a duct.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(tag = "shape")]
pub enum SaveAmbient {
    /// The duct is the ambient duct of the corridor.
    Ambient,
    /// The duct is not the ambient duct; it has its own shape.
    Cylindrical {
        /// Radius of the duct in absolute scale.
        radius: f32,
        /// Position of the duct relative to the corridor center.
        inner:  proto::PlanePosition,
    },
}

impl save::Def for Save {
    const TYPE: &'static str = "traffloat.save.Duct";

    type Runtime = Entity;

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(
            mut writer: save::Writer<Save>,
            (corridor_dep,): (save::StoreDepend<super::Save>,),
            (query, corridor_query): (
                Query<(Entity, &hierarchy::Parent, &Transform, &Appearance, &Radius), With<Marker>>,
                Query<&super::DuctList, With<super::Marker>>,
            ),
        ) {
            writer.write_all(query.iter().map(
                |(entity, parent, &transform, appearance, radius)| {
                    (
                        entity,
                        Save {
                            parent:     corridor_dep.must_get(parent.get()),
                            appearance: appearance.clone(),
                            ambient:    if corridor_query
                                .get(parent.get())
                                .expect("dangling parent building reference")
                                .ambient
                                == entity
                            {
                                SaveAmbient::Ambient
                            } else {
                                SaveAmbient::Cylindrical {
                                    radius: radius.radius,
                                    inner:  proto::PlanePosition {
                                        x: transform.translation.x,
                                        y: transform.translation.y,
                                    },
                                }
                            },
                        },
                    )
                },
            ));
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
            let sid = viewable::next_sid(world);
            let parent = building_dep.get(def.parent)?;

            let &super::Radius { radius: corridor_radius } =
                world.get(parent).expect("entity exists as corridor in dep cache");
            let (is_ambient, duct_radius, inner_translation) = match def.ambient {
                SaveAmbient::Ambient => (true, corridor_radius, Vec3::ZERO),
                SaveAmbient::Cylindrical { radius, inner } => {
                    (false, radius, Vec3::new(inner.x, inner.y, 0.))
                }
            };
            let relative_radius_scale = duct_radius / corridor_radius;

            let facility_bundle = Bundle::builder()
                .viewable(
                    viewable::StationaryChildBundle::builder()
                        .base(
                            viewable::BaseBundle::builder()
                                .sid(sid)
                                .appearance(def.appearance)
                                .build(),
                        )
                        .inner_transform(Transform {
                            translation: inner_translation,
                            rotation:    Quat::IDENTITY,
                            scale:       Vec3::new(
                                relative_radius_scale,
                                relative_radius_scale,
                                1.,
                            ),
                        })
                        .build(),
                )
                .build();

            let id = if is_ambient {
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
