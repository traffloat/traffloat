//! Vanilla building definitions.
//!
//! # Adding a new building
//! 1. Open `vanilla/src/building.rs`.
//! 2. Navigate to the `buildings! { ... }` block.
//! 3. Each top-level block is in the format `snake_case_category "Real category name" ("Category description") { ...
//!    }`. Find an appropriate category or add a new category for the new building.
//! 4. Add a `snake_case` identifier for the building, then curly braces with the data fields
//!     Every field is in the form `property: value,`. Do not forget to add the trailing comma.
//! 5. Add the texture of the building to a new directory in `client/textures`.
//!
//! ## Data fields
//! The following data fields are required in `vanilla/src/building.rs`:
//! - `name`: The name of the building.
//! - `summary`: A one-line short description of the building.
//! - `description`: The full description in markdown format.
//! - Shape of the building
//!     - If the building is a cube, write `cube` with the half-length of each side.
//!     - If the building is a cuboid, write `cuboid` with the two corners,
//!         e.g. `cuboid: [-1., -2. -3.] .. [4., 5., 6.]` means that
//!         the building is a cuboid with two opposite corners at (-1, -2, -3) and (4, 5, 6)
//!         respectively.
//! - `texture`: The directory name of the building texture.
//!     Conventionally, this name should be the `kebab-case` of the building.
//! - `reactions`: The list of reactions supported by the building.
//!     The list is comma-separated and surrounded by a pair of `[]`.
//!     See the documentation on [reactions][super::reaction] for adding new reaction types.
//!     Each reaction is in the format `reaction_name { ... }`,
//!     where `...` are building-specific options on the reaction.
//!     See [`building::ReactionPolicy`] for possible options.
//! - `hitpoint`: The full hitpoints of the building type.
//! - `storage`: The maximum amount of cargo, liquid and gas stored in the building,
//!     in the format `{cargo: 1000., liquid: 2000., gas: 3000.,}`.
//!     - The storage is used as a temporary buffer for reaction input/output,
//!         liquid transfer and gas diffusion.
//!         The storage restricts the maximum amount of liquid and gas
//!         that passes through the building per second.
//!         Therefore, the storage size should be larger than
//!         the total factory input/output per second,
//!         and reasonably much larger than the amount of liquid/gas passing through per second,
//!         otherwise the building would be the bottleneck for transfer.
//!     - The gas storage is used to buffer oxygen that inhabitants breathe.
//!         If the gas storage is too small, inhabitants without oxygen bottles may suffocate.
//! - `features`: A list of extra features (in addition to reactions) supported by the building,
//!     separated by comma and surrounded by `[]`.
//!     See [`building::ExtraFeature`] for possible options.
//!
//! ## Texture
//! ### Cube/Cuboid
//! The texture directory contains 6 files: `xp.svg`, `xn.svg`, `yp.svg`, `yn.svg`, `zp.svg`,
//! `zn.svg`.
//! Each file represents the shape of the building in the +X/-X/+Y/-Y/+Z/-Z direction.
//! While the dimension is not constrained,
//! each file is rescaled to different sprites of
//! 16&times;16, 64&times;64, 256&times;256 and 1024&times;1024 pixels
//! during the texture preprocessing phase.
//! It is fine to create an SVG file with a rectangular shape;
//! it will be stretched to a square shape during preprocessing phase,
//! then compressed back to a rectangle when it is rendered.

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

    storage "Storage" ("Buildings to store resources") {
        vault {
            name: "Vault",
            summary: "Store lots of cargo",
            description: "Large amounts of cargo can be stored here for future use. \
                This can also be used as a temporary junkyard \
                if junk launchers cannot catch up quickly enough.",
            cube: 1.,
            texture: "vault",
            reactions: [],
            hitpoint: 500.,
            storage: {
                cargo: 20000.,
                liquid: 1000.,
                gas: 1000.,
            },
            features: [],
        }
    }

    industrial "Industrial" ("Buildings for processing intermediates") {
        filtration_plant {
            name: "Filtration plant",
            summary: "Filter insoluble impurities from water",
            description: "Asteroidal water is often mixed with different sorts of junk. \
                These solid impurities will clog and damage conduits if passed directly. \
                A filtration plant can separate the solid junk from the water.
\n\
                Although filtered water is still not edible for inhabitants, \
                it demonstrates most of the physical properties of water, \
                such as the high heat capacity and low viscosity. \
                Therefore, filtered water is sufficient for most industrial uses.",
                cube: 1.,
                texture: "filtration-plant",
                reactions: [
                    asteroidal_water_filtration {configurable: true},
                ],
                hitpoint: 150.,
                storage: {
                    cargo: 3000.,
                    liquid: 5000.,
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
