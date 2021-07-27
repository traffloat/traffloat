//! Vanilla mechanism definitions.

use arcstr::literal;
use smallvec::smallvec;

use traffloat_types::def::{reaction, GameDefinition};
use traffloat_types::time::Rate;

macro_rules! reactions {
    (
        $cargo:ident $liquid:ident $gas:ident $skill:ident; // macro hygiene hack
        $($category_ident:ident $category:literal ($category_description:literal) {
            $($ident:ident {
                name: $name:literal,
                description: $description:literal,
                catalysts: [$(
                    $catalyst_variant:ident {
                        $(ty: $catalyst_type:expr,)?
                        levels: $catalyst_min:literal .. $catalyst_max:literal,
                        multipliers: [
                            $catalyst_underflow_mul:literal,
                            $catalyst_min_mul:literal,
                            $catalyst_max_mul:literal,
                            $catalyst_overflow_mul:literal
                        ],
                    },
                )*],
                puts: [$(
                    $put_variant:ident {
                        $(ty: $put_type:expr,)?
                        rate: $put_rate:literal,
                    },
                )*],
            })*
        })*
    ) => {
        /// IDs assigned to the vanilla game definition.
        pub struct Ids {
            $(
                $(
                    pub $ident: reaction::TypeId,
                )*
            )*
        }

        /// Populates a [`GameDefinition`] with cargo definition.
        #[allow(unused_variables)]
        pub fn populate(def: &mut GameDefinition, $cargo: &super::cargo::Ids, $liquid: &super::liquid::Ids, $gas: &super::gas::Ids, $skill: &super::skill::Ids) -> Ids {
            $(
                let $category_ident = def.add_reaction_category(
                    reaction::Category::builder()
                        .title(literal!($category))
                        .description(literal!($category_description))
                        .build()
                );
                $(
                    let $ident = def.add_reaction(
                        reaction::Type::builder()
                            .name(literal!($name))
                            .description(literal!($description))
                            .catalysts(smallvec![
                                $(
                                    reaction::Catalyst::builder()
                                        .range(reaction::CatalystRange::$catalyst_variant {
                                            $(ty: $catalyst_type,)?
                                            levels: $catalyst_min.into() .. $catalyst_max.into(),
                                        })
                                        .multipliers(reaction::Multipliers::builder()
                                            .underflow($catalyst_underflow_mul)
                                            .min($catalyst_min_mul)
                                            .max($catalyst_max_mul)
                                            .overflow($catalyst_overflow_mul)
                                            .build()
                                        )
                                        .build(),
                                )*
                            ])
                            .puts(smallvec![
                                $(
                                    reaction::Put::$put_variant {
                                        $(ty: $put_type,)?
                                        base: Rate($put_rate.into()),
                                    },
                                )*
                            ])
                            .category($category_ident)
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
    };
}

reactions! {
    cargo liquid gas skill;

    electricity "Electricity" ("Electricity management.") {
        solar_power {
            name: "Solar power",
            description: "Generates [electricity](../../electricity) from sunlight.",
            catalysts: [
                Light {
                    levels: 0. .. 10.,
                    multipliers: [0., 0., 1., 1.],
                },
            ],
            puts: [
                Electricity {
                    rate: 100.,
                },
            ],
        }
    }

    industrial "Industrial" ("Industrial processes.") {
        asteroidal_water_filtration {
            name: "Asteroidal water filtration",
            description: "Filters out rocks and other sediments in asteroidal water.",
            catalysts: [
                Skill {
                    ty: skill.mechanic,
                    levels: 0. .. 20.,
                    multipliers: [1., 1., 2., 2.],
                },
            ],
            puts: [
                Liquid {
                    ty: liquid.asteroidal_water,
                    rate: -100.,
                },
                Liquid {
                    ty: liquid.filtered_water,
                    rate: 100.,
                },
                Cargo {
                    ty: cargo.rock,
                    rate: 10.,
                },
                Cargo {
                    ty: cargo.sediment,
                    rate: 10.,
                },
            ],
        }
    }

    education "Education" ("Train inhabitant skills.") {
        driving_lesson {
            name: "Driving lesson",
            description: "Assign an experienced driver to teach driving skills to other inhabitants.",
            catalysts: [
                Skill {
                    ty: skill.driving,
                    levels: 10. .. 20.,
                    multipliers: [0., 0., 1., 1.],
                },
            ],
            puts: [
                Skill {
                    ty: skill.driving,
                    rate: 0.03,
                },
            ],
        }
    }

    happiness "Happiness" ("Entertainment and correctional services.") {
        imprisonment {
            name: "Imprisonment",
            description: "Imprison criminals to recultivate their moral values and turn to a good citizen.",
            catalysts: [
                Skill {
                    ty: skill.teaching,
                    levels: 5. .. 50.,
                    multipliers: [0., 1., 1., 1.],
                },
            ],
            puts: [
                Skill {
                    ty: skill.satisfaction,
                    rate: 0.1,
                },
                Skill {
                    ty: skill.morality,
                    rate: 0.01,
                },
            ],
        }

        stadium {
            name: "Stadium",
            description: "Inhabitants can improve athletic skills slowly but gain happiness.",
            catalysts: [],
            puts: [
                Skill {
                    ty: skill.happiness,
                    rate: 0.2,
                },
                Skill {
                    ty: skill.athletic,
                    rate: 0.02,
                },
            ],
        }
    }
}
