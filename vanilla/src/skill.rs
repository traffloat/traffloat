//! Vanilla skill type definitions.

use traffloat_types::def::{skill, GameDefinition};

macro_rules! skills {
    (
        $($ident:ident {
            name: $name:literal,
            description: $description:literal,
        })*
    ) => {
        /// IDs assigned to the vanilla game definition.
        pub struct Ids {
            $(
                pub $ident: skill::TypeId,
            )*
        }

        /// Populates a [`GameDefinition`] with gas definition.
        pub fn populate(def: &mut GameDefinition) -> Ids {
            $(
                let $ident = def.add_skill(
                    skill::Type::builder()
                        .name(String::from($name))
                        .description(String::from($description))
                        .build()
                );
            )*

            Ids {
                $($ident,)*
            }
        }
    }
}

skills! {
    driving {
        name: "Driving",
        description: "When an inhabitant with good driving skill operates a vehicle, it can move faster across rails.",
    }
    mechanic {
        name: "Mechanic",
        description: "Inhabitants are trained as mechanics to operate factories more effectively.",
    }
    construction {
        name: "Construction",
        description: "Inhabitants are trained as builders to construct new buildings faster.",
    }
    teaching {
        name: "Teaching",
        description: "Inhabitants are trained as teachers so that the next generation can learn better in schools.",
    }
}
