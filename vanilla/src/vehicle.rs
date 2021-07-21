//! Vanilla vehicle type definitions.

use arcstr::literal;

use super::skill;
use traffloat_types::def::{reaction, vehicle, GameDefinition};

macro_rules! vehicles {
    (
        $skill:ident;

        $($ident:ident {
            name: $name:literal,
            description: $description:literal,
            speed: $speed:literal,
            capacity: $capacity:literal,
            passengers: $passengers:literal,
            skill: {
                ty: $skill_ty:expr,
                levels: [$skill_min_level:literal, $skill_max_level:literal],
                multipliers: [
                    $skill_underflow_mul:literal,
                    $skill_min_mul:literal,
                    $skill_max_mul:literal,
                    $skill_overflow_mul:literal
                ],
            },
            texture: $texture:literal,
        })*
    ) => {
        /// IDs assigned to the vanilla game definition.
        pub struct Ids {
            $(
                pub $ident: vehicle::TypeId,
            )*
        }

        /// Populates a [`GameDefinition`] with gas definition.
        pub fn populate(def: &mut GameDefinition, $skill: &skill::Ids) -> Ids {
            $(
                let $ident = def.add_vehicle(
                    vehicle::Type::builder()
                        .name(literal!($name))
                        .description(literal!($description))
                        .speed($speed.into())
                        .capacity($capacity.into())
                        .passengers($passengers)
                        .skill(vehicle::Skill::builder()
                               .skill($skill_ty)
                               .levels($skill_min_level.into()..$skill_max_level.into())
                               .multipliers(reaction::Multipliers::builder()
                                   .underflow($skill_underflow_mul)
                                   .min($skill_min_mul)
                                   .max($skill_max_mul)
                                   .overflow($skill_overflow_mul)
                                   .build())
                               .build())
                        .texture(literal!($texture))
                        .build()
                );
            )*

            Ids {
                $($ident,)*
            }
        }
    }
}

vehicles! {
    skill;

    raptor {
        name: "Raptor",
        description: "A fast, lightweight vehicle.",
        speed: 20.,
        capacity: 0.,
        passengers: 0,
        skill: {
            ty: skill.driving,
            levels: [0., 20.],
            multipliers: [1., 1., 2., 2.],
        },
        texture: "dummy",
    }

    freighter {
        name: "Freighter",
        description: "A slow vehicle used to carry large amounts of cargo",
        speed: 5.,
        capacity: 1000.,
        passengers: 0,
        skill: {
            ty: skill.driving,
            levels: [0., 20.],
            multipliers: [1., 1., 2., 2.],
        },
        texture: "dummy",
    }

    bus {
        name: "Bus",
        description: "A slow vehicle used to carry many passengers",
        speed: 5.,
        capacity: 0.,
        passengers: 16,
        skill: {
            ty: skill.driving,
            levels: [0., 20.],
            multipliers: [1., 1., 2., 2.],
        },
        texture: "dummy",
    }
}
