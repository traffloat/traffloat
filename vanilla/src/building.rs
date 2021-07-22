//! Vanilla building definitions.

use arcstr::literal;

use crate::VANILLA_TEXTURE;
use traffloat_types::def::{building, GameDefinition};
use traffloat_types::{space, units};

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
                            $($reaction_param:ident: $reaction_value:expr),* $(,)?
                        },
                    )*
                ],
                hitpoint: $hitpoint:literal,
                storage: {
                    cargo: $cargo_storage:literal,
                    liquid: $liquid_storage:literal,
                    gas: $gas_storage:literal,
                },
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
                        .title(literal!($category))
                        .description(literal!($category_description))
                        .build()
                );
                $(
                    let $ident = def.add_building(
                        building::Type::builder()
                            .name(literal!($name))
                            .summary(literal!($summary))
                            .description(literal!($description))
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
                                .texture_src(literal!(VANILLA_TEXTURE))
                                .texture_name(literal!($texture))
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
                            .hitpoint($hitpoint.into())
                            .storage(building::Storage::builder()
                                .cargo($cargo_storage.into())
                                .liquid($liquid_storage.into())
                                .gas($gas_storage.into())
                                .build())
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
            hitpoint: 3000.,
            storage: {
                cargo: 1000.,
                liquid: 1000.,
                gas: 1000.,
            },
            features: [Core, ProvidesHousing(4)],
        }
        hut {
            name: "Hut",
            summary: "A small living quarter.",
            description: "",
            cube: 1.,
            texture: "house",
            reactions: [],
            hitpoint: 150.,
            storage: {
                cargo: 1000.,
                liquid: 1000.,
                gas: 1000.,
            },
            features: [ProvidesHousing(3)],
        }
    }

    traffic "Traffic" ("Buildings for traffic control") {
        terminal {
            name: "Terminal",
            summary: "Connects and drives corridors",
            description: "",
            cube: 0.2,
            texture: "terminal",
            reactions: [],
            hitpoint: 100.,
            storage: {
                cargo: 1000.,
                liquid: 1000.,
                gas: 1000.,
            },
            features: [
                RailTerminal(units::RailForce(100.)),
                LiquidPump(units::PipeForce(100.)),
                GasPump(units::FanForce(100.)),
            ],
        }
    }

    electricity "Electricity" ("Buildings to manage electricity") {
        solar_panel {
            name: "Solar panel",
            summary: "Basic power production",
            description: "Solar power is an effective power generation mechanism. \
                Unobstructed by planetary atmosphere, \
                solar panels are the primary source of power in early to mid game.",
            cube: 1.,
            texture: "solar-panel",
            reactions: [
                solar_power {configurable: true},
            ],
            hitpoint: 200.,
            storage: {
                cargo: 1000.,
                liquid: 1000.,
                gas: 1000.,
            },
            features: [],
        }
    }

    education "Education" ("Buildings to train inhabitant skills") {
        driving_school {
            name: "Driving school",
            summary: "Train inhabitants to drive better",
            description: "Train inhabitants to drive better.",
            cube: 1.,
            texture: "driving-school",
            reactions: [
                driving_lesson {},
            ],
            hitpoint: 150.,
            storage: {
                cargo: 1000.,
                liquid: 1000.,
                gas: 1000.,
            },
            features: [],
        }
    }

    entertainment "Entertainment" ("Buildings to restore happiness") {
    }

    security "Security" ("Buildings related to crimes") {
        prison {
            name: "Prison",
            summary: "Correctional services for criminals",
            description: "Inhabitants with negative happiness are imprisoned here \
                to recultivate morality and restore happiness.",
            cube: 1.,
            texture: "prison",
            reactions: [
                imprisonment {},
            ],
            hitpoint: 200.,
            storage: {
                cargo: 1000.,
                liquid: 1000.,
                gas: 1000.,
            },
            features: [
                SecureExit {
                    min_happiness: 10f64.into(),
                    breach_probability: 0.001,
                },
            ],
        }
        customs {
            name: "Customs",
            summary: "Disallow outlaws from intrusion",
            description: "Customs is a security checkpoint at which \
                operators check all inhabitants passing through. \
                Surround important buildings with customs \
                so that outlaws cannot grief them.",
            cube: 1.,
            texture: "customs",
            reactions: [],
            hitpoint: 300.,
            storage: {
                cargo: 1000.,
                liquid: 1000.,
                gas: 1000.,
            },
            features: [
                SecureEntry {
                    min_happiness: 10f64.into(),
                    breach_probability: 0.005,
                },
            ],
        }
    }
}
