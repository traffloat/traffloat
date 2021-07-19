//! Vanilla building definitions.

use crate::VANILLA_TEXTURE;
use traffloat_types::def::{building, GameDefinition};
use traffloat_types::space;

macro_rules! buildings {
    (
        $($category_ident:ident $category:literal ($category_description:literal) {
            $($ident:ident {
                name: $name:literal,
                summary: $summary:literal,
                description: $description:literal,
                $(cube: $cube_size:literal,)?
                $(cuboid: [
                    $cuboid_x1:literal,
                    $cuboid_y1:literal,
                    $cuboid_z1:literal $(,)?
                ] .. [
                    $cuboid_x2:literal,
                    $cuboid_y2:literal,
                    $cuboid_z2:literal $(,)?
                ],)?
                texture: $texture:literal,
                reactions: [
                    $(
                        $reaction_name:ident {
                            $($reaction_param:ident: $reaction_value:expr,)*
                        },
                    )*
                ],
                features: [$($features:expr),* $(,)?],
            })*
        })*
    ) => {
        /// IDs assigned to the vanilla game definition.
        pub struct Ids {
            $(
                $(
                    pub $ident: building::TypeId,
                )*
            )*
        }

        /// Populates a [`GameDefinition`] with building definition.
        pub fn populate(def: &mut GameDefinition, reactions: &super::reaction::Ids) -> Ids {
            $(
                let $category_ident = def.add_building_category(
                    building::Category::builder()
                        .title(String::from($category))
                        .description(String::from($category_description))
                        .build()
                );
                $(
                    let $ident = def.add_building(
                        building::Type::builder()
                            .name(String::from($name))
                            .summary(String::from($summary))
                            .description(String::from($description))
                            .shape(building::Shape::builder()
                                $(
                                    .transform(space::Matrix::new_scaling($cube_size))
                                )?
                                $(
                                    .transform(space::transform_cuboid(
                                        space::Vector::new($cuboid_x1, $cuboid_y1, $cuboid_z1),
                                        space::Vector::new($cuboid_x2, $cuboid_y2, $cuboid_z2),
                                    ))
                                )?
                                .texture_src(String::from(VANILLA_TEXTURE))
                                .texture_name(String::from($texture))
                                .build())
                            .category($category_ident)
                            .reactions(vec![
                                $(
                                    (
                                        reactions.$reaction_name,
                                        building::ReactionPolicy::builder()
                                            $(
                                                .$reaction_param($reaction_value)
                                            )*
                                            .build(),
                                    ),
                                )*
                            ])
                            .features({
                                #[allow(unused_imports)]
                                use traffloat_types::def::building::ExtraFeature::*;
                                vec![
                                    $($features,)*
                                ]
                            })
                            .build()
                    );
                )*
            )*

            Ids {
                $(
                    $($ident,)*
                )*
            }
        }
    }
}

buildings! {
    population "Population" ("Buildings to support inhabitant population") {
        core {
            name: "Core",
            summary: "The center of the whole colony",
            description: "The core is the ultimate building to protect. \
                It provides basic resources, including a small amount of uninterrupted power, \
                some oxygen generation and a few population housing. \
                Destruction of the core ends the game.",
            cube: 1.,
            texture: "core",
            reactions: [],
            features: [Core],
        }
        hut {
            name: "Hut",
            summary: "A small living quarter.",
            description: "",
            cube: 1.,
            texture: "house",
            reactions: [],
            features: [ProvidesHousing(6)],
        }
    }

    electricity "Electricity" ("Buildings to manage electricity") {
        solar_panel {
            name: "Solar panel",
            summary: "Basic power production",
            description: "",
            cube: 1.,
            texture: "solar-panel",
            reactions: [
                solar_power {},
            ],
            features: [],
        }
    }
}